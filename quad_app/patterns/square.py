"""Square pattern generation."""

from quad_app.waypoints import Waypoint
from skycanvas_config import Config


def generate_square(
    size: float = 4.0,
    points_per_side: int = 8
) -> list[Waypoint]:
    """Generate a 2D square pattern in 3D space using global Config.
    
    The pattern uses the X-Z plane (NED: North-Down) with East=0,
    same as the smiley face pattern.
    
    Args:
        size: Side length of the square in meters (before scaling)
        points_per_side: Number of waypoints per side (minimum 2)
    
    Configuration is read from global Config singleton:
    - Config['mission.center']: NED center position
    - Config['mission.scale']: Scale factor
    - Config['mission.default_color']: RGB color
    - Config['mission.hold_time']: Time to hold at each waypoint
        
    Returns:
        List of waypoints forming a square
    """
    path = []
    center = Config.get('mission.center', [0.0, 0.0, -10.0])
    if isinstance(center, dict):
        # Handle Lua tables converted to dicts
        center = [center.get(i, 0.0) for i in range(1, 4)]
    scale = Config.get('mission.scale', 1.0)
    color = Config.get('mission.default_color', [1.0, 1.0, 1.0])
    hold_time = Config.get('mission.hold_time', 0.3)
    
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
