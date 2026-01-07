from quad_app.app import QuadApp
import logging
import asyncio

async def main():
    logging.basicConfig(level=logging.INFO)
    logging.info("Starting SkyCanvas...")
    quad_app = QuadApp()
    await quad_app.run()


if __name__ == "__main__":
    asyncio.run(main())
