"""Smiley face pattern generation."""

import math
from quad_app.patterns.base import PatternConfig
from quad_app.waypoints import Waypoint


def generate_smiley(config: PatternConfig) -> list[Waypoint]:
    """Generate a smiley face pattern in 3D space.
    
    The pattern uses the X-Z plane (NED: North-Down) with East=0:
    - Face outline: circular path
    - Two eyes: small circles at the top
    - Smile: curved arc at the bottom
    
    Args:
        config: Pattern configuration with center position and scale
        
    Returns:
        List of waypoints forming a smiley face
    """
    path = []
    center = config.center
    scale = config.scale
    hold_time = config.hold_time
    
    # Face outline - circular (24 points)
    face_radius = 2.3 * scale
    for i in range(24):
        angle = (i / 24) * 2 * math.pi
        x = center[0] + face_radius * math.cos(angle)
        z = center[2] + face_radius * math.sin(angle)
        path.append(Waypoint(
            ned=[x, center[1], z],
            color=[1.0, 1.0, 0.0],  # Yellow for face
            hold_time=hold_time
        ))
    
    # Left eye - small circle (8 points)
    # Eyes at TOP of face (MORE negative z = higher altitude in NED)
    left_eye_offset = [-0.8 * scale, 0.0, -1.3 * scale]
    eye_radius = 0.3 * scale
    for i in range(8):
        angle = (i / 8) * 2 * math.pi
        x = center[0] + left_eye_offset[0] + eye_radius * math.cos(angle)
        z = center[2] + left_eye_offset[2] + eye_radius * math.sin(angle)
        path.append(Waypoint(
            ned=[x, center[1], z],
            color=[0.0, 0.0, 1.0],  # Blue for eyes
            hold_time=hold_time
        ))
    
    # Right eye - small circle (8 points)
    right_eye_offset = [0.8 * scale, 0.0, -1.3 * scale]
    for i in range(8):
        angle = (i / 8) * 2 * math.pi
        x = center[0] + right_eye_offset[0] + eye_radius * math.cos(angle)
        z = center[2] + right_eye_offset[2] + eye_radius * math.sin(angle)
        path.append(Waypoint(
            ned=[x, center[1], z],
            color=[0.0, 0.0, 1.0],  # Blue for eyes
            hold_time=hold_time
        ))
    
    # Smile - curved arc (16 points)
    # Smile at BOTTOM of face (LESS negative z = lower altitude in NED)
    # Arc from 180 to 360 degrees creates downward-curving smile (happy face)
    smile_offset = [0.0, 0.0, 1.3 * scale]
    smile_radius = 1.2 * scale
    for i in range(16):
        angle = math.radians(180 + i * 12)  # 180 to 360 degrees (12° increments)
        x = center[0] + smile_offset[0] + smile_radius * math.cos(angle)
        z = center[2] + smile_offset[2] + smile_radius * math.sin(angle)
        path.append(Waypoint(
            ned=[x, center[1], z],
            color=[1.0, 0.0, 0.0],  # Red for smile
            hold_time=hold_time
        ))
    
    return path
