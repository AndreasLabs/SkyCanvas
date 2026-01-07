import logging
import time
from quad_app.quad import Quad, QuadOptions
class QuadApp:
    def __init__(self):
        self.quad = Quad(QuadOptions())
        logging.info("QuadApp // Initializing QuadApp...")
    def run(self):
        logging.info("QuadApp // Intinmg QuadApp...")
        self.quad.init()
        logging.info("QuadApp // Running QuadApp...")
        while True:
            time.sleep(1)
            self.quad.run()