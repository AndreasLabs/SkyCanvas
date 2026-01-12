import logging
import json
from typing import Any
from mavsdk import System as MavSystem
import asyncio
import rerun as rr
from datetime import datetime
from quad_app.context import QuadContext
from quad_app.quad_rerun import QuadRerun
from quad_app.systems import LED
from quad_app.waypoints import WaypointSystem, Waypoint
class QuadOptions:
    def __init__(self):
        self.connection_string = "tcpout://127.0.0.1:5760"
        self.telemetry_rate_hz = 50.0

    def set_connection_string(self, connection_string: str):
        self.connection_string = connection_string
    
    def set_telemetry_rate_hz(self, rate_hz: float):
        self.telemetry_rate_hz = rate_hz



    
class Quad:
    def __init__(self, options: QuadOptions):
        logging.info("Quad // Initializing")
        self.options = options
        self.context = QuadContext()
        self.waypoints = WaypointSystem()
        self.quad_rerun = QuadRerun("quad_app", self.context)


    async def connect(self):
        """Connect to the MAVLink system"""
        logging.info(f"Quad // Connecting to {self.options.connection_string}")
        self.context.mav_system = MavSystem()
        await self.context.mav_system.connect(system_address=self.options.connection_string)
                # Wait for connection
        async for state in self.context.mav_system.core.connection_state():
            if state.is_connected:
                logging.info("Quad // Connected to drone")
                break
        
        # Request telemetry streams from ArduPilot (required for ArduPilot SITL/SIL)
        logging.info(f"Quad // Requesting telemetry streams at {self.options.telemetry_rate_hz} Hz")
        try:
            await self.context.mav_system.telemetry.set_rate_health(self.options.telemetry_rate_hz)  # Includes EKF status
            await self.context.mav_system.telemetry.set_rate_position(self.options.telemetry_rate_hz)
            await self.context.mav_system.telemetry.set_rate_position_velocity_ned(self.options.telemetry_rate_hz)
            await self.context.mav_system.telemetry.set_rate_battery(self.options.telemetry_rate_hz)
            await self.context.mav_system.telemetry.set_rate_in_air(self.options.telemetry_rate_hz)
            await self.context.mav_system.telemetry.set_rate_gps_info(self.options.telemetry_rate_hz)  # GPS satellite info
            logging.info("Quad // Telemetry streams requested successfully (including EKF status)")
        except Exception as e:
            logging.warning(f"Quad // Error requesting telemetry streams: {e}")
            logging.info("Quad // Continuing anyway...")

    async def wait_for_ready(self):
        """Wait for drone to be ready for flight - includes EKF initialization checks"""
        logging.info("Quad // Waiting for drone to be ready")
        

        # Wait for EKF to initialize with local and global position estimates
        logging.info("Quad // Waiting for EKF initialization (local & global position)")
        async for health in self.context.mav_system.telemetry.health():
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
        await self.context.mav_system.action.arm()
        
        # Wait for armed confirmatio
        logging.info("Quad // Waiting for armed confirmation")
        async for armed in self.context.mav_system.telemetry.armed():
            if armed:
                logging.info("Quad // Armed")
                break
            else:
                logging.info("Quad // Not armed")
                await asyncio.sleep(1)

    async def takeoff(self):
        """Take off to default altitude"""
        logging.info("Quad // Taking off")
        await self.context.mav_system.action.takeoff()

    async def goto_location(self, latitude: float, longitude: float, altitude: float, yaw: float):
        """Fly to specified location"""
        logging.info(f"Quad // Going to lat={latitude}, lon={longitude}, alt={altitude}m, yaw={yaw}°")
        await self.context.mav_system.action.goto_location(latitude, longitude, altitude, yaw)

    async def land(self):
        """Land the drone"""
        logging.info("Quad // Landing")
        await self.context.mav_system.action.land()

    async def disarm(self):
        """Disarm the drone"""
        logging.info("Quad // Disarming")
        await self.context.mav_system.action.disarm()

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
            asyncio.create_task(self.log_exposure_history()),
            asyncio.create_task(self.run_waypoints()),
        ]
        
        

        exit_event = asyncio.Event()
        await exit_event.wait()

    async def run_waypoints(self):
        logging.info("Quad // Running waypoints")
        await self.waypoints.run(self.context)

    
    async def fly_mission(self):
        logging.info("Quad // Flying mission")
        await self.wait_for_ready()
        await self.arm()
        
        # Red LED for takeoff (hop)
        logging.info("Quad // Setting LED to RED for takeoff")
        self.context.led_system.rgb = [1.0, 0.0, 0.0]  # Red
        self.context.led_system.is_on = True
        
        await self.takeoff()
        
        # Green LED while flying/hovering
        logging.info("Quad // Setting LED to GREEN while flying")
        self.context.led_system.rgb = [0.0, 1.0, 0.0]  # Green
        self.context.led_system.is_on = False
        # Wait for 10 seconds
        await asyncio.sleep(10)

        # Command goto waypoint in NED coordinates
        # Go 10m North, 10m East, -10m Down (up in NED), face East (90°)
        waypoint = Waypoint(
            ned=[10.0, 10.0, -10.0],
            color=[0.0, 1.0, 1.0],
            yaw_deg=90.0
        )
        await self.waypoints.command_goto(waypoint)
        # Wait 20s to reach waypoint
        await asyncio.sleep(20)
        # Blue LED for landing
        logging.info("Quad // Setting LED to BLUE for landing")
        self.context.led_system.rgb = [0.0, 0.0, 1.0]  # Blue
        await self.land()
        
        # Wait for 10 seconds
        await asyncio.sleep(10)
        
        # Turn off LED after disarm
        logging.info("Quad // Turning LED OFF")
        self.context.led_system.is_on = False
        await self.disarm()
    
    async def log_status_text(self):
        """Log status text messages from the drone"""
        try:
            logging.info("Quad // Starting status text logging")
            async for message in self.context.mav_system.telemetry.status_text():
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
        async for position in self.context.mav_system.telemetry.position():
            self.log_time_now()
            self.log_dict("mavlink/position/raw", position)
            # Log the altitudes as scalars
            rr.log("mavlink/position/absolute_altitude_m", rr.Scalars(position.absolute_altitude_m))
            self.context.lla_current = [position.latitude_deg, position.longitude_deg, position.absolute_altitude_m]
            rr.log("mavlink/position/relative_altitude_m", rr.Scalars(position.relative_altitude_m))
            
            # Log latitude_deg and longitude_deg as Geo
            rr.log("mavlink/position/lat_lon", rr.GeoPoints(lat_lon=[position.latitude_deg, position.longitude_deg]))
    
    async def log_position_ned(self):
        """Log local position in NED (North-East-Down) coordinates"""
        try:
            logging.info("Quad // Starting local position NED logging")
            async for position_ned in self.context.mav_system.telemetry.position_velocity_ned():
                try:
                    await self.waypoints.update_last_position_ned(position_ned)
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
                    color = self.context.led_system.to_rerun_color()
                    self.context.ned_current = [position_ned.position.north_m, position_ned.position.east_m, -position_ned.position.down_m]
                    rr.log("mavlink/position_ned/points", rr.Points3D([self.context.ned_current], radii=0.2, labels=["Quad"], show_labels=True, colors=[color]))
     


                except Exception as e:
                    logging.error(f"Error in log_position_ned iteration: {e}", exc_info=True)
        except Exception as e:
            logging.error(f"Fatal error in log_position_ned: {e}", exc_info=True)
            raise
    
    async def log_exposure_history(self):
        """Log the exposure history"""
        while True:
            self.log_time_now()
          #  logging.info(f"Quad // Exposure history: {len(self.context.ned_history)}")
        
            # Only track entries when LED is on
            if self.context.led_system.is_on and self.context.ned_current is not None:
                # Position is [north_m, east_m, -down_m]
                current_entry = {
                    "position": self.context.ned_current,
                    "color": self.context.led_system.rgb,
                    "brightness": self.context.led_system.brightness
                }
                
                # If empty, add the current position
                if len(self.context.ned_history) == 0:
                    self.context.ned_history.append(current_entry)
                    logging.info(f"Quad // Added new entry to exposure history: {current_entry}")
                # If there is a last entry - if the current position is at least 0.01m away from the last entry, add a new entry
                elif len(self.context.ned_history) > 0:
                    last_entry = self.context.ned_history[-1]
                    if abs(self.context.ned_current[0] - last_entry["position"][0]) > 0.01 or abs(self.context.ned_current[1] - last_entry["position"][1]) > 0.01 or abs(self.context.ned_current[2] - last_entry["position"][2]) > 0.01:
                        self.context.ned_history.append(current_entry)
                      #  logging.info(f"Quad // Added new entry to exposure history: {current_entry}")
            
            # Log the exposure history as Points3D
            if len(self.context.ned_history) > 0:
                rr.log("exposure/history/2d", rr.Points2D([entry["position"][:2] for entry in self.context.ned_history], colors=[entry["color"] for entry in self.context.ned_history], radii=0.05))
                rr.log("exposure/history/3d", rr.Points3D([entry["position"] for entry in self.context.ned_history], colors=[entry["color"] for entry in self.context.ned_history], radii=0.05))
            # Run at 20hz
            await asyncio.sleep(0.02)
    
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
        async for battery in self.context.mav_system.telemetry.battery():
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
            async for gps_info in self.context.mav_system.telemetry.gps_info():
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
            async for in_air in self.context.mav_system.telemetry.in_air():
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
        while True:
            self.log_time_now()
            # Log LED state as JSON
            led_data = {
                "rgb": self.context.led_system.rgb,
                "brightness": self.context.led_system.brightness,
                "is_on": self.context.led_system.is_on
            }
            self.log_dict("led/state", led_data)
            await asyncio.sleep(0.02)  # Log at ~50Hz