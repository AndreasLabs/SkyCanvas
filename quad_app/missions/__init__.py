"""Mission registry for SkyCanvas."""

from quad_app.missions.base import Mission
from quad_app.missions.smiley import SmileyMission
from quad_app.missions.pointcloud import PointcloudMission
from quad_app.missions.spiral import SpiralMission


# Mission registry mapping names to mission classes
MISSIONS = {
    "smiley": SmileyMission,
    "pointcloud": PointcloudMission,
    "spiral": SpiralMission,
}


def get_mission(name: str, config: dict = None) -> Mission:
    """Get a mission instance by name.
    
    Args:
        name: Mission name (must be in MISSIONS registry)
        config: Configuration dict to pass to mission constructor
        
    Returns:
        Mission instance
        
    Raises:
        ValueError: If mission name is not registered
    """
    if name not in MISSIONS:
        available = ", ".join(MISSIONS.keys())
        raise ValueError(f"Unknown mission '{name}'. Available missions: {available}")
    
    mission_class = MISSIONS[name]
    return mission_class(config)


__all__ = [
    "Mission",
    "SmileyMission",
    "PointcloudMission",
    "SpiralMission",
    "get_mission",
    "MISSIONS",
]
