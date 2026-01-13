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
        
        # Get mission config if available
        mission_config = self.config.get('mission', {})
        
        self.quad = Quad(QuadOptions(self.config['quad']), mission_config)
    def load_lua_config(self, config_path: str):
        with open(config_path, 'r') as file:
            config_content = file.read()
            self.lua.execute(config_content)
            table = self.lua.globals()
            # Convert Lua table to Python dict recursively
            config_dir = self._lua_table_to_dict(table.config)
            return config_dir
    
    def _lua_table_to_dict(self, lua_table):
        """Recursively convert Lua tables to Python dicts/lists.
        
        Lua arrays like {1, 2, 3} have numeric keys starting at 1.
        These are converted to Python lists.
        Lua tables with string keys are converted to dicts.
        """
        # Check if it's an array-like table (consecutive numeric keys starting at 1)
        try:
            items = list(lua_table.items())
        except (AttributeError, TypeError):
            # Not a table, return as-is
            return lua_table
        
        if not items:
            return {}
        
        # Check if all keys are consecutive integers starting at 1 (Lua array)
        keys = [k for k, v in items]
        is_array = all(isinstance(k, int) for k in keys)
        if is_array:
            # Check if keys are 1, 2, 3, ... (Lua 1-indexed array)
            sorted_keys = sorted(keys)
            if sorted_keys == list(range(1, len(keys) + 1)):
                # Convert to Python list
                result = []
                for i in range(1, len(keys) + 1):
                    value = lua_table[i]
                    try:
                        result.append(self._lua_table_to_dict(value))
                    except (AttributeError, TypeError):
                        result.append(value)
                return result
        
        # Regular dict conversion
        result = {}
        for key, value in items:
            try:
                result[key] = self._lua_table_to_dict(value)
            except (AttributeError, TypeError):
                result[key] = value
        return result

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