"""PLY point cloud viewer using Open3D."""

import numpy as np
import open3d as o3d
from pathlib import Path


def get_camera_params(view: str, center: np.ndarray, extent: float) -> dict:
    """Get camera parameters for preset views.
    
    Point cloud coordinate system:
    - X: horizontal (left = -X, right = +X)
    - Y: vertical (up = -Y, down = +Y) 
    - Z: depth (near = small -Z, far = large -Z)
    
    Args:
        view: View name ("front", "top", "side", "iso")
        center: Point cloud center
        extent: Point cloud extent (max dimension)
        
    Returns:
        Dict with 'eye', 'lookat', 'up' vectors
    """
    dist = extent * 2.0  # Camera distance
    
    views = {
        "front": {  # Looking at front face
            "eye": center + [0, 0, dist],
            "lookat": center,
            "up": [0, -1, 0],
        },
        "top": {  # Looking down from above
            "eye": center + [0, -dist, -dist * 0.3],
            "lookat": center,
            "up": [0, 0, -1],
        },
        "side": {  # Looking from right side
            "eye": center + [dist, 0, -dist * 0.3],
            "lookat": center,
            "up": [0, -1, 0],
        },
        "iso": {  # Front view (same as front for now)
            "eye": center + [0, 0, dist],
            "lookat": center,
            "up": [0, 1, 0],  # Flipped vertical
        },
    }
    
    return views.get(view, views["iso"])


def render_pointcloud(
    ply_path: str,
    output_path: str | None = None,
    view: str = "iso",
    width: int = 1024,
    height: int = 768,
    point_size: float = 4.0,
    background: tuple[float, float, float] = (0.05, 0.05, 0.05),
    show_axes: bool = False,
) -> str:
    """Render point cloud to image from specified view.
    
    Args:
        ply_path: Path to PLY file
        output_path: Output image path (default: {ply_stem}.{view}.png)
        view: Camera view ("front", "top", "side", "iso")
        width: Image width
        height: Image height
        point_size: Point size for rendering
        background: Background RGB color (0-1)
        show_axes: Show coordinate axes
        
    Returns:
        Path to saved image
    """
    path = Path(ply_path)
    if not path.exists():
        raise FileNotFoundError(f"PLY not found: {path}")
    
    # Default output path
    if output_path is None:
        output_path = str(path.with_suffix(f".{view}.png"))
    
    # Load point cloud
    pcd = o3d.io.read_point_cloud(str(path))
    points = np.asarray(pcd.points)
    
    if len(points) == 0:
        raise ValueError("Point cloud is empty")
    
    # Compute bounds
    center = points.mean(axis=0)
    extent = np.linalg.norm(points.max(axis=0) - points.min(axis=0))
    
    # Create visualizer
    vis = o3d.visualization.Visualizer()
    vis.create_window(width=width, height=height, visible=False)
    
    # Add geometries
    vis.add_geometry(pcd)
    
    if show_axes:
        axis_scale = extent * 0.15
        axes = o3d.geometry.TriangleMesh.create_coordinate_frame(size=axis_scale, origin=center)
        vis.add_geometry(axes)
    
    # Set render options
    opt = vis.get_render_option()
    opt.background_color = np.array(background)
    opt.point_size = point_size
    
    # Set camera view
    cam = get_camera_params(view, center, extent)
    ctr = vis.get_view_control()
    ctr.set_lookat(cam["lookat"])
    ctr.set_front(cam["eye"] - cam["lookat"])
    ctr.set_up(cam["up"])
    ctr.set_zoom(0.5)
    
    # Render and save
    vis.poll_events()
    vis.update_renderer()
    vis.capture_screen_image(output_path, do_render=True)
    vis.destroy_window()
    
    print(f"  Saved: {Path(output_path).name}")
    return output_path


def view_pointcloud(ply_path: str, depth_min: float = 0.5, depth_max: float = 10.0) -> None:
    """View a PLY point cloud with scale reference axes (interactive)."""
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
