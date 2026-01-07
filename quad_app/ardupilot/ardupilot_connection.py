import logging

class ArdupilotConnection:
    def __init__(self, connection_string: str):
        logging.info(f"QuadApp // Ardupilot // Initializing ArdupilotConnection with {connection_string}")
        self.connection_string = connection_string

    def connect(self):
        logging.info(f"QuadApp // Ardupilot // Connecting to {self.connection_string}")
        pass

    def disconnect(self):
        logging.info(f"QuadApp // Ardupilot // Disconnecting from {self.connection_string}")
        pass

    def arm(self):
        logging.info(f"QuadApp // Ardupilot // Arming {self.connection_string}")
        pass
