"""PLY point cloud viewer using Open3D."""

import numpy as np
import open3d as o3d
from pathlib import Path


def view_pointcloud(ply_path: str, depth_min: float = 0.5, depth_max: float = 10.0) -> None:
    """View a PLY point cloud with scale reference axes."""
    path = Path(ply_path)
    if not path.exists():
        raise FileNotFoundError(f"PLY not found: {path}")
    
    pcd = o3d.io.read_point_cloud(str(path))
    print(f"Loaded {len(pcd.points)} points from {path.name}")
    
    # Print actual depth range
    points = np.asarray(pcd.points)
    if len(points):
        print(f"Depth range: {-points[:, 2].max():.2f}m - {-points[:, 2].min():.2f}m")
    
    # Add coordinate axes scaled to depth range
    axis_scale = max(0.2, (depth_max - depth_min) * 0.15)
    axes = o3d.geometry.TriangleMesh.create_coordinate_frame(size=axis_scale)
    
    print("\nControls: drag=rotate, scroll=zoom, shift+drag=pan, Q=quit")
    o3d.visualization.draw_geometries([pcd, axes], window_name=path.name, width=1024, height=768)


if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1:
        view_pointcloud(sys.argv[1])
    else:
        print("Usage: python -m depth_capture.viewer <file.ply>")
