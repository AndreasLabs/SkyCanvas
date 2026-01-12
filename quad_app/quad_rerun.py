import rerun as rr
import numpy as np
import logging
from quad_app.context import QuadContext
class QuadRerun:
    def __init__(self, name: str, context: QuadContext):
        self.name = name
        self.context = context
        self.initialized = False
    async def init(self):
        logging.info(f"QuadRerun // Initializing {self.name}")
        rr.init(self.name, spawn=True)
        self.initialized = True

    async def smoketest_log(self):
      
        if not self.initialized:
            logging.warning(f"QuadRerun // Not initialized, initializing {self.name}")
            await self.init()
        SIZE = 10

        pos_grid = np.meshgrid(*[np.linspace(-10, 10, SIZE)]*3)
        positions = np.vstack([d.reshape(-1) for d in pos_grid]).T

        col_grid = np.meshgrid(*[np.linspace(0, 255, SIZE)]*3)
        colors = np.vstack([c.reshape(-1) for c in col_grid]).astype(np.uint8).T

        rr.log(
            "smoketest/points3d",
            rr.Points3D(positions, colors=colors, radii=0.5)
        )
        
    async def log_tick(self, quad):
        self.log_time_now()
        pass

    async def log_position_geo(self):
        pass