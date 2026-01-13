"""Export depth map and RGB image to PLY pointcloud."""

import cv2
import numpy as np
from plyfile import PlyData, PlyElement
from pathlib import Path


def save_depth_image(depth_map: np.ndarray, output_path: str) -> None:
    """Save depth map as a colorized debug image.
    
    Args:
        depth_map: Depth map (HxW, float32)
        output_path: Path to save the depth image
    """
    # Normalize to 0-255
    depth_normalized = (depth_map - depth_map.min()) / (depth_map.max() - depth_map.min() + 1e-8)
    depth_uint8 = (depth_normalized * 255).astype(np.uint8)
    
    # Apply colormap for better visualization
    depth_colored = cv2.applyColorMap(depth_uint8, cv2.COLORMAP_MAGMA)
    
    # Save
    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    cv2.imwrite(str(output_path), depth_colored)
    
    print(f"Saved depth visualization to {output_path}")


def downsample_points(
    points: np.ndarray,
    colors: np.ndarray,
    max_points: int | None = None,
    step: int | None = None
) -> tuple[np.ndarray, np.ndarray]:
    """Downsample pointcloud to reduce number of points.
    
    Args:
        points: Nx3 array of XYZ coordinates
        colors: Nx3 array of RGB values
        max_points: Maximum number of points to keep (uses uniform sampling)
        step: Take every Nth point (alternative to max_points)
        
    Returns:
        Downsampled (points, colors) tuple
    """
    n_points = len(points)
    
    if step is not None and step > 1:
        # Use step-based downsampling
        indices = np.arange(0, n_points, step)
        print(f"Downsampling with step={step}: {n_points} -> {len(indices)} points")
        return points[indices], colors[indices]
    
    if max_points is not None and max_points < n_points:
        # Use uniform sampling to get max_points
        indices = np.linspace(0, n_points - 1, max_points, dtype=int)
        print(f"Downsampling to max_points={max_points}: {n_points} -> {len(indices)} points")
        return points[indices], colors[indices]
    
    return points, colors


def depth_to_pointcloud(
    image: np.ndarray,
    depth_map: np.ndarray,
    focal_length: float | None = None,
    depth_min: float = 0.5,
    depth_max: float = 10.0
) -> tuple[np.ndarray, np.ndarray]:
    """Convert depth map and RGB image to 3D pointcloud.
    
    Args:
        image: RGB image (HxWx3, BGR, uint8)
        depth_map: Depth map (HxW, float32) - MiDaS outputs inverse relative depth
        focal_length: Camera focal length in pixels (None = auto based on image width)
        depth_min: Minimum depth in meters (closest objects)
        depth_max: Maximum depth in meters (furthest objects)
        
    Returns:
        Tuple of (points, colors) where:
            - points: Nx3 array of XYZ coordinates
            - colors: Nx3 array of RGB values (0-255, uint8)
    """
    height, width = depth_map.shape
    
    # Auto focal length: assume ~55° horizontal FOV (typical camera)
    if focal_length is None:
        focal_length = width * 1.0  # f ≈ width gives ~53° FOV
    
    # Create pixel coordinate grids
    u, v = np.meshgrid(np.arange(width), np.arange(height))
    
    # MiDaS outputs inverse relative depth (higher = closer)
    # Normalize to 0-1 range
    depth_normalized = (depth_map - depth_map.min()) / (depth_map.max() - depth_map.min() + 1e-8)
    
    # Map to metric depth range: high disparity (1) = close (depth_min), low (0) = far (depth_max)
    depth = depth_min + (1.0 - depth_normalized) * (depth_max - depth_min)
    
    # Back-project to 3D (pinhole camera model)
    cx = width / 2.0
    cy = height / 2.0
    
    x = (u - cx) * depth / focal_length
    y = -(v - cy) * depth / focal_length  # Negative Y so up is up
    z = -depth  # Negative Z so we view from the front
    
    # Stack into Nx3 point array
    points = np.stack([x.flatten(), y.flatten(), z.flatten()], axis=1)
    
    # Get RGB colors (convert BGR to RGB)
    colors = image.reshape(-1, 3)[:, ::-1]  # BGR to RGB
    
    return points, colors


def export_to_ply(
    points: np.ndarray,
    colors: np.ndarray,
    output_path: str
) -> None:
    """Export pointcloud to PLY file.
    
    Args:
        points: Nx3 array of XYZ coordinates
        colors: Nx3 array of RGB values (0-255, uint8)
        output_path: Path to output PLY file
    """
    # Ensure colors are uint8
    colors = colors.astype(np.uint8)
    
    # Create structured array for PLY
    vertex_data = np.zeros(
        len(points),
        dtype=[
            ('x', 'f4'),
            ('y', 'f4'),
            ('z', 'f4'),
            ('red', 'u1'),
            ('green', 'u1'),
            ('blue', 'u1')
        ]
    )
    
    vertex_data['x'] = points[:, 0]
    vertex_data['y'] = points[:, 1]
    vertex_data['z'] = points[:, 2]
    vertex_data['red'] = colors[:, 0]
    vertex_data['green'] = colors[:, 1]
    vertex_data['blue'] = colors[:, 2]
    
    # Create PLY element
    vertex_element = PlyElement.describe(vertex_data, 'vertex')
    
    # Write PLY file
    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    PlyData([vertex_element]).write(str(output_path))
    
    print(f"Exported {len(points)} points to {output_path}")


def create_pointcloud_from_depth(
    image: np.ndarray,
    depth_map: np.ndarray,
    output_path: str,
    focal_length: float | None = None,
    depth_min: float = 0.5,
    depth_max: float = 10.0,
    max_points: int | None = None,
    downsample_step: int | None = None,
    save_depth: bool = False
) -> None:
    """Complete pipeline: depth + RGB -> PLY pointcloud.
    
    Args:
        image: RGB image (HxWx3, BGR, uint8)
        depth_map: Depth map (HxW, float32)
        output_path: Path to output PLY file
        focal_length: Camera focal length in pixels (None = auto)
        depth_min: Minimum depth in meters (closest objects)
        depth_max: Maximum depth in meters (furthest objects)
        max_points: Maximum number of points to export (None = no limit)
        downsample_step: Take every Nth point (None = no step)
        save_depth: Save depth visualization image for debugging
    """
    # Optionally save depth visualization
    if save_depth:
        depth_img_path = Path(output_path).with_suffix('.depth.png')
        save_depth_image(depth_map, str(depth_img_path))
    
    print("Converting depth map to pointcloud...")
    print(f"  Depth range: {depth_min}m - {depth_max}m")
    print(f"  Focal length: {focal_length if focal_length else 'auto (image width)'}")
    points, colors = depth_to_pointcloud(image, depth_map, focal_length, depth_min, depth_max)
    
    print(f"Generated pointcloud with {len(points)} points")
    
    # Downsample if requested
    points, colors = downsample_points(points, colors, max_points, downsample_step)
    
    export_to_ply(points, colors, output_path)
