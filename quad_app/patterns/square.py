"""Square pattern generation."""

from quad_app.patterns.base import PatternConfig
from quad_app.waypoints import Waypoint


def generate_square(
    config: PatternConfig,
    size: float = 4.0,
    points_per_side: int = 8
) -> list[Waypoint]:
    """Generate a 2D square pattern in 3D space.
    
    The pattern uses the X-Z plane (NED: North-Down) with East=0,
    same as the smiley face pattern.
    
    Args:
        config: Pattern configuration with center position and scale
        size: Side length of the square in meters (before scaling)
        points_per_side: Number of waypoints per side (minimum 2)
        
    Returns:
        List of waypoints forming a square
    """
    path = []
    center = config.center
    scale = config.scale
    color = config.default_color
    hold_time = config.hold_time
    
    # Ensure minimum points per side
    points_per_side = max(2, points_per_side)
    
    # Calculate half-size with scale applied
    half_size = (size * scale) / 2.0
    
    # Define the four corners of the square in NED coordinates
    # Top-left, top-right, bottom-right, bottom-left (clockwise)
    corners = [
        (center[0] - half_size, center[2] - half_size),  # Top-left (north-, down-)
        (center[0] + half_size, center[2] - half_size),  # Top-right (north+, down-)
        (center[0] + half_size, center[2] + half_size),  # Bottom-right (north+, down+)
        (center[0] - half_size, center[2] + half_size),  # Bottom-left (north-, down+)
    ]
    
    # Generate waypoints along each side
    for i in range(4):
        start_corner = corners[i]
        end_corner = corners[(i + 1) % 4]
        
        # Generate points along this side (excluding the end point to avoid duplicates)
        for j in range(points_per_side):
            t = j / points_per_side
            x = start_corner[0] + t * (end_corner[0] - start_corner[0])
            z = start_corner[1] + t * (end_corner[1] - start_corner[1])
            
            path.append(Waypoint(
                ned=[x, center[1], z],
                color=list(color),
                hold_time=hold_time
            ))
    
    return path
