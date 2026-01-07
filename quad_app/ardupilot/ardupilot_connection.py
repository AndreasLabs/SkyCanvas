import logging
from mavsdk import System as MavSystem
class ArdupilotConnection:
    def __init__(self, connection_string: str):
        logging.info(f"QuadApp // Ardupilot // Initializing ArdupilotConnection with {connection_string}")
        self.connection_string = connection_string
        self.mav_system = None

    async def connect(self):
        logging.info(f"QuadApp // Ardupilot // Connecting to {self.connection_string}")
        self.mav_system = MavSystem()
        await self.mav_system.connect(system_address=self.connection_string)

    async def disconnect(self):
        logging.info(f"QuadApp // Ardupilot // Disconnecting from {self.connection_string}")
        pass

    async def wait_for_ready(self):
        logging.info("QuadApp // Ardupilot // Waiting for drone to be ready...")
        
        # Wait for connection
        logging.info("QuadApp // Ardupilot // Waiting for drone to connect...")
        async for state in self.mav_system.core.connection_state():
            if state.is_connected:
                logging.info("QuadApp // Ardupilot // Connected to drone!")
                break
        
        # Wait for global position
        logging.info("QuadApp // Ardupilot // Waiting for drone to have a global position estimate...")
        async for health in self.mav_system.telemetry.health():
            if health.is_global_position_ok and health.is_home_position_ok:
                logging.info("QuadApp // Ardupilot // Global position state is good enough for flying.")
                break
        
        # Fetch home altitude
        logging.info("QuadApp // Ardupilot // Fetching amsl altitude at home location....")
        async for terrain_info in self.mav_system.telemetry.home():
            absolute_altitude = terrain_info.absolute_altitude_m
            logging.info(f"QuadApp // Ardupilot // Home altitude: {absolute_altitude}m")
            self.home_altitude = absolute_altitude
            break
        
        logging.info("QuadApp // Ardupilot // Drone is ready!")

    async def arm(self):
        logging.info("QuadApp // Ardupilot // Arming")
        await self.mav_system.action.arm()

    async def takeoff(self):
        logging.info("QuadApp // Ardupilot // Taking off")
        await self.mav_system.action.takeoff()

    async def goto_location(self, latitude: float, longitude: float, altitude: float, yaw: float):
        logging.info(f"QuadApp // Ardupilot // Going to location: lat={latitude}, lon={longitude}, alt={altitude}, yaw={yaw}")
        await self.mav_system.action.goto_location(latitude, longitude, altitude, yaw)

