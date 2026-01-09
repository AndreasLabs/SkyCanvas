import logging
from mavsdk import System as MavSystem


class QuadOptions:
    def __init__(self):
        self.connection_string = "tcpout://127.0.0.1:5760"

    def set_connection_string(self, connection_string: str):
        self.connection_string = connection_string


class Quad:
    def __init__(self, options: QuadOptions):
        logging.info("Quad // Initializing")
        self.options = options
        self.mav_system = None
        self.home_altitude = None

    async def connect(self):
        """Connect to the MAVLink system"""
        logging.info(f"Quad // Connecting to {self.options.connection_string}")
        self.mav_system = MavSystem()
        await self.mav_system.connect(system_address=self.options.connection_string)

    async def wait_for_ready(self):
        """Wait for drone to be ready for flight"""
        logging.info("Quad // Waiting for drone to be ready")
        
        # Wait for connection
        async for state in self.mav_system.core.connection_state():
            if state.is_connected:
                logging.info("Quad // Connected to drone")
                break
        
        # Wait for global position
        logging.info("Quad // Waiting for global position estimate")
        async for health in self.mav_system.telemetry.health():
            if health.is_global_position_ok and health.is_home_position_ok:
                logging.info("Quad // Global position OK")
                break
        
        # Fetch home altitude
        async for terrain_info in self.mav_system.telemetry.home():
            self.home_altitude = terrain_info.absolute_altitude_m
            logging.info(f"Quad // Home altitude: {self.home_altitude}m")
            break
        
        logging.info("Quad // Ready for flight")

    async def arm(self):
        """Arm the drone"""
        logging.info("Quad // Arming")
        await self.mav_system.action.arm()

    async def takeoff(self):
        """Take off to default altitude"""
        logging.info("Quad // Taking off")
        await self.mav_system.action.takeoff()

    async def goto_location(self, latitude: float, longitude: float, altitude: float, yaw: float):
        """Fly to specified location"""
        logging.info(f"Quad // Going to lat={latitude}, lon={longitude}, alt={altitude}m, yaw={yaw}°")
        await self.mav_system.action.goto_location(latitude, longitude, altitude, yaw)

    async def land(self):
        """Land the drone"""
        logging.info("Quad // Landing")
        await self.mav_system.action.land()

    async def disarm(self):
        """Disarm the drone"""
        logging.info("Quad // Disarming")
        await self.mav_system.action.disarm()

    async def run(self):
        """Execute a simple test flight"""
        logging.info("Quad // Running test flight")
        await self.wait_for_ready()
        await self.arm()
        await self.takeoff()
        await self.goto_location(0, 0, 10, 0)
        await self.land()
        await self.disarm()