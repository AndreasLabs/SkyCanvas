import logging
from quad_app.quad import Quad, QuadOptions
from quad_app.log_rerun import LogRerun

class QuadApp:
    def __init__(self):
        logging.info("QuadApp // Initializing")
        self.quad = Quad(QuadOptions())
        self.log_rerun = LogRerun("quad_app")


    async def run(self):
        logging.info("QuadApp // Starting")
        await self.log_rerun.init()

        await self.quad.connect()
        
       # await self.log_rerun.smoketest_log()
        await self.quad.run()