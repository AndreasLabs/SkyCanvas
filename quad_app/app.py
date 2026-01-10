import logging
import asyncio
from quad_app.quad import Quad, QuadOptions

from quad_app.docker_manager import DockerManager


class QuadApp:
    def __init__(self, ensure_fresh_sitl: bool = True):
        logging.info("QuadApp // Initializing")
        self.quad = Quad(QuadOptions())
        self.docker_manager = DockerManager()
        self.ensure_fresh_sitl = ensure_fresh_sitl

    async def run(self):
        logging.info("QuadApp // Starting")

        # Ensure SITL container is running fresh
        if self.ensure_fresh_sitl:
            self.docker_manager.ensure_fresh()
            # Give SITL a moment to initialize after restart
            logging.info("QuadApp // Waiting for SITL to initialize...")
            await asyncio.sleep(2)


        await self.quad.connect()

        # await self.log_rerun.smoketest_log()
        await self.quad.run()