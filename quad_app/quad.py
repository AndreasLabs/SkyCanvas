import logging
from quad_app.ardupilot.ardupilot_connection import ArdupilotConnection


class QuadOptions:
    def __init__(self):
        logging.info("QuadOptions // Initializing QuadOptions")
        self.connection_string = "tcp:127.0.0.1:5760"

    def set_connection_string(self, connection_string: str):
        logging.info(f"QuadOptions // Setting connection string to {connection_string}")
        self.connection_string = connection_string


class Quad:
    def __init__(self, options: QuadOptions):
        logging.info("Quad // Initializing Quad")
        self.options = options
        self.connection = ArdupilotConnection(options.connection_string)

    def init(self):
        logging.info("Quad // Initializing Quad")
        self.connection.connect()
    
    def run(self):
        logging.info("Quad // Running Quad")
        pass