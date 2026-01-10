"""
LED System - Basic LED state storage
"""

import rerun as rr


class LED:
    """Simple LED with RGB, brightness, and on/off state"""
    
    def __init__(self):
        self.rgb = [1.0, 1.0, 1.0]  # [red, green, blue] 0.0 to 1.0
        self.brightness = 1.0  # 0.0 to 1.0
        self.is_on = True
    
    def to_rerun_color(self):
        if not self.is_on:
            return rr.components.Color([0, 0, 0, 0])
        
        # Apply brightness to RGB and convert to 0-255 range
        r = int(self.rgb[0] * self.brightness * 255)
        g = int(self.rgb[1] * self.brightness * 255)
        b = int(self.rgb[2] * self.brightness * 255)
        a = 255  # Fully opaque
        
        return rr.components.Color([r, g, b, a])
