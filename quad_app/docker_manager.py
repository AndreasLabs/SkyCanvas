import logging
from pathlib import Path
from python_on_whales import DockerClient

# Path to compose file relative to project root
COMPOSE_FILE = Path(__file__).parent.parent / "docker" / "compose.sil.yml"


class DockerManager:
    """Manages Docker Compose services for the quad application."""

    def __init__(self, compose_file: Path = COMPOSE_FILE):
        self.compose_file = compose_file
        self.docker = DockerClient(compose_files=[self.compose_file])

    def ensure_fresh(self, service: str = "ardupilot-sitl", timeout_seconds: int = 30):
        """
        Ensure a fresh instance of the service is running.
        - If running: restart it
        - If not running: start it
        """
        logging.info(f"DockerManager // Ensuring fresh instance of '{service}'")

        containers = self.docker.compose.ps(services=[service])

        if containers:
            # Service is running - restart it for a fresh state
            logging.info(f"DockerManager // Service '{service}' is running, restarting...")
            self.docker.compose.restart(services=[service], timeout=timeout_seconds)
            logging.info(f"DockerManager // Service '{service}' restarted")
        else:
            # Service not running - start it
            logging.info(f"DockerManager // Service '{service}' not running, starting...")
            self.docker.compose.up(services=[service], detach=True)
            logging.info(f"DockerManager // Service '{service}' started")

    def stop(self, service: str = "ardupilot-sitl", timeout_seconds: int = 30):
        """Stop the specified service."""
        logging.info(f"DockerManager // Stopping service '{service}'")
        self.docker.compose.stop(services=[service], timeout=timeout_seconds)
        logging.info(f"DockerManager // Service '{service}' stopped")

    def down(self, timeout_seconds: int = 30):
        """Stop and remove all compose services."""
        logging.info("DockerManager // Bringing down all services")
        self.docker.compose.down(timeout=timeout_seconds)
        logging.info("DockerManager // All services down")

    def is_running(self, service: str = "ardupilot-sitl") -> bool:
        """Check if the service is currently running."""
        containers = self.docker.compose.ps(services=[service])
        return len(containers) > 0 and all(c.state.running for c in containers)

    def logs(self, service: str = "ardupilot-sitl", tail: int = 50) -> str:
        """Get recent logs from the service."""
        return self.docker.compose.logs(services=[service], tail=str(tail))


