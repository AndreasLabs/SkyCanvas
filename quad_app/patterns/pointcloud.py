"""Pointcloud-based pattern generation from PLY files."""

import logging
import numpy as np
from pathlib import Path
from plyfile import PlyData
from quad_app.patterns.base import PointcloudConfig
from quad_app.waypoints import Waypoint
from skycanvas_config import Config


def generate_from_pointcloud() -> list[Waypoint]:
    """Generate waypoints from a PLY pointcloud file using global Config.
    
    The PLY coordinate system (from depth_capture/export_ply.py):
    - X = horizontal in image (left/right)
    - Y = vertical in image (up is positive)
    - Z = -depth (negative values, more negative = farther)
    
    Mapped to NED flight coordinates:
    - PLY X -> North (horizontal movement)
    - PLY Y -> Down (inverted: up in image = higher altitude = more negative Down)
    - PLY Z -> East (depth for 2.5D relief, scaled by depth_scale)
    
    When depth_scale = 0, all points project to East=0 (flat 2D image plane).
    When depth_scale > 0, depth creates a 2.5D relief effect in East direction.
    
    Configuration is read from global Config singleton:
    - Config['mission.ply_path']: Path to PLY file
    - Config['mission.center']: NED center position
    - Config['mission.scale']: Scale factor in meters
    - Config['mission.density']: Minimum distance between points
    - Config['mission.depth_scale']: Depth range for 2.5D effect
    - Config['mission.hold_time']: Time to hold at each waypoint
    - Config['mission.spatial_sort']: Enable zig-zag sorting
    
    Returns:
        List of waypoints sampled from the pointcloud
        
    Raises:
        FileNotFoundError: If PLY file doesn't exist
        ValueError: If PLY file is invalid or missing required data
    """
    # Read all config values from global Config
    ply_path = Path(Config.get('mission.ply_path', 'data/test_images/depth_out/color_car1.ply'))
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
        default_color = Config.get('mission.default_color', [1.0, 1.0, 1.0])
        colors = np.tile(default_color, (len(points), 1))
    
    logging.info(f"Loaded {len(points)} points from pointcloud")
    
    # Get all config values from global Config
    scale = Config.get('mission.scale', 25.0)
    density = Config.get('mission.density', 0.1)
    spatial_sort = Config.get('mission.spatial_sort', False)
    depth_scale = Config.get('mission.depth_scale', 1.0)
    hold_time = Config.get('mission.hold_time', 0.15)
    
    # Normalize pointcloud BEFORE downsampling so density works in scaled space
    points_normalized = _normalize_pointcloud(points, scale)
    
    # Downsample based on density (now in scaled meters)
    sampled_indices = _downsample_pointcloud(points_normalized, density)
    points_normalized = points_normalized[sampled_indices]
    colors = colors[sampled_indices]
    
    logging.info(f"Downsampled to {len(points_normalized)} points (density={density}m)")
    
    # Optionally sort points spatially for efficient traversal (zig-zag/lawnmower pattern)
    if spatial_sort:
        # Row by Y (vertical in image -> altitude), column by X (horizontal -> north)
        points_normalized, colors = _sort_spatially(
            points_normalized, colors, 
            row_axis=1,  # Y = vertical rows
            col_axis=0,  # X = horizontal within rows
            row_tolerance=density * 1.5  # Group nearby points into rows
        )
        logging.info(f"Sorted points spatially (zig-zag pattern by Y rows, X columns)")
    
    # Convert center config to tuple (handle Lua tables converted to dicts)
    center_raw = Config.get('mission.center', [0.0, 0.0, -25.0])
    center = _ensure_tuple(center_raw, default=[0.0, 0.0, -25.0])
    
    # Map to NED coordinates
    # PLY: X=horizontal, Y=vertical(up+), Z=-depth
    # NED: North, East, Down (positive down = lower altitude)
    waypoints = []
    for i, (point, color) in enumerate(zip(points_normalized, colors)):
        # PLY X -> North (horizontal in image -> horizontal in flight)
        north = center[0] + point[0]
        
        # PLY Z -> East (depth creates 2.5D relief effect)
        # Z is negative in PLY (more negative = farther), normalize to 0-1 range
        if depth_scale > 0:
            z_min = points_normalized[:, 2].min()
            z_max = points_normalized[:, 2].max()
            z_range = z_max - z_min + 1e-8
            depth_normalized = (point[2] - z_min) / z_range
            east = center[1] + depth_normalized * depth_scale
        else:
            # Flat 2D projection
            east = center[1]
        
        # PLY Y -> Down (inverted: positive Y in PLY = up = more negative Down)
        # In NED, negative Down = higher altitude
        down = center[2] - point[1]
        
        waypoints.append(Waypoint(
            ned=[north, east, down],
            color=[float(color[0]), float(color[1]), float(color[2])],
            hold_time=hold_time,
            segment_id=1 # For now just set to 1 for all points so its one giant line
        ))
    
    logging.info(f"Generated {len(waypoints)} waypoints from pointcloud")
    return waypoints


def _ensure_tuple(value, default):
    """Convert various formats to tuple (handles Lua tables, dicts, lists)."""
    if value is None:
        return default
    if isinstance(value, tuple):
        return value
    if isinstance(value, list):
        return tuple(value)
    if isinstance(value, dict):
        # Lua tables become dicts with numeric keys {1: x, 2: y, 3: z}
        try:
            return tuple(value[i] for i in sorted(value.keys()))
        except (KeyError, TypeError):
            return default
    return default


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


def _sort_spatially(points: np.ndarray, colors: np.ndarray, row_axis: int = 1, col_axis: int = 0, row_tolerance: float = 0.3) -> tuple[np.ndarray, np.ndarray]:
    """Sort points in a zig-zag (boustrophedon) pattern for efficient traversal.
    
    Groups points into rows based on row_axis, then sorts within each row by col_axis,
    alternating direction for a lawnmower pattern that minimizes travel distance.
    
    Args:
        points: Nx3 array of XYZ coordinates
        colors: Nx3 array of RGB colors
        row_axis: Axis to group rows by (0=X, 1=Y, 2=Z). Default Y for horizontal rows.
        col_axis: Axis to sort within rows (0=X, 1=Y, 2=Z). Default X.
        row_tolerance: Distance threshold for grouping points into same row
        
    Returns:
        Tuple of (sorted_points, sorted_colors)
    """
    if len(points) == 0:
        return points, colors
    
    # Get row values and discretize into bins
    row_values = points[:, row_axis]
    row_min, row_max = row_values.min(), row_values.max()
    
    # Assign each point to a row bin
    if row_max - row_min < row_tolerance:
        # All points in one row
        row_bins = np.zeros(len(points), dtype=int)
    else:
        row_bins = np.floor((row_values - row_min) / row_tolerance).astype(int)
    
    # Get unique rows sorted
    unique_rows = np.unique(row_bins)
    
    # Build sorted index list with zig-zag pattern
    sorted_indices = []
    for i, row in enumerate(unique_rows):
        # Get indices of points in this row
        row_mask = row_bins == row
        row_indices = np.where(row_mask)[0]
        
        # Sort by column axis
        col_values = points[row_indices, col_axis]
        col_order = np.argsort(col_values)
        
        # Reverse every other row for zig-zag
        if i % 2 == 1:
            col_order = col_order[::-1]
        
        sorted_indices.extend(row_indices[col_order])
    
    sorted_indices = np.array(sorted_indices)
    return points[sorted_indices], colors[sorted_indices]
