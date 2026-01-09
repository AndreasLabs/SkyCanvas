import logging
import json
from mavsdk import System as MavSystem
import asyncio
import rerun as rr
from datetime import datetime
class QuadOptions:
    def __init__(self):
        self.connection_string = "tcpout://127.0.0.1:5760"
        self.telemetry_rate_hz = 5.0

    def set_connection_string(self, connection_string: str):
        self.connection_string = connection_string
    
    def set_telemetry_rate_hz(self, rate_hz: float):
        self.telemetry_rate_hz = rate_hz


class Quad:
    def __init__(self, options: QuadOptions):
        logging.info("Quad // Initializing")
        self.options = options
        self.mav_system = None
        self.home_altitude = None

    async def connect(self):
        """Connect to the MAVLink system"""
        logging.info(f"Quad // Connecting to {self.options.connection_string}")
        self.mav_system = MavSystem()
        await self.mav_system.connect(system_address=self.options.connection_string)
                # Wait for connection
        async for state in self.mav_system.core.connection_state():
            if state.is_connected:
                logging.info("Quad // Connected to drone")
                break
        
        # Request telemetry streams from ArduPilot (required for ArduPilot SITL/SIL)
        logging.info(f"Quad // Requesting telemetry streams at {self.options.telemetry_rate_hz} Hz")
        try:
            await self.mav_system.telemetry.set_rate_position(self.options.telemetry_rate_hz)
            await self.mav_system.telemetry.set_rate_battery(self.options.telemetry_rate_hz)
            await self.mav_system.telemetry.set_rate_in_air(self.options.telemetry_rate_hz)
            logging.info("Quad // Telemetry streams requested successfully")
        except Exception as e:
            logging.warning(f"Quad // Error requesting telemetry streams: {e}")
            logging.info("Quad // Continuing anyway...")

    async def wait_for_ready(self):
        """Wait for drone to be ready for flight"""
        logging.info("Quad // Waiting for drone to be ready")
        

        
        # Wait for global position
        logging.info("Quad // Waiting for global position estimate")
        async for health in self.mav_system.telemetry.health():
            if health.is_global_position_ok and health.is_home_position_ok:
                logging.info("Quad // Global position OK")
                break
        
        # Fetch home altitude
        async for terrain_info in self.mav_system.telemetry.home():
            self.home_altitude = terrain_info.absolute_altitude_m
            logging.info(f"Quad // Home altitude: {self.home_altitude}m")
            break
        
        logging.info("Quad // Ready for flight")

    async def arm(self):
        """Arm the drone"""
        logging.info("Quad // Arming")
        await self.mav_system.action.arm()

    async def takeoff(self):
        """Take off to default altitude"""
        logging.info("Quad // Taking off")
        await self.mav_system.action.takeoff()

    async def goto_location(self, latitude: float, longitude: float, altitude: float, yaw: float):
        """Fly to specified location"""
        logging.info(f"Quad // Going to lat={latitude}, lon={longitude}, alt={altitude}m, yaw={yaw}°")
        await self.mav_system.action.goto_location(latitude, longitude, altitude, yaw)

    async def land(self):
        """Land the drone"""
        logging.info("Quad // Landing")
        await self.mav_system.action.land()

    async def disarm(self):
        """Disarm the drone"""
        logging.info("Quad // Disarming")
        await self.mav_system.action.disarm()

    async def run(self):
        """Execute a simple test flight"""
        logging.info("Quad // Running test flight")
        
        # Start telemetry logging tasks
        _tasks = [
            asyncio.create_task(self.log_status_text()),
            asyncio.create_task(self.log_position()),
            asyncio.create_task(self.log_battery()),
            asyncio.create_task(self.log_in_air()),
        ]
        exit_event = asyncio.Event()
        await exit_event.wait()

    
    async def log_status_text(self):
        """Log status text messages from the drone"""
        try:
            logging.info("Quad // Starting status text logging")
            async for message in self.mav_system.telemetry.status_text():
                try:
                    logging.info(f" ==== ARDUPILOT // Message: {message}")
                    self.log_time_now()
                    rr.log("mavlink/status_text", rr.TextLog(message.text, level=rr.TextLogLevel.INFO))
                except Exception as e:
                    logging.error(f"Error in log_status_text iteration: {e}", exc_info=True)
        except Exception as e:
            logging.error(f"Fatal error in log_status_text: {e}", exc_info=True)
            raise
    
    async def log_position(self):
        async for position in self.mav_system.telemetry.position():
            # Log the altitudes as scalars
            rr.log("mavlink/position/absolute_altitude_m", rr.Scalars(position.absolute_altitude_m))
            rr.log("mavlink/position/relative_altitude_m", rr.Scalars(position.relative_altitude_m))
            
            # Log latitude_deg and longitude_deg as Geo
            rr.log("mavlink/position/lat_lon", rr.GeoPoints(lat_lon=[position.latitude_deg, position.longitude_deg]))
    
    def log_time_now(self):
        """Set the current time for rerun logging"""
        date_time = datetime.now()
        rr.set_time("realtime", timestamp=date_time)

    async def log_battery(self):
        async for battery in self.mav_system.telemetry.battery():
             self.log_time_now()
             #remaining_percent
             rr.log("mavlink/battery/remaining_percent", rr.Scalars(battery.remaining_percent))
             #voltage_v
             rr.log("mavlink/battery/voltage_v", rr.Scalars(battery.voltage_v))
    
    async def log_in_air(self):
        """Log in-air status"""
        try:
            logging.info("Quad // Starting in-air logging")
            async for in_air in self.mav_system.telemetry.in_air():
                try:
                    self.log_time_now()
                    
                    air_data = {"in_air": in_air}
                    rr.log("drone/in_air", rr.TextLog(json.dumps(air_data)))
                except Exception as e:
                    logging.error(f"Error in log_in_air iteration: {e}", exc_info=True)
        except Exception as e:
            logging.error(f"Fatal error in log_in_air: {e}", exc_info=True)
            raise
        