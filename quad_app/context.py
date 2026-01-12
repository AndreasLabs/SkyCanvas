from quad_app.systems.led import LED

class QuadContext:
    def __init__(self):
        self.mav_system = None
        self.led_system = LED()
        
        self.lla_current = None
        self.ned_current = None
        self.ned_history = []