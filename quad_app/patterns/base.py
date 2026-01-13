"""Base configuration classes for pattern generation."""

from dataclasses import dataclass


@dataclass
class PatternConfig:
    """Base configuration for pattern generation.
    
    Attributes:
        center: NED center position (north, east, down) in meters
        scale: Scale factor for the pattern
        default_color: RGB color values (0.0 to 1.0)
        hold_time: Time to hold at each waypoint in seconds
    """
    center: tuple[float, float, float]
    scale: float = 1.0
    default_color: tuple[float, float, float] = (1.0, 1.0, 1.0)
    hold_time: float = 1.0


@dataclass
class PointcloudConfig(PatternConfig):
    """Configuration for pointcloud-based pattern generation.
    
    Attributes:
        ply_path: Path to PLY pointcloud file
        density: Minimum distance between points in meters
        depth_scale: Max depth range in meters (0 = flat/2D, >0 = 2.5D relief)
    """
    ply_path: str = ""
    density: float = 0.1
    depth_scale: float = 0.0
