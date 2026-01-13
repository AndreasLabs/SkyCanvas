"""Pointcloud mission - flies a 3D pattern from a PLY file."""

import logging
import asyncio
from quad_app.missions.base import Mission
from quad_app.context import QuadContext
from quad_app.waypoints import WaypointSystem
from quad_app.patterns import generate_from_pointcloud, PointcloudConfig


class PointcloudMission(Mission):
    """Flies a 3D pattern loaded from a PLY pointcloud file."""
    
    name = "pointcloud"
    
    def __init__(self, config: dict = None):
        """Initialize pointcloud mission.
        
        Args:
            config: Mission configuration dict from Lua config with:
                - ply_path: Path to PLY file
                - center: NED center position tuple (north, east, down)
                - scale: Scale factor
                - density: Minimum distance between points
                - depth_scale: Depth range (0 = flat, >0 = 2.5D)
                - hold_time: Time to hold at each waypoint
        """
        self.config = config or {}
        
    async def run(self, context: QuadContext, waypoints: WaypointSystem):
        """Execute the pointcloud mission.
        
        Args:
            context: QuadContext with mav_system, led_system, etc.
            waypoints: WaypointSystem for running waypoint sequences
        """
        logging.info("PointcloudMission // Starting")
        
        # Red LED for takeoff (hop)
        logging.info("PointcloudMission // Setting LED to RED for takeoff")
        context.led_system.rgb = [1.0, 0.0, 0.0]  # Red
        context.led_system.is_on = True
        
        await context.mav_system.action.takeoff()
        
        # Green LED while flying/hovering
        logging.info("PointcloudMission // Setting LED to GREEN while flying")
        context.led_system.rgb = [0.0, 1.0, 0.0]  # Green
        context.led_system.is_on = False
        
        # Wait for stabilization
        await asyncio.sleep(5)

        # Load pointcloud configuration from mission config
        ply_path = self.config.get('ply_path', 'data/test_images/depth_out/color_car1.ply')
        center = tuple(self.config.get('center', [5.0, 0.0, -5.0]))
        scale = self.config.get('scale', 3.0)
        density = self.config.get('density', 0.2)
        depth_scale = self.config.get('depth_scale', 1.0)
        hold_time = self.config.get('hold_time', 0.3)
        
        # Generate pointcloud pattern
        pattern_config = PointcloudConfig(
            ply_path=ply_path,
            center=center,
            scale=scale,
            density=density,
            depth_scale=depth_scale,
            hold_time=hold_time,
        )
        
        logging.info(f"PointcloudMission // Loading PLY from {ply_path}")
        path = generate_from_pointcloud(pattern_config)
        logging.info(f"PointcloudMission // Created pointcloud path with {len(path)} waypoints")
        
        # Execute the waypoint path
        await waypoints.run_path(path) 
        await waypoints.wait_until_disabled()
        await asyncio.sleep(2)
        
        # Blue LED for landing
        logging.info("PointcloudMission // Setting LED to BLUE for landing")
        context.led_system.rgb = [0.0, 0.0, 1.0]  # Blue
        await context.mav_system.action.land()
        
        # Wait for landing to complete
        await asyncio.sleep(10)
        
        # Turn off LED after disarm
        logging.info("PointcloudMission // Turning LED OFF")
        context.led_system.is_on = False
        await context.mav_system.action.disarm()
        
        logging.info("PointcloudMission // Complete")
