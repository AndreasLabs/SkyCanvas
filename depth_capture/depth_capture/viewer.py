"""Simple PLY point cloud viewer using Open3D."""

import argparse
import open3d as o3d
from pathlib import Path


def view_pointcloud(ply_path: str) -> None:
    """View a PLY point cloud file.
    
    Args:
        ply_path: Path to PLY file
    """
    path = Path(ply_path)
    if not path.exists():
        raise FileNotFoundError(f"PLY file not found: {path}")
    
    print(f"Loading {path}...")
    pcd = o3d.io.read_point_cloud(str(path))
    
    print(f"Point cloud has {len(pcd.points)} points")
    print(f"Has colors: {pcd.has_colors()}")
    print(f"Bounding box: {pcd.get_axis_aligned_bounding_box()}")
    
    # Create visualizer
    print("\nControls:")
    print("  Mouse drag: Rotate")
    print("  Scroll: Zoom")
    print("  Shift+drag: Pan")
    print("  Q/Esc: Quit")
    print("  R: Reset view")
    print("  +/-: Point size")
    
    o3d.visualization.draw_geometries(
        [pcd],
        window_name=f"Point Cloud: {path.name}",
        width=1024,
        height=768,
        point_show_normal=False
    )


def main():
    """CLI entry point."""
    parser = argparse.ArgumentParser(description="View PLY point cloud files")
    parser.add_argument("ply_file", type=str, help="Path to PLY file")
    args = parser.parse_args()
    
    view_pointcloud(args.ply_file)


if __name__ == "__main__":
    main()
