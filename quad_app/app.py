import logging
import asyncio
from quad_app.quad import Quad, QuadOptions

from quad_app.docker_manager import DockerManager
from lupa.lua54 import LuaRuntime

class QuadApp:
    def __init__(self, ensure_fresh_sitl: bool = True):
        logging.info("QuadApp // Initializing")

        self.docker_manager = DockerManager()
        self.ensure_fresh_sitl = ensure_fresh_sitl
        self.lua = LuaRuntime(unpack_returned_tuples=True)
        self.config = self.load_lua_config("config.lua")
        self.quad = Quad(QuadOptions(self.config['quad']))
    def load_lua_config(self, config_path: str):
        with open(config_path, 'r') as file:
            config_content = file.read()
            self.lua.execute(config_content)
            table = self.lua.globals()
            # as dictionary
            config_dir = dict(table.config)
            return config_dir

    async def run(self):
        logging.info("QuadApp // Starting")
        logging.info(f"QuadApp // Config: {self.config}")

        # Ensure SITL container is running fresh
        if self.ensure_fresh_sitl:
            self.docker_manager.ensure_fresh()
            # Give SITL a moment to initialize after restart
            logging.info("QuadApp // Waiting for SITL to initialize...")
            await asyncio.sleep(5)


        await self.quad.connect()

        # await self.log_rerun.smoketest_log()
        await self.quad.run()