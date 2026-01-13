#!/usr/bin/env python3
"""Main CLI for depth capture and PLY export."""

import argparse
from pathlib import Path
from depth_capture.capture import DepthEstimator, capture_from_webcam, load_from_file, center_crop_and_resize
from depth_capture.export_ply import create_pointcloud_from_depth
from depth_capture.viewer import view_pointcloud


def parse_resolution(resolution_str: str) -> tuple[int, int]:
    """Parse resolution string in WIDTHxHEIGHT format.
    
    Args:
        resolution_str: Resolution string (e.g., "640x480")
        
    Returns:
        Tuple of (width, height)
    """
    try:
        width, height = map(int, resolution_str.lower().split('x'))
        return (width, height)
    except ValueError:
        raise argparse.ArgumentTypeError(
            f"Invalid resolution format: {resolution_str}. Expected WIDTHxHEIGHT (e.g., 640x480)"
        )


def get_default_output_path(input_path: str | None) -> str:
    """Generate default output path based on input file.
    
    Places output in a 'depth_out/' folder next to the input file.
    
    Args:
        input_path: Path to input image file, or None for webcam
        
    Returns:
        Default output PLY path
    """
    if input_path:
        input_file = Path(input_path)
        output_dir = input_file.parent / "depth_out"
        return str(output_dir / f"{input_file.stem}.ply")
    else:
        return "depth_out/webcam_capture.ply"


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Generate PLY pointcloud from monocular depth estimation"
    )
    parser.add_argument(
        "--input",
        type=str,
        default=None,
        help="Input image file (if not specified, uses webcam)"
    )
    parser.add_argument(
        "--output",
        type=str,
        default=None,
        help="Output PLY file path (default: depth_out/<input_name>.ply next to input)"
    )
    parser.add_argument(
        "--resolution",
        type=str,
        default="640x480",
        help="Webcam resolution in WIDTHxHEIGHT format (default: 640x480, only used for webcam)"
    )
    parser.add_argument(
        "--model",
        type=str,
        default="DPT_Large",
        choices=["DPT_Large", "DPT_Hybrid", "MiDaS_small"],
        help="MiDaS model type (default: DPT_Large)"
    )
    parser.add_argument(
        "--crop-size",
        type=int,
        default=512,
        help="Center crop and resize to square of this size (default: 512, 0 = no crop)"
    )
    parser.add_argument(
        "--max-points",
        type=int,
        default=None,
        help="Maximum number of points to export (default: no limit)"
    )
    parser.add_argument(
        "--downsample-step",
        type=int,
        default=None,
        help="Take every Nth point (alternative to --max-points)"
    )
    parser.add_argument(
        "--save-depth",
        action="store_true",
        help="Save depth visualization image for debugging"
    )
    parser.add_argument(
        "--depth-min",
        type=float,
        default=0.5,
        help="Minimum depth in meters for closest objects (default: 0.5)"
    )
    parser.add_argument(
        "--depth-max",
        type=float,
        default=10.0,
        help="Maximum depth in meters for furthest objects (default: 10.0)"
    )
    parser.add_argument(
        "--focal-length",
        type=float,
        default=None,
        help="Camera focal length in pixels (default: auto = image width for ~53° FOV)"
    )
    parser.add_argument(
        "--view",
        action="store_true",
        help="Open the result in 3D viewer after export"
    )
    parser.add_argument(
        "--crop-min",
        type=float,
        default=None,
        help="Crop points closer than this depth in meters (removes foreground)"
    )
    parser.add_argument(
        "--crop-max",
        type=float,
        default=None,
        help="Crop points farther than this depth in meters (removes background)"
    )
    parser.add_argument(
        "--flatten",
        type=float,
        default=None,
        help="Flatten depth to this range in meters (e.g., --flatten 1 = 1m total depth). Overrides depth-min/max."
    )
    
    args = parser.parse_args()
    
    # Handle flatten shortcut
    if args.flatten is not None:
        center = (args.depth_min + args.depth_max) / 2
        args.depth_min = center - args.flatten / 2
        args.depth_max = center + args.flatten / 2
    
    # Determine output path
    output_path = args.output if args.output else get_default_output_path(args.input)
    
    # Create output directory
    output_dir = Path(output_path).parent
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Initialize depth estimator
    print("=" * 60)
    print("Depth Capture - Monocular Depth to PLY Converter")
    print("=" * 60)
    
    estimator = DepthEstimator(model_type=args.model)
    
    # Load or capture image
    if args.input:
        print(f"\nLoading image from file: {args.input}")
        image = load_from_file(args.input)
    else:
        resolution = parse_resolution(args.resolution)
        print(f"\nCapturing from webcam at {resolution[0]}x{resolution[1]}...")
        image = capture_from_webcam(resolution)
    
    # Center crop and resize if requested
    if args.crop_size > 0:
        print(f"\nCropping to {args.crop_size}x{args.crop_size}...")
        image = center_crop_and_resize(image, args.crop_size)
    
    # Estimate depth
    print("\nEstimating depth...")
    depth_map = estimator.estimate_depth(image)
    print(f"Depth map computed: {depth_map.shape}")
    
    # Export to PLY
    print(f"\nExporting to PLY: {output_path}")
    create_pointcloud_from_depth(
        image,
        depth_map,
        output_path,
        focal_length=args.focal_length,
        depth_min=args.depth_min,
        depth_max=args.depth_max,
        max_points=args.max_points,
        downsample_step=args.downsample_step,
        save_depth=args.save_depth,
        crop_min=args.crop_min,
        crop_max=args.crop_max
    )
    
    print("\n" + "=" * 60)
    print("✓ Complete!")
    print("=" * 60)
    print(f"\nPLY file saved to: {Path(output_path).resolve()}")
    print("\nYou can now use this PLY file with quad_app patterns:")
    print("  from quad_app.patterns import generate_from_pointcloud, PointcloudConfig")
    print(f"  config = PointcloudConfig(")
    print(f"      center=(5.0, 0.0, -5.0),")
    print(f"      ply_path='{output_path}',")
    print(f"      density=0.2,")
    print(f"      depth_scale=2.0  # 0 = flat, >0 = 2.5D relief")
    print(f"  )")
    print(f"  path = generate_from_pointcloud(config)")
    
    # Open in viewer if requested
    if args.view:
        print("\nOpening in 3D viewer...")
        view_pointcloud(
            output_path,
            depth_min=args.depth_min,
            depth_max=args.depth_max
        )


if __name__ == "__main__":
    main()
