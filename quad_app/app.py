import logging
from quad_app.quad import Quad, QuadOptions


class QuadApp:
    def __init__(self):
        logging.info("QuadApp // Initializing")
        self.quad = Quad(QuadOptions())

    async def run(self):
        logging.info("QuadApp // Starting")
        await self.quad.connect()
        await self.quad.run()