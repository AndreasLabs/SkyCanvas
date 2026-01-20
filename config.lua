config = {
    rerun = true,
}

config.quad = {
    connection_string = "tcpout://127.0.0.1:5760",
    telemetry_rate_hz = 20
}

config.mission = {
    name = "spiral",  -- Options: "smiley", "pointcloud", "spiral"
    
    -- Spiral mission settings
    center = {0.0, 0.0, -10.0},  -- NED: north, east, down (down is negative for altitude)
    scale = 1.0,                 -- Pattern size in meters
    hold_time = 0.1,             -- Time to hold LED at each waypoint (seconds)
    spiral_turns = 3,            -- Number of complete rotations
    spiral_points = 16,          -- Points per turn (total = turns × points)
    
    -- Smiley mission settings (commented out)
    -- (uses center, scale, hold_time from above)
    
    -- Pointcloud mission settings (commented out)
    -- ply_path = "data/test_images/depth_out/color_zipline1.ply",
    -- density = 0.01,               -- Minimum distance between waypoints (meters)
    -- depth_scale = 0,           -- Depth range: 0 = flat 2D, >0 = 2.5D relief effect
}

config.depth = {
    -- Input (can override via CLI: python main.py [input])
    -- input = "../data/test_images/color_car1.jpg",
    input = "../data/test_images/color_zipline1.jpeg",
    -- Processing
    model = "DPT_Large",
    crop_size = 512,
    downsample_step = 200,
    
    -- Segmentation (YOLOE - open-vocabulary)
    segment = nil,  -- Text prompt for object selection (e.g., "car", "person", "dog")
                      -- Set to nil to disable segmentation
    
    -- Depth mapping (meters)
    -- depth_min = 0,
    -- depth_max = 1.0,
    flatten = 0.1,  -- Overrides depth range (e.g., 2.0 = 2m total depth)
    
    -- Depth cropping (removes points outside range)
    -- crop_min = 0.0,
    --crop_max = 0,
    
    -- Debug output (saved to depth_out/)
    save_depth = true,    -- Colorized depth map (.depth.png)
    save_mask = true,     -- Binary segmentation mask (.mask.png)
    save_masked = true,   -- Image with background removed (.masked.png)
    save_overlay = true,  -- Mask overlay on original (.overlay.png)
    save_render = true,   -- 3D render from isometric angle (.iso.png)
    
    -- Viewer
    view = true,         -- Open interactive 3D viewer after export
}