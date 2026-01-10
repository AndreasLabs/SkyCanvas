import logging
import json
from typing import Any
from mavsdk import System as MavSystem
import asyncio
import rerun as rr
from datetime import datetime
from quad_app.quad_rerun import QuadRerun
from quad_app.systems import LED
class QuadOptions:
    def __init__(self):
        self.connection_string = "tcpout://127.0.0.1:5760"
        self.telemetry_rate_hz = 25.0

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
        self.quad_rerun = QuadRerun("quad_app")
        self.led = LED()



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
            await self.mav_system.telemetry.set_rate_health(self.options.telemetry_rate_hz)  # Includes EKF status
            await self.mav_system.telemetry.set_rate_position(self.options.telemetry_rate_hz)
            await self.mav_system.telemetry.set_rate_position_velocity_ned(self.options.telemetry_rate_hz)
            await self.mav_system.telemetry.set_rate_battery(self.options.telemetry_rate_hz)
            await self.mav_system.telemetry.set_rate_in_air(self.options.telemetry_rate_hz)
            await self.mav_system.telemetry.set_rate_gps_info(self.options.telemetry_rate_hz)  # GPS satellite info
            logging.info("Quad // Telemetry streams requested successfully (including EKF status)")
        except Exception as e:
            logging.warning(f"Quad // Error requesting telemetry streams: {e}")
            logging.info("Quad // Continuing anyway...")

    async def wait_for_ready(self):
        """Wait for drone to be ready for flight - includes EKF initialization checks"""
        logging.info("Quad // Waiting for drone to be ready")
        

        # Wait for EKF to initialize with local and global position estimates
        logging.info("Quad // Waiting for EKF initialization (local & global position)")
        async for health in self.mav_system.telemetry.health():
            self.log_dict("mavlink/health/raw", health)
            if health.is_local_position_ok and health.is_global_position_ok and health.is_home_position_ok:
                logging.info("Quad // EKF Ready - Local position OK, Global position OK, Home position OK")
                break
        
        # Wait longer for EKF variance to stabilize after initial lock
        logging.info("Quad // Waiting 15s for EKF variance to stabilize...")
        await asyncio.sleep(20)

        logging.info("Quad // Ready for flight")

    async def arm(self):
        """Arm the drone"""
        logging.info("Quad // Arming")
        await self.mav_system.action.arm()
        
        # Wait for armed confirmatio
        logging.info("Quad // Waiting for armed confirmation")
        async for armed in self.mav_system.telemetry.armed():
            if armed:
                logging.info("Quad // Armed")
                break
            else:
                logging.info("Quad // Not armed")
                await asyncio.sleep(1)

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
        await self.quad_rerun.init()
        logging.info("Quad // Running test flight")
        
        # Start telemetry logging tasks
        _tasks = [
            asyncio.create_task(self.log_status_text()),
            asyncio.create_task(self.log_position()),
            asyncio.create_task(self.log_position_ned()),
            asyncio.create_task(self.log_battery()),
            asyncio.create_task(self.log_gps_info()),
            asyncio.create_task(self.log_in_air()),
            asyncio.create_task(self.log_led()),
            asyncio.create_task(self.fly_mission()),
        ]
        
        

        exit_event = asyncio.Event()
        await exit_event.wait()

    
    async def fly_mission(self):
        # Run our current desired actions after drone is ready.
        # currently just wait for ready, arm, takeoff, land, disarm
        logging.info("Quad // Flying mission")
        await self.wait_for_ready()
        
        await self.arm()
        
        # Red LED for takeoff (hop)
        logging.info("Quad // Setting LED to RED for takeoff")
        self.led.rgb = [1.0, 0.0, 0.0]  # Red
        self.led.is_on = True
        
        await self.takeoff()
        
        # Green LED while flying/hovering
        logging.info("Quad // Setting LED to GREEN while flying")
        self.led.rgb = [0.0, 1.0, 0.0]  # Green
        
        # Wait for 10 seconds
        await asyncio.sleep(10)
        
        # Blue LED for landing
        logging.info("Quad // Setting LED to BLUE for landing")
        self.led.rgb = [0.0, 0.0, 1.0]  # Blue
        await self.land()
        
        # Wait for 10 seconds
        await asyncio.sleep(10)
        
        # Turn off LED after disarm
        logging.info("Quad // Turning LED OFF")
        self.led.is_on = False
        await self.disarm()
    
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
            self.log_time_now()
            self.log_dict("mavlink/position/raw", position)
            # Log the altitudes as scalars
            rr.log("mavlink/position/absolute_altitude_m", rr.Scalars(position.absolute_altitude_m))
            rr.log("mavlink/position/relative_altitude_m", rr.Scalars(position.relative_altitude_m))
            
            # Log latitude_deg and longitude_deg as Geo
            rr.log("mavlink/position/lat_lon", rr.GeoPoints(lat_lon=[position.latitude_deg, position.longitude_deg]))
    
    async def log_position_ned(self):
        """Log local position in NED (North-East-Down) coordinates"""
        try:
            logging.info("Quad // Starting local position NED logging")
            async for position_ned in self.mav_system.telemetry.position_velocity_ned():
                try:
                    self.log_time_now()
                    self.log_dict("mavlink/position_ned/raw", position_ned)
                    # Log NED position coordinates as scalars
                    rr.log("mavlink/position_ned/north_m", rr.Scalars(position_ned.position.north_m))
                    rr.log("mavlink/position_ned/east_m", rr.Scalars(position_ned.position.east_m))
                    rr.log("mavlink/position_ned/down_m", rr.Scalars(position_ned.position.down_m))
                    # Log NED velocity coordinates as scalars
                    rr.log("mavlink/velocity_ned/north_m_s", rr.Scalars(position_ned.velocity.north_m_s))
                    rr.log("mavlink/velocity_ned/east_m_s", rr.Scalars(position_ned.velocity.east_m_s))
                    rr.log("mavlink/velocity_ned/down_m_s", rr.Scalars(position_ned.velocity.down_m_s))
                    
                    # Log 3d point
                    color = self.led.to_rerun_color()
                    position = [position_ned.position.north_m, position_ned.position.east_m, -position_ned.position.down_m]
                    rr.log("mavlink/position_ned/points", rr.Points3D([position], radii=0.2, labels=["Quad"], show_labels=True, colors=[color]))
     


                except Exception as e:
                    logging.error(f"Error in log_position_ned iteration: {e}", exc_info=True)
        except Exception as e:
            logging.error(f"Fatal error in log_position_ned: {e}", exc_info=True)
            raise
    
    def log_time_now(self):
        """Set the current time for rerun logging"""
        date_time = datetime.now()
        rr.set_time("realtime", timestamp=date_time)

    def log_dict(self, path: str, obj: Any):
        """Log a python object as a pretty JSON string in a TextDocument"""
        self.log_time_now()
        try:
            # Handle objects that might not be directly serializable
            def default_converter(o):
                if hasattr(o, "__dict__"):
                    return o.__dict__
                return str(o)

            pretty_json = json.dumps(obj, default=default_converter, indent=2)
            markdown_content = f"```json\n{pretty_json}\n```"
            
            rr.log(
                path,
                rr.TextDocument(
                    markdown_content,
                    media_type=rr.MediaType.MARKDOWN,
                ),
            )
        except Exception as e:
            logging.error(f"Error logging dict to {path}: {e}")

    async def log_battery(self):
        async for battery in self.mav_system.telemetry.battery():
             self.log_time_now()
             self.log_dict("mavlink/battery/raw", battery)
             #remaining_percent
             rr.log("mavlink/battery/remaining_percent", rr.Scalars(battery.remaining_percent))
             #voltage_v
             rr.log("mavlink/battery/voltage_v", rr.Scalars(battery.voltage_v))
    
    async def log_gps_info(self):
        """Log GPS information including satellite count and fix type"""
        try:
            logging.info("Quad // Starting GPS info logging")
            async for gps_info in self.mav_system.telemetry.gps_info():
                try:
                    self.log_time_now()
                    # Log satellite count
                    rr.log("mavlink/gps/num_satellites", rr.Scalars(gps_info.num_satellites))
                    # Log fix type (0=none, 1=no fix, 2=2D, 3=3D, 4=DGPS, 5=RTK float, 6=RTK fixed)
                    #rr.log("mavlink/gps/fix_type", rr.Scalars(int(gps_info.fix_type)))
                except Exception as e:
                    logging.error(f"Error in log_gps_info iteration: {e}", exc_info=True)
        except Exception as e:
            logging.error(f"Fatal error in log_gps_info: {e}", exc_info=True)
            raise
    
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
    
    async def log_led(self):
        """Log LED state to Rerun"""
        try:
            logging.info("Quad // Starting LED logging")
            while True:
                try:
                    self.log_time_now()
                    
                    # Log LED state as JSON
                    led_data = {
                        "rgb": self.led.rgb,
                        "brightness": self.led.brightness,
                        "is_on": self.led.is_on
                    }
                    self.log_dict("led/state", led_data)
                    
                    await asyncio.sleep(0.1)  # Log at ~10Hz
                except Exception as e:
                    logging.error(f"Error in log_led iteration: {e}", exc_info=True)
        except Exception as e:
            logging.error(f"Fatal error in log_led: {e}", exc_info=True)
            raise
        