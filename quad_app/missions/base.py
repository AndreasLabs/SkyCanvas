"""Base class for all missions."""

from abc import ABC, abstractmethod
from quad_app.context import QuadContext
from quad_app.waypoints import WaypointSystem


class Mission(ABC):
    """Base class for all missions.
    
    Missions encapsulate the complete flight sequence including:
    - Takeoff
    - Waypoint pattern execution
    - Landing
    - LED control
    """
    
    name: str = "base"
    
    @abstractmethod
    async def run(self, context: QuadContext, waypoints: WaypointSystem):
        """Execute the mission.
        
        Args:
            context: QuadContext with mav_system, led_system, etc.
            waypoints: WaypointSystem for running waypoint sequences
        """
        pass
