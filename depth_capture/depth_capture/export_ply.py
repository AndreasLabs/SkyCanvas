"""Export depth map and RGB image to PLY pointcloud."""

import cv2
import numpy as np
from plyfile import PlyData, PlyElement
from pathlib import Path


def save_debug_images(
    output_path: str,
    image: np.ndarray,
    depth_map: np.ndarray,
    mask: np.ndarray | None = None,
    save_depth: bool = True,
    save_mask: bool = True,
    save_masked: bool = True,
    save_overlay: bool = True,
) -> None:
    """Save debug visualization images.
    
    Args:
        output_path: Base output path (will add suffixes)
        image: Original BGR image
        depth_map: Depth map array
        mask: Binary segmentation mask (optional)
        save_depth: Save colorized depth map
        save_mask: Save binary mask visualization
        save_masked: Save image with mask applied (background removed)
        save_overlay: Save mask overlay on original image
    """
    base = Path(output_path).with_suffix('')
    
    # Depth visualization (magma colormap)
    if save_depth:
        depth_norm = (depth_map - depth_map.min()) / (depth_map.max() - depth_map.min() + 1e-8)
        depth_img = cv2.applyColorMap((depth_norm * 255).astype(np.uint8), cv2.COLORMAP_MAGMA)
        cv2.imwrite(str(base) + '.depth.png', depth_img)
        print(f"  Saved: {base.name}.depth.png")
    
    if mask is not None:
        # Binary mask visualization (white on black)
        if save_mask:
            mask_img = (mask.astype(np.uint8) * 255)
            cv2.imwrite(str(base) + '.mask.png', mask_img)
            print(f"  Saved: {base.name}.mask.png")
        
        # Masked image (background removed / transparent)
        if save_masked:
            masked_img = image.copy()
            masked_img[~mask] = 0  # Black background
            cv2.imwrite(str(base) + '.masked.png', masked_img)
            print(f"  Saved: {base.name}.masked.png")
        
        # Overlay visualization (mask as colored overlay)
        if save_overlay:
            overlay = image.copy()
            # Green tint for masked region
            overlay[mask] = cv2.addWeighted(
                overlay[mask], 0.7,
                np.full_like(overlay[mask], [0, 255, 0]), 0.3,
                0
            )
            # Dim the background
            overlay[~mask] = (overlay[~mask] * 0.3).astype(np.uint8)
            cv2.imwrite(str(base) + '.overlay.png', overlay)
            print(f"  Saved: {base.name}.overlay.png")


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
    step: int | None = None,
    mask: np.ndarray | None = None
) -> tuple[np.ndarray, np.ndarray]:
    """Filter and downsample points.
    
    Args:
        points: Point cloud array (Nx3)
        colors: Color array (Nx3)
        crop_min: Minimum depth to keep (meters)
        crop_max: Maximum depth to keep (meters)
        step: Downsampling step (keep every Nth point)
        mask: Binary segmentation mask (HxW, bool) - if provided, only keep masked points
        
    Returns:
        Filtered points and colors
    """
    # Apply segmentation mask first (if provided)
    if mask is not None:
        # Mask is 2D (HxW), points are flattened from image
        # Need to flatten mask to match points ordering
        mask_flat = mask.flatten()
        if len(mask_flat) != len(points):
            print(f"  Warning: Mask size {mask.shape} doesn't match points {len(points)}")
        else:
            points, colors = points[mask_flat], colors[mask_flat]
            print(f"  Segmentation mask applied: {len(points)} points")
    
    # Crop by depth
    if crop_min is not None or crop_max is not None:
        depth = -points[:, 2]
        depth_mask = np.ones(len(points), dtype=bool)
        if crop_min:
            depth_mask &= depth >= crop_min
        if crop_max:
            depth_mask &= depth <= crop_max
        points, colors = points[depth_mask], colors[depth_mask]
        print(f"  Depth cropped to {len(points)} points")
    
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
    crop_min: float | None = None,
    crop_max: float | None = None,
    mask: np.ndarray | None = None,
    # Debug output options
    save_depth: bool = False,
    save_mask: bool = False,
    save_masked: bool = False,
    save_overlay: bool = False,
    **kwargs  # Ignore extra args
) -> None:
    """Pipeline: depth + RGB -> filtered PLY pointcloud.
    
    Args:
        image: BGR image
        depth_map: Depth map
        output_path: Output PLY file path
        depth_min: Minimum depth value (meters)
        depth_max: Maximum depth value (meters)
        downsample_step: Downsampling step
        crop_min: Minimum depth to keep
        crop_max: Maximum depth to keep
        mask: Binary segmentation mask (HxW, bool) to filter points
        save_depth: Save depth visualization PNG
        save_mask: Save binary mask PNG
        save_masked: Save masked image (background removed)
        save_overlay: Save mask overlay visualization
    """
    # Save debug images
    if save_depth or save_mask or save_masked or save_overlay:
        save_debug_images(
            output_path, image, depth_map, mask,
            save_depth=save_depth,
            save_mask=save_mask,
            save_masked=save_masked,
            save_overlay=save_overlay,
        )
    
    points, colors = depth_to_pointcloud(image, depth_map, depth_min, depth_max)
    points, colors = filter_points(points, colors, crop_min, crop_max, downsample_step, mask)
    export_to_ply(points, colors.astype(np.uint8), output_path)
