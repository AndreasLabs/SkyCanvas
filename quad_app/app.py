import logging
import time
from quad_app.quad import Quad, QuadOptions
class QuadApp:
    def __init__(self):
        self.quad = Quad(QuadOptions())
        logging.info("QuadApp // Initializing QuadApp...")
    async def run(self):
        logging.info("QuadApp // Intinmg QuadApp...")
        await self.quad.init()
        logging.info("QuadApp // Running QuadApp...")
        await self.quad.run()
           