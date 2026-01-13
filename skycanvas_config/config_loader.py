"""Configuration loader with singleton pattern and Lua support."""

import logging
from pathlib import Path
from typing import Any, Optional
from lupa.lua54 import LuaRuntime


class ConfigSingleton:
    """Singleton config loader with dotted key access and default value support.
    
    Usage:
        Config.load("config.lua")
        value = Config['mission.ply_path']
        density = Config['mission.density', 0.1]  # with default
        config_dict = Config.get('mission', {})
    """
    
    _instance = None
    _config: Optional[dict] = None
    _loaded_path: Optional[Path] = None
    
    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance
    
    def load(self, config_path: str | Path) -> None:
        """Load Lua configuration file.
        
        Args:
            config_path: Path to config.lua file
            
        Note:
            If already loaded from same path, does nothing (idempotent).
            If loaded from different path, reloads configuration.
        """
        config_path = Path(config_path)
        
        # If already loaded from same path, skip
        if self._loaded_path == config_path and self._config is not None:
            logging.debug(f"Config already loaded from {config_path}")
            return
        
        if not config_path.exists():
            raise FileNotFoundError(f"Config file not found: {config_path}")
        
        logging.info(f"Loading config from {config_path}")
        
        # Create Lua runtime and execute config
        lua = LuaRuntime(unpack_returned_tuples=True)
        with open(config_path, 'r') as f:
            lua.execute(f.read())
        
        # Extract config table
        lua_config = lua.globals().config
        self._config = self._lua_table_to_dict(lua_config)
        self._loaded_path = config_path
        
        logging.debug(f"Config loaded: {list(self._config.keys())}")
    
    def __getitem__(self, key_or_tuple) -> Any:
        """Get config value with dotted key notation.
        
        Args:
            key_or_tuple: Either a string key or tuple (key, default)
            
        Returns:
            Config value or default if provided
            
        Examples:
            Config['mission.ply_path']
            Config['mission.density', 0.1]
        """
        if self._config is None:
            raise RuntimeError("Config not loaded. Call Config.load(path) first.")
        
        # Handle tuple syntax: Config['key', default]
        if isinstance(key_or_tuple, tuple):
            if len(key_or_tuple) == 2:
                key, default = key_or_tuple
                return self._get_nested(key, default)
            else:
                raise ValueError("Tuple must be (key, default)")
        
        # Handle simple key
        return self._get_nested(key_or_tuple)
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get config value with optional default (dict-like interface).
        
        Args:
            key: Dotted key path (e.g., 'mission.ply_path')
            default: Default value if key not found
            
        Returns:
            Config value or default
        """
        if self._config is None:
            raise RuntimeError("Config not loaded. Call Config.load(path) first.")
        
        return self._get_nested(key, default)
    
    def _get_nested(self, key: str, default: Any = None) -> Any:
        """Get nested config value using dotted notation.
        
        Args:
            key: Dotted key path (e.g., 'mission.ply_path')
            default: Default value if key not found
            
        Returns:
            Config value or default
        """
        keys = key.split('.')
        value = self._config
        
        for k in keys:
            if isinstance(value, dict) and k in value:
                value = value[k]
            else:
                return default
        
        return value
    
    def _lua_table_to_dict(self, lua_table):
        """Recursively convert Lua tables to Python dicts/lists.
        
        Lua arrays like {1, 2, 3} have numeric keys starting at 1.
        These are converted to Python lists.
        Lua tables with string keys are converted to dicts.
        
        Args:
            lua_table: Lua table object
            
        Returns:
            Python dict or list
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
    
    def reset(self) -> None:
        """Reset config (mainly for testing)."""
        self._config = None
        self._loaded_path = None


# Create singleton instance
Config = ConfigSingleton()
