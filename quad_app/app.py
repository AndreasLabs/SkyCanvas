import logging
import asyncio
from quad_app.quad import Quad, QuadOptions

from quad_app.docker_manager import DockerManager
from skycanvas_config import Config

class QuadApp:
    def __init__(self, ensure_fresh_sitl: bool = True):
        logging.info("QuadApp // Initializing")

        self.docker_manager = DockerManager()
        self.ensure_fresh_sitl = ensure_fresh_sitl
        
        # Load config once (singleton handles duplicates)
        Config.load("config.lua")
        
        # Get mission config if available
        mission_config = Config.get('mission', {})
        
        self.quad = Quad(QuadOptions(Config['quad']), mission_config)
    async def run(self):
        logging.info("QuadApp // Starting")
        logging.info(f"QuadApp // Config: rerun={Config['rerun']}, mission={Config['mission.name']}")

        # Ensure SITL container is running fresh
        if self.ensure_fresh_sitl:
            self.docker_manager.ensure_fresh()
            # Give SITL a moment to initialize after restart
            logging.info("QuadApp // Waiting for SITL to initialize...")
            await asyncio.sleep(5)


        await self.quad.connect()

        # await self.log_rerun.smoketest_log()
        await self.quad.run()