# SkyCanvas Missions

This directory contains mission modules for the SkyCanvas quad system. Missions encapsulate complete flight sequences including takeoff, waypoint execution, and landing.

## Available Missions

### 1. Smiley Mission (`smiley`)
Flies a smiley face pattern in the air using the `generate_smiley` pattern.

**Config:** No special configuration required.

### 2. Pointcloud Mission (`pointcloud`)
Flies a 3D pattern loaded from a PLY pointcloud file with colored waypoints.

**Config:**
```lua
config.mission = {
    name = "pointcloud",
    ply_path = "data/test_images/depth_out/color_car1.ply",
    center = {5.0, 0.0, -5.0},  -- NED coordinates
    scale = 3.0,                 -- Pattern size in meters
    density = 0.2,               -- Min spacing between waypoints
    depth_scale = 1.0,           -- 0=flat, >0=2.5D relief
    hold_time = 0.3,             -- LED hold time per waypoint
}
```

## How to Use

### Selecting a Mission

Edit `config.lua` and set the mission name:

```lua
config.mission = {
    name = "pointcloud",  -- or "smiley"
    -- ... mission-specific config ...
}
```

Then run:
```bash
python main.py
```

### Creating a New Mission

1. Create a new file in `quad_app/missions/` (e.g., `my_mission.py`)
2. Implement the `Mission` base class:

```python
from quad_app.missions.base import Mission
from quad_app.context import QuadContext
from quad_app.waypoints import WaypointSystem

class MyMission(Mission):
    name = "my_mission"
    
    def __init__(self, config: dict = None):
        self.config = config or {}
    
    async def run(self, context: QuadContext, waypoints: WaypointSystem):
        # Takeoff
        await context.mav_system.action.takeoff()
        
        # Generate and run waypoints
        # path = generate_my_pattern(...)
        # await waypoints.run_path(path)
        # await waypoints.wait_until_disabled()
        
        # Land
        await context.mav_system.action.land()
        await context.mav_system.action.disarm()
```

3. Register it in `quad_app/missions/__init__.py`:

```python
from quad_app.missions.my_mission import MyMission

MISSIONS = {
    "smiley": SmileyMission,
    "pointcloud": PointcloudMission,
    "my_mission": MyMission,  # Add here
}
```

4. Add config to `config.lua`:

```lua
config.mission = {
    name = "my_mission",
    -- ... your config ...
}
```

## Mission Architecture

```
QuadApp (app.py)
  └─> Quad (quad.py)
       ├─> Mission (loaded by name)
       │    └─> run(context, waypoints)
       │         ├─> Uses patterns/ for waypoint generation
       │         └─> Uses waypoints system for execution
       └─> Context (led_system, mav_system, etc.)
```

## Available Patterns

Missions can use any pattern from `quad_app/patterns/`:
- `generate_smiley(config)` - Smiley face
- `generate_square(config)` - Square pattern
- `generate_from_pointcloud(config)` - PLY pointcloud loader
