from quad_app.app import QuadApp
import logging
def main():
    logging.basicConfig(level=logging.INFO)
    logging.info("Starting SkyCanvas...")
    quad_app = QuadApp()
    quad_app.run()


if __name__ == "__main__":
    main()
