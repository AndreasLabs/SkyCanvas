import logging
import asyncio
from enum import Enum
from mavsdk.offboard import OffboardError, PositionNedYaw


class Waypoint:
    def __init__(self, ned, color, brightness=1.0, hold_time=1.0, yaw_deg=0.0):
        self.ned = ned
        self.color = color
        self.brightness = brightness
        self.hold_time = hold_time
        self.yaw_deg = yaw_deg

class WaypointState(Enum):
    HOLD = 0
    COMMAND_GOTO = 1
    GOTO = 2
    REACHED = 3

class WaypointSystem:
    def __init__(self):
        self.current_waypoint = None
        self.time_start_hold = None
        self.state = WaypointState.HOLD
        self.offboard_active = False
        self.last_position_ned = None

    async def command_goto(self, waypoint):
        # Only allow if in hold
        if self.state != WaypointState.HOLD:
            logging.error(f"WaypointSystem // Cannot command goto if not in hold")
            return
        if self.current_waypoint is not None:
            logging.error(f"WaypointSystem // Cannot command goto if already have a waypoint")
            return
        self.current_waypoint = waypoint
        self.state = WaypointState.COMMAND_GOTO
        
    async def update_last_position_ned(self, position_ned):
        self.last_position_ned = position_ned
    
    async def run(self, mav_system):
        while True:
            await self.tick_state_machine(mav_system)
            await asyncio.sleep(0.1)
    
    async def tick_state_machine(self, mav_system):
        if self.state == WaypointState.HOLD:
            await self.tick_hold(mav_system)
        elif self.state == WaypointState.COMMAND_GOTO:
            await self.tick_command_goto(mav_system)
        elif self.state == WaypointState.GOTO:
            await self.tick_goto(mav_system)
        elif self.state == WaypointState.REACHED:
            await self.tick_reached(mav_system)
        else:
            logging.error(f"WaypointSystem // Invalid state: {self.state}")
    
    async def tick_hold(self, mav_system):
        pass

    async def tick_command_goto(self, mav_system):
        logging.info(f"WaypointSystem // COMMAND_GOTO - Starting offboard mode")
        
        try:
            # Set initial setpoint to target position
            target_ned = PositionNedYaw(
                self.current_waypoint.ned[0],
                self.current_waypoint.ned[1],
                self.current_waypoint.ned[2],
                self.current_waypoint.yaw_deg
            )
            await mav_system.offboard.set_position_ned(target_ned)
            
            # Start offboard mode
            await mav_system.offboard.start()
            self.offboard_active = True
            logging.info(f"WaypointSystem // Offboard mode started, going to NED: {self.current_waypoint.ned}")
            
            self.state = WaypointState.GOTO
            
        except OffboardError as e:
            logging.error(f"WaypointSystem // Failed to start offboard mode: {e}")
            self.state = WaypointState.HOLD
            self.current_waypoint = None

    async def tick_goto(self, mav_system):
        if self.last_position_ned is None:
            logging.error(f"WaypointSystem // No last position NED")
            return
        position_ned = self.last_position_ned 
        # Calculate distance in NED coordinates
        north_diff = position_ned.position.north_m - self.current_waypoint.ned[0]
        east_diff = position_ned.position.east_m - self.current_waypoint.ned[1]
        down_diff = position_ned.position.down_m - self.current_waypoint.ned[2]
        
        distance_m = (north_diff**2 + east_diff**2 + down_diff**2)**0.5
        
        logging.info(f"WaypointSystem // GOTO - Distance to waypoint: {distance_m:.2f}m")
        
        if distance_m < 0.5:
            logging.info(f"WaypointSystem // Reached waypoint!")
            self.state = WaypointState.REACHED

    async def tick_reached(self, mav_system):
        logging.info(f"WaypointSystem // REACHED - Stopping offboard mode")
        
        self.current_waypoint = None
        self.state = WaypointState.HOLD
    
    

