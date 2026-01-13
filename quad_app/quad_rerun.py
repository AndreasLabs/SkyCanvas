import rerun as rr
import numpy as np
import logging
from quad_app.context import QuadContext
import asyncio
from datetime import datetime
import json
from typing import Any
class QuadRerun:
    def __init__(self, name: str, context: QuadContext):
        self.name = name
        self.context = context
        self.initialized = False
    async def init(self):
        logging.info(f"QuadRerun // Initializing {self.name}")
        rr.init(self.name, spawn=True)
        self.initialized = True

    async def smoketest_log(self):
      
        if not self.initialized:
            logging.warning(f"QuadRerun // Not initialized, initializing {self.name}")
            await self.init()
        SIZE = 10

        pos_grid = np.meshgrid(*[np.linspace(-10, 10, SIZE)]*3)
        positions = np.vstack([d.reshape(-1) for d in pos_grid]).T

        col_grid = np.meshgrid(*[np.linspace(0, 255, SIZE)]*3)
        colors = np.vstack([c.reshape(-1) for c in col_grid]).astype(np.uint8).T

        rr.log(
            "smoketest/points3d",
            rr.Points3D(positions, colors=colors, radii=0.5)
        )
    
    async def start_log_tasks(self, waypoints):
        logging.info(f"QuadRerun // Starting log tasks for {self.name}")
        # Start the log tasks
        _tasks = [
            asyncio.create_task(self.log_position_geo()),
            asyncio.create_task(self.log_status_text()),
            asyncio.create_task(self.log_position_ned(waypoints)),
            asyncio.create_task(self.log_battery()),
            asyncio.create_task(self.log_gps_info()),
            asyncio.create_task(self.log_in_air()),
            asyncio.create_task(self.log_led()),
            asyncio.create_task(self.log_exposure_history()),
        ]
        logging.info(f"QuadRerun // Log tasks started")
        
    def log_time_now(self):
        date_time = datetime.now()
        rr.set_time("realtime", timestamp=date_time)

    async def log_dict(self, path: str, obj: Any):
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

    async def log_position_geo(self):
        async for position in self.context.mav_system.telemetry.position():
            self.log_time_now()
            await self.log_dict("mavlink/position/raw", position)
            # Log the altitudes as scalars
            rr.log("mavlink/position/absolute_altitude_m", rr.Scalars(position.absolute_altitude_m))
            self.context.lla_current = [position.latitude_deg, position.longitude_deg, position.absolute_altitude_m]
            rr.log("mavlink/position/relative_altitude_m", rr.Scalars(position.relative_altitude_m))
            
            # Log latitude_deg and longitude_deg as Geo
            rr.log("mavlink/position/lat_lon", rr.GeoPoints(lat_lon=[position.latitude_deg, position.longitude_deg]))
    
    async def log_status_text(self):
        """Log status text messages from the drone"""
        try:
            logging.info("QuadRerun // Starting status text logging")
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
    
    async def log_position_ned(self, waypoints):
        """Log local position in NED (North-East-Down) coordinates"""
        try:
            logging.info("QuadRerun // Starting local position NED logging")
            async for position_ned in self.context.mav_system.telemetry.position_velocity_ned():
                try:
                    await waypoints.update_last_position_ned(position_ned)
                    self.log_time_now()
                    await self.log_dict("mavlink/position_ned/raw", position_ned)
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
          #  logging.info(f"QuadRerun // Exposure history: {len(self.context.ned_history)}")
        
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
                    logging.info(f"QuadRerun // Added new entry to exposure history: {current_entry}")
                # If there is a last entry - if the current position is at least 0.01m away from the last entry, add a new entry
                elif len(self.context.ned_history) > 0:
                    last_entry = self.context.ned_history[-1]
                    if abs(self.context.ned_current[0] - last_entry["position"][0]) > 0.01 or abs(self.context.ned_current[1] - last_entry["position"][1]) > 0.01 or abs(self.context.ned_current[2] - last_entry["position"][2]) > 0.01:
                        self.context.ned_history.append(current_entry)
                      #  logging.info(f"QuadRerun // Added new entry to exposure history: {current_entry}")
            
            # Log the exposure history as Points3D
            if len(self.context.ned_history) > 0:
                # 2d is the X (east) and Alt (0, and 2, index)
                pos_2d = [[entry["position"][0], -entry["position"][2]] for entry in self.context.ned_history]
                rr.log("exposure/history/2d", rr.Points2D(pos_2d, colors=[entry["color"] for entry in self.context.ned_history], radii=0.05))
                rr.log("exposure/history/3d", rr.Points3D([entry["position"] for entry in self.context.ned_history], colors=[entry["color"] for entry in self.context.ned_history], radii=0.05))
            # Run at 20hz
            await asyncio.sleep(0.02)

    async def log_battery(self):
        async for battery in self.context.mav_system.telemetry.battery():
             self.log_time_now()
             await self.log_dict("mavlink/battery/raw", battery)
             #remaining_percent
             rr.log("mavlink/battery/remaining_percent", rr.Scalars(battery.remaining_percent))
             #voltage_v
             rr.log("mavlink/battery/voltage_v", rr.Scalars(battery.voltage_v))
    
    async def log_gps_info(self):
        """Log GPS information including satellite count and fix type"""
        try:
            logging.info("QuadRerun // Starting GPS info logging")
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
            logging.info("QuadRerun // Starting in-air logging")
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
            await self.log_dict("led/state", led_data)
            await asyncio.sleep(0.02)  # Log at ~50Hz
    