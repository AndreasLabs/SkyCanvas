"""Simple PLY point cloud viewer using Open3D."""

import argparse
import numpy as np
import open3d as o3d
from pathlib import Path


def view_pointcloud(
    ply_path: str,
    depth_min: float = 0.5,
    depth_max: float = 10.0,
    show_axes: bool = True
) -> None:
    """View a PLY point cloud file.
    
    Args:
        ply_path: Path to PLY file
        depth_min: Minimum depth used during export (for axes scaling)
        depth_max: Maximum depth used during export (for axes scaling)
        show_axes: Show coordinate axes for scale reference
    """
    path = Path(ply_path)
    if not path.exists():
        raise FileNotFoundError(f"PLY file not found: {path}")
    
    print(f"Loading {path}...")
    pcd = o3d.io.read_point_cloud(str(path))
    
    print(f"Point cloud has {len(pcd.points)} points")
    print(f"Has colors: {pcd.has_colors()}")
    print(f"Bounding box: {pcd.get_axis_aligned_bounding_box()}")
    
    # Print actual depth range for reference
    points = np.asarray(pcd.points)
    if len(points) > 0:
        actual_depth_min = -points[:, 2].max()  # Z is negative
        actual_depth_max = -points[:, 2].min()
        print(f"Actual depth range: {actual_depth_min:.2f}m - {actual_depth_max:.2f}m")
    
    # Build geometry list
    geometries = [pcd]
    
    # Add coordinate frame scaled to depth range for reference
    if show_axes:
        depth_range = depth_max - depth_min
        axis_scale = max(0.2, depth_range * 0.15)  # 15% of depth range
        coord_frame = o3d.geometry.TriangleMesh.create_coordinate_frame(
            size=axis_scale, origin=[0, 0, 0]
        )
        geometries.append(coord_frame)
        print(f"Axes scale: {axis_scale:.2f}m (Red=X, Green=Y, Blue=Z)")
    
    # Create visualizer
    print("\nControls:")
    print("  Mouse drag: Rotate")
    print("  Scroll: Zoom")
    print("  Shift+drag: Pan")
    print("  Q/Esc: Quit")
    print("  R: Reset view")
    print("  +/-: Point size")
    
    # Use Visualizer for fixed-scale view (not auto-fit)
    vis = o3d.visualization.Visualizer()
    vis.create_window(
        window_name=f"Point Cloud: {path.name}",
        width=1024,
        height=768
    )
    
    for geom in geometries:
        vis.add_geometry(geom)
    
    # Set view based on actual depth range
    ctr = vis.get_view_control()
    center_depth = (depth_min + depth_max) / 2
    
    # Set camera to look at center of depth range
    ctr.set_front([0, 0, 1])
    ctr.set_lookat([0, 0, -center_depth])
    ctr.set_up([0, 1, 0])
    
    # Fixed zoom: larger depth range = more zoomed out
    # This makes 1 meter appear the same size regardless of depth range
    ctr.set_zoom(0.3 / (depth_max - depth_min + 0.1))
    
    vis.run()
    vis.destroy_window()


def main():
    """CLI entry point."""
    parser = argparse.ArgumentParser(description="View PLY point cloud files")
    parser.add_argument("ply_file", type=str, help="Path to PLY file")
    parser.add_argument(
        "--depth-min",
        type=float,
        default=0.5,
        help="Minimum depth in meters for axes scale reference (default: 0.5)"
    )
    parser.add_argument(
        "--depth-max",
        type=float,
        default=10.0,
        help="Maximum depth in meters for axes scale reference (default: 10.0)"
    )
    parser.add_argument(
        "--no-axes",
        action="store_true",
        help="Hide coordinate axes"
    )
    args = parser.parse_args()
    
    view_pointcloud(
        args.ply_file,
        depth_min=args.depth_min,
        depth_max=args.depth_max,
        show_axes=not args.no_axes
    )


if __name__ == "__main__":
    main()
