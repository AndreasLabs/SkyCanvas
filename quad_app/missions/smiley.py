"""Smiley face mission - flies a smiley pattern."""

import logging
import asyncio
from quad_app.missions.base import Mission
from quad_app.context import QuadContext
from quad_app.waypoints import WaypointSystem
from quad_app.patterns import generate_smiley


class SmileyMission(Mission):
    """Flies a smiley face pattern in the air."""
    
    name = "smiley"
    
    def __init__(self, config: dict = None):
        """Initialize smiley mission.
        
        Args:
            config: Mission configuration dict from Lua config
        """
        self.config = config or {}
        
    async def run(self, context: QuadContext, waypoints: WaypointSystem):
        """Execute the smiley face mission.
        
        Args:
            context: QuadContext with mav_system, led_system, etc.
            waypoints: WaypointSystem for running waypoint sequences
        """
        logging.info("SmileyMission // Starting")
        
        # Red LED for takeoff (hop)
        logging.info("SmileyMission // Setting LED to RED for takeoff")
        context.led_system.rgb = [1.0, 0.0, 0.0]  # Red
        context.led_system.is_on = True
        
        await context.mav_system.action.takeoff()
        
        # Green LED while flying/hovering
        logging.info("SmileyMission // Setting LED to GREEN while flying")
        context.led_system.rgb = [0.0, 1.0, 0.0]  # Green
        context.led_system.is_on = False
        
        # Wait for stabilization
        await asyncio.sleep(5)

        # Generate smiley face pattern (reads config from global Config)
        path = generate_smiley()
        logging.info(f"SmileyMission // Created smiley face path with {len(path)} waypoints")
        
        # Execute the waypoint path
        await waypoints.run_path(path) 
        await waypoints.wait_until_disabled()
        await asyncio.sleep(2)
        
        # Blue LED for landing
        logging.info("SmileyMission // Setting LED to BLUE for landing")
        context.led_system.rgb = [0.0, 0.0, 1.0]  # Blue
        await context.mav_system.action.land()
        
        # Wait for landing to complete
        await asyncio.sleep(10)
        
        # Turn off LED after disarm
        logging.info("SmileyMission // Turning LED OFF")
        context.led_system.is_on = False
        await context.mav_system.action.disarm()
        
        logging.info("SmileyMission // Complete")
