config = {
    rerun = true,
}

config.quad = {
    connection_string = "tcpout://127.0.0.1:5760",
    telemetry_rate_hz = 50
}

config.depth = {
    -- Input (can override via CLI: python main.py [input])
    input = "../data/test_images/color_car1.jpg",
    
    -- Processing
    model = "DPT_Large",
    crop_size = 512,
    downsample_step = 10,
    
    -- Depth mapping (meters)
    depth_min = 0.5,
    depth_max = 0.75,
    flatten = nil,  -- Overrides depth range (e.g., 2.0 = 2m total depth)
    
    -- Depth cropping (removes points outside range)
    crop_min = 0.53,
    crop_max = 0.67,
    
    -- Output options
    save_depth = true,  -- Save depth visualization PNG
    view = true,        -- Open 3D viewer after export
}