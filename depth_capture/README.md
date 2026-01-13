# Depth Capture

Monocular depth estimation tool using MiDaS to generate PLY pointclouds from webcam or image files.

## Installation

```bash
uv sync
```

## Usage

### From Webcam

Capture a frame from your Mac webcam and generate a PLY pointcloud:

```bash
uv run python main.py --output scene.ply --resolution 640x480
```

### From Image File

Process an existing image file:

```bash
uv run python main.py --input ../data/test_images/color_car1.jpg --output car.ply
```

### Options

- `--input`: Path to input image file (optional, uses webcam if not specified)
- `--output`: Path to output PLY file (default: `output.ply`)
- `--resolution`: Webcam resolution in WIDTHxHEIGHT format (default: `640x480`, only used for webcam)

## How It Works

1. **Input**: Captures frame from webcam or loads image file
2. **Depth Estimation**: Uses MiDaS model to estimate depth from monocular image
3. **Pointcloud Generation**: Combines RGB image with depth map to create 3D pointcloud
4. **Export**: Saves as PLY file compatible with the quad_app patterns module

## Using with SkyCanvas

Once you've generated a PLY file, you can use it with the SkyCanvas patterns module:

```python
from quad_app.patterns import generate_from_pointcloud, PointcloudConfig

config = PointcloudConfig(
    center=(5.0, 0.0, -5.0),
    ply_path="depth_capture/car.ply",
    density=0.2,
    depth_scale=2.0  # 0 = flat 2D, >0 = 2.5D relief
)
path = generate_from_pointcloud(config)
```
