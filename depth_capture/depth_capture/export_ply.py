"""Export depth map and RGB image to PLY pointcloud."""

import cv2
import numpy as np
from plyfile import PlyData, PlyElement
from pathlib import Path


def depth_to_pointcloud(
    image: np.ndarray,
    depth_map: np.ndarray,
    depth_min: float = 0.5,
    depth_max: float = 10.0
) -> tuple[np.ndarray, np.ndarray]:
    """Convert depth map and RGB image to 3D pointcloud.
    
    Uses pinhole camera model with ~53° FOV.
    MiDaS outputs inverse relative depth (higher = closer).
    """
    height, width = depth_map.shape
    focal_length = width  # ~53° FOV
    
    u, v = np.meshgrid(np.arange(width), np.arange(height))
    
    # Normalize and map to metric depth
    depth_norm = (depth_map - depth_map.min()) / (depth_map.max() - depth_map.min() + 1e-8)
    depth = depth_min + (1.0 - depth_norm) * (depth_max - depth_min)
    
    # Back-project to 3D
    cx, cy = width / 2.0, height / 2.0
    x = (u - cx) * depth / focal_length
    y = -(v - cy) * depth / focal_length
    z = -depth
    
    points = np.stack([x.flatten(), y.flatten(), z.flatten()], axis=1)
    colors = image.reshape(-1, 3)[:, ::-1]  # BGR to RGB
    
    return points, colors


def filter_points(
    points: np.ndarray,
    colors: np.ndarray,
    crop_min: float | None = None,
    crop_max: float | None = None,
    step: int | None = None
) -> tuple[np.ndarray, np.ndarray]:
    """Filter and downsample points."""
    # Crop by depth
    if crop_min is not None or crop_max is not None:
        depth = -points[:, 2]
        mask = np.ones(len(points), dtype=bool)
        if crop_min:
            mask &= depth >= crop_min
        if crop_max:
            mask &= depth <= crop_max
        points, colors = points[mask], colors[mask]
        print(f"  Cropped to {len(points)} points")
    
    # Downsample
    if step and step > 1:
        indices = np.arange(0, len(points), step)
        points, colors = points[indices], colors[indices]
        print(f"  Downsampled to {len(points)} points")
    
    return points, colors


def export_to_ply(points: np.ndarray, colors: np.ndarray, output_path: str) -> None:
    """Export pointcloud to PLY file."""
    vertex = np.zeros(len(points), dtype=[
        ('x', 'f4'), ('y', 'f4'), ('z', 'f4'),
        ('red', 'u1'), ('green', 'u1'), ('blue', 'u1')
    ])
    vertex['x'], vertex['y'], vertex['z'] = points[:, 0], points[:, 1], points[:, 2]
    vertex['red'], vertex['green'], vertex['blue'] = colors[:, 0], colors[:, 1], colors[:, 2]
    
    PlyData([PlyElement.describe(vertex, 'vertex')]).write(str(output_path))
    print(f"  Exported {len(points)} points")


def create_pointcloud_from_depth(
    image: np.ndarray,
    depth_map: np.ndarray,
    output_path: str,
    depth_min: float = 0.5,
    depth_max: float = 10.0,
    downsample_step: int | None = None,
    save_depth: bool = False,
    crop_min: float | None = None,
    crop_max: float | None = None,
    **kwargs  # Ignore extra args
) -> None:
    """Pipeline: depth + RGB -> filtered PLY pointcloud."""
    if save_depth:
        depth_norm = (depth_map - depth_map.min()) / (depth_map.max() - depth_map.min() + 1e-8)
        depth_img = cv2.applyColorMap((depth_norm * 255).astype(np.uint8), cv2.COLORMAP_MAGMA)
        cv2.imwrite(str(Path(output_path).with_suffix('.depth.png')), depth_img)
    
    points, colors = depth_to_pointcloud(image, depth_map, depth_min, depth_max)
    points, colors = filter_points(points, colors, crop_min, crop_max, downsample_step)
    export_to_ply(points, colors.astype(np.uint8), output_path)
