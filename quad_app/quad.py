import logging
import json
from typing import Any
from mavsdk import System as MavSystem
import asyncio
from datetime import datetime
from quad_app.context import QuadContext
from quad_app.quad_rerun import QuadRerun
from quad_app.systems import LED
from quad_app.waypoints import WaypointSystem, Waypoint
from quad_app.missions import get_mission
class QuadOptions:
    def __init__(self, config: dict):
        self.connection_string = config['connection_string']
        self.telemetry_rate_hz = config['telemetry_rate_hz']
    
class Quad:
    def __init__(self, options: QuadOptions, mission_config: dict = None):
        logging.info("Quad // Initializing")
        self.options = options
        self.context = QuadContext()
        self.waypoints = WaypointSystem()
        self.quad_rerun = QuadRerun("quad_app", self.context)
        
        # Load mission from config
        if mission_config is None:
            mission_config = {}
        
        mission_name = mission_config.get('name', 'smiley')
        logging.info(f"Quad // Loading mission: {mission_name}")
        self.mission = get_mission(mission_name, mission_config)


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
            await self.quad_rerun.log_dict("mavlink/health/raw", health)
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
            asyncio.create_task(self.fly_mission()),
            asyncio.create_task(self.run_waypoints()),
        ]
        
        await self.quad_rerun.start_log_tasks(self.waypoints)

        exit_event = asyncio.Event()
        await exit_event.wait()

    async def run_waypoints(self):
        logging.info("Quad // Running waypoints")
        await self.waypoints.run(self.context)

    
    async def fly_mission(self):
        """Execute the loaded mission."""
        logging.info(f"Quad // Flying mission: {self.mission.name}")
        await self.wait_for_ready()
        await self.arm()
        
        # Delegate to the mission
        await self.mission.run(self.context, self.waypoints)
