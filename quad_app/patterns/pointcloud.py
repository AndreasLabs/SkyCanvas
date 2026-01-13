"""Pointcloud-based pattern generation from PLY files."""

import logging
import numpy as np
from pathlib import Path
from plyfile import PlyData
from quad_app.patterns.base import PointcloudConfig
from quad_app.waypoints import Waypoint


def generate_from_pointcloud(config: PointcloudConfig) -> list[Waypoint]:
    """Generate waypoints from a PLY pointcloud file.
    
    The pointcloud is mapped to NED coordinates:
    - X from PLY -> North (NED X)
    - Y from PLY -> East (NED Y), scaled by depth_scale
    - Z from PLY -> Down (NED Z, inverted for altitude)
    
    When depth_scale = 0, all points are projected to East=0 (flat 2D image).
    When depth_scale > 0, depth creates a 2.5D relief effect.
    
    Args:
        config: Pointcloud configuration including file path and parameters
        
    Returns:
        List of waypoints sampled from the pointcloud
        
    Raises:
        FileNotFoundError: If PLY file doesn't exist
        ValueError: If PLY file is invalid or missing required data
    """
    ply_path = Path(config.ply_path)
    if not ply_path.exists():
        raise FileNotFoundError(f"PLY file not found: {ply_path}")
    
    logging.info(f"Loading pointcloud from {ply_path}")
    
    # Load PLY file
    try:
        ply_data = PlyData.read(str(ply_path))
        vertex_data = ply_data['vertex']
    except Exception as e:
        raise ValueError(f"Failed to read PLY file: {e}")
    
    # Extract XYZ coordinates
    if not all(prop in vertex_data for prop in ['x', 'y', 'z']):
        raise ValueError("PLY file missing x, y, z coordinates")
    
    points = np.column_stack([
        vertex_data['x'],
        vertex_data['y'],
        vertex_data['z']
    ])
    
    # Extract RGB colors (normalized to 0-1 range)
    if all(prop in vertex_data for prop in ['red', 'green', 'blue']):
        colors = np.column_stack([
            vertex_data['red'],
            vertex_data['green'],
            vertex_data['blue']
        ]).astype(float)
        
        # Normalize to 0-1 if values are in 0-255 range
        if colors.max() > 1.0:
            colors = colors / 255.0
    else:
        # Use default color if no RGB data
        logging.warning("PLY file missing RGB data, using default color")
        colors = np.tile(config.default_color, (len(points), 1))
    
    logging.info(f"Loaded {len(points)} points from pointcloud")
    
    # Downsample based on density
    sampled_indices = _downsample_pointcloud(points, config.density)
    points = points[sampled_indices]
    colors = colors[sampled_indices]
    
    logging.info(f"Downsampled to {len(points)} points (density={config.density}m)")
    
    # Normalize pointcloud to center it and scale appropriately
    points_centered = _normalize_pointcloud(points, config.scale)
    
    # Map to NED coordinates
    waypoints = []
    for i, (point, color) in enumerate(zip(points_centered, colors)):
        # Map pointcloud coordinates to NED
        # X -> North, Y -> depth (East with scale), Z -> Down (inverted)
        north = config.center[0] + point[0]
        
        # Apply depth scaling
        if config.depth_scale > 0:
            # Normalize depth to 0-1 range and scale
            depth_normalized = (point[1] - points_centered[:, 1].min()) / (
                points_centered[:, 1].max() - points_centered[:, 1].min() + 1e-8
            )
            east = config.center[1] + depth_normalized * config.depth_scale
        else:
            # Flat 2D projection
            east = config.center[1]
        
        down = config.center[2] - point[2]  # Invert Z for altitude
        
        waypoints.append(Waypoint(
            ned=[north, east, down],
            color=[float(color[0]), float(color[1]), float(color[2])],
            hold_time=config.hold_time
        ))
    
    logging.info(f"Generated {len(waypoints)} waypoints from pointcloud")
    return waypoints


def _downsample_pointcloud(points: np.ndarray, density: float) -> np.ndarray:
    """Downsample pointcloud using voxel grid filtering.
    
    Args:
        points: Nx3 array of XYZ coordinates
        density: Minimum distance between points in meters
        
    Returns:
        Array of indices of selected points
    """
    if density <= 0:
        return np.arange(len(points))
    
    # Create voxel grid
    voxel_size = density
    voxel_coords = np.floor(points / voxel_size).astype(int)
    
    # Find unique voxels and keep first point in each voxel
    _, unique_indices = np.unique(voxel_coords, axis=0, return_index=True)
    
    return unique_indices


def _normalize_pointcloud(points: np.ndarray, scale: float) -> np.ndarray:
    """Center and scale pointcloud.
    
    Args:
        points: Nx3 array of XYZ coordinates
        scale: Scale factor to apply
        
    Returns:
        Normalized pointcloud centered at origin
    """
    # Center at origin
    centroid = points.mean(axis=0)
    points_centered = points - centroid
    
    # Scale to desired size
    max_extent = np.abs(points_centered).max()
    if max_extent > 0:
        points_centered = (points_centered / max_extent) * scale
    
    return points_centered
