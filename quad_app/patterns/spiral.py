"""3D spiral (DNA helix style) pattern generation."""

import math
from quad_app.waypoints import Waypoint
from skycanvas_config import Config


def generate_spiral() -> list[Waypoint]:
    """Generate a 3D spiral pattern (DNA helix style) using global Config.
    
    The spiral rises vertically (in NED: increasingly negative Down)
    while rotating in the North-East plane.
    
    Configuration is read from global Config singleton:
    - Config['mission.center']: NED center position (base of spiral)
    - Config['mission.scale']: Scale factor (affects radius and height)
    - Config['mission.hold_time']: Time to hold at each waypoint
    - Config['mission.spiral_turns']: Number of complete rotations (default: 3)
    - Config['mission.spiral_points']: Points per turn (default: 16)
    
    Returns:
        List of waypoints forming a 3D spiral
    """
    path = []
    center = Config.get('mission.center', [0.0, 0.0, -10.0])
    if isinstance(center, dict):
        # Handle Lua tables converted to dicts
        center = [center.get(i, 0.0) for i in range(1, 4)]
    scale = Config.get('mission.scale', 1.0)
    hold_time = Config.get('mission.hold_time', 0.1)
    
    # Spiral parameters
    num_turns = Config.get('mission.spiral_turns', 3)
    points_per_turn = Config.get('mission.spiral_points', 16)
    total_points = int(num_turns * points_per_turn)
    
    # Dimensions
    radius = 1.5 * scale          # Radius of the helix
    height = 4.0 * scale          # Total vertical rise (negative = up in NED)
    
    # Single segment for continuous line
    segment_id = 0
    
    for i in range(total_points):
        # Progress from 0 to 1
        t = i / (total_points - 1) if total_points > 1 else 0
        
        # Angle increases with each point
        angle = t * num_turns * 2 * math.pi
        
        # Position in North-East plane (horizontal circle)
        north = center[0] + radius * math.cos(angle)
        east = center[1] + radius * math.sin(angle)
        
        # Vertical position rises over time (more negative = higher in NED)
        down = center[2] - t * height
        
        # Color gradient: cycle through hues as we spiral up
        # HSV to RGB conversion for rainbow effect
        hue = t * 360  # Full spectrum over the spiral
        r, g, b = _hsv_to_rgb(hue, 1.0, 1.0)
        
        path.append(Waypoint(
            ned=[north, east, down],
            color=[r, g, b],
            hold_time=hold_time,
            segment_id=segment_id
        ))
    
    return path


def _hsv_to_rgb(h: float, s: float, v: float) -> tuple[float, float, float]:
    """Convert HSV to RGB color.
    
    Args:
        h: Hue in degrees (0-360)
        s: Saturation (0-1)
        v: Value/brightness (0-1)
        
    Returns:
        Tuple of (r, g, b) each in range 0-1
    """
    h = h % 360
    c = v * s
    x = c * (1 - abs((h / 60) % 2 - 1))
    m = v - c
    
    if h < 60:
        r, g, b = c, x, 0
    elif h < 120:
        r, g, b = x, c, 0
    elif h < 180:
        r, g, b = 0, c, x
    elif h < 240:
        r, g, b = 0, x, c
    elif h < 300:
        r, g, b = x, 0, c
    else:
        r, g, b = c, 0, x
    
    return (r + m, g + m, b + m)
