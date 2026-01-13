#!/usr/bin/env python3
"""Depth capture - generates PLY pointcloud from monocular depth estimation."""

import sys
from pathlib import Path
from lupa.lua54 import LuaRuntime

from depth_capture.capture import DepthEstimator, load_from_file, center_crop_and_resize
from depth_capture.export_ply import create_pointcloud_from_depth
from depth_capture.viewer import view_pointcloud


def load_config(config_path: str = "config.lua") -> dict:
    """Load depth config from Lua file."""
    lua = LuaRuntime(unpack_returned_tuples=True)
    with open(config_path, 'r') as f:
        lua.execute(f.read())
    
    t = lua.globals().config.depth
    return {k: t[k] for k in t}


def get_output_path(input_path: str) -> str:
    """Generate output path: input_dir/depth_out/name.ply"""
    p = Path(input_path)
    return str(p.parent / "depth_out" / f"{p.stem}.ply")


def main():
    # Load config
    config = load_config(Path(__file__).parent.parent / "config.lua")
    
    # CLI override: [input]
    if len(sys.argv) > 1 and not sys.argv[1].startswith('-'):
        config['input'] = sys.argv[1]
    
    if not config.get('input'):
        print("Usage: python main.py <input.jpg>")
        print("Or set config.depth.input in config.lua")
        sys.exit(1)
    
    # Handle flatten -> depth_min/max
    if config.get('flatten'):
        center = (config['depth_min'] + config['depth_max']) / 2
        config['depth_min'] = center - config['flatten'] / 2
        config['depth_max'] = center + config['flatten'] / 2

    output = get_output_path(config['input'])
    Path(output).parent.mkdir(parents=True, exist_ok=True)

    print("=" * 50)
    print("Depth Capture")
    print("=" * 50)
    print(f"Input:  {config['input']}")
    print(f"Output: {output}")
    print(f"Depth:  {config['depth_min']:.1f}m - {config['depth_max']:.1f}m")
    if config.get('crop_min') or config.get('crop_max'):
        print(f"Crop:   {config.get('crop_min') or 0}m - {config.get('crop_max') or '∞'}m")

    # Process
    image = load_from_file(config['input'])
    if config.get('crop_size'):
        image = center_crop_and_resize(image, config['crop_size'])

    estimator = DepthEstimator(model_type=config['model'])
    depth_map = estimator.estimate_depth(image)

    create_pointcloud_from_depth(
        image, depth_map, output,
        depth_min=config['depth_min'],
        depth_max=config['depth_max'],
        downsample_step=config.get('downsample_step'),
        save_depth=config.get('save_depth'),
        crop_min=config.get('crop_min'),
        crop_max=config.get('crop_max'),
    )

    print(f"\n✓ Saved: {Path(output).resolve()}")

    if config.get('view'):
        view_pointcloud(output, config['depth_min'], config['depth_max'])


if __name__ == "__main__":
    main()
