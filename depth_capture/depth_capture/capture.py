"""Image and depth capture using webcam or file input."""

import cv2
import numpy as np
import torch
from pathlib import Path


def center_crop_and_resize(image: np.ndarray, size: int = 512) -> np.ndarray:
    """Center crop image to square and resize.
    
    Args:
        image: Input image (HxWx3, uint8)
        size: Target size for the square output (default: 512)
        
    Returns:
        Cropped and resized image (sizexsize, uint8)
    """
    h, w = image.shape[:2]
    
    # Determine crop size (smallest dimension)
    crop_size = min(h, w)
    
    # Calculate crop offsets (center crop)
    start_y = (h - crop_size) // 2
    start_x = (w - crop_size) // 2
    
    # Crop to square
    cropped = image[start_y:start_y + crop_size, start_x:start_x + crop_size]
    
    # Resize to target size
    resized = cv2.resize(cropped, (size, size), interpolation=cv2.INTER_AREA)
    
    print(f"Center cropped and resized: {w}x{h} -> {size}x{size}")
    return resized


class DepthEstimator:
    """MiDaS-based depth estimator."""
    
    def __init__(self, model_type: str = "DPT_Large"):
        """Initialize MiDaS depth estimator.
        
        Args:
            model_type: MiDaS model type (DPT_Large, DPT_Hybrid, or MiDaS_small)
        """
        print(f"Loading MiDaS model: {model_type}")
        
        # Load MiDaS model from torch hub
        self.model = torch.hub.load("intel-isl/MiDaS", model_type)
        
        # Set device (CPU or GPU)
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        self.model.to(self.device)
        self.model.eval()
        
        # Load transforms
        midas_transforms = torch.hub.load("intel-isl/MiDaS", "transforms")
        
        if model_type in ["DPT_Large", "DPT_Hybrid"]:
            self.transform = midas_transforms.dpt_transform
        else:
            self.transform = midas_transforms.small_transform
        
        print(f"Model loaded on {self.device}")
    
    def estimate_depth(self, image: np.ndarray) -> np.ndarray:
        """Estimate depth from RGB image.
        
        Args:
            image: RGB image (HxWx3, uint8)
            
        Returns:
            Depth map (HxW, float32)
        """
        # Convert BGR to RGB if needed (OpenCV uses BGR)
        if image.shape[2] == 3:
            image_rgb = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
        else:
            image_rgb = image
        
        # Apply transforms
        input_batch = self.transform(image_rgb).to(self.device)
        
        # Predict depth
        with torch.no_grad():
            prediction = self.model(input_batch)
            
            # Resize to original resolution
            prediction = torch.nn.functional.interpolate(
                prediction.unsqueeze(1),
                size=image.shape[:2],
                mode="bicubic",
                align_corners=False,
            ).squeeze()
        
        depth_map = prediction.cpu().numpy()
        
        return depth_map


def capture_from_webcam(resolution: tuple[int, int] = (640, 480)) -> np.ndarray:
    """Capture a frame from the webcam.
    
    Args:
        resolution: (width, height) for webcam capture
        
    Returns:
        Captured image (HxWx3, BGR, uint8)
        
    Raises:
        RuntimeError: If webcam cannot be opened
    """
    cap = cv2.VideoCapture(0)
    
    if not cap.isOpened():
        raise RuntimeError("Failed to open webcam")
    
    # Set resolution
    cap.set(cv2.CAP_PROP_FRAME_WIDTH, resolution[0])
    cap.set(cv2.CAP_PROP_FRAME_HEIGHT, resolution[1])
    
    # Warm up camera
    for _ in range(5):
        cap.read()
    
    # Capture frame
    ret, frame = cap.read()
    cap.release()
    
    if not ret:
        raise RuntimeError("Failed to capture frame from webcam")
    
    print(f"Captured frame from webcam: {frame.shape[1]}x{frame.shape[0]}")
    return frame


def load_from_file(image_path: str) -> np.ndarray:
    """Load image from file.
    
    Args:
        image_path: Path to image file
        
    Returns:
        Loaded image (HxWx3, BGR, uint8)
        
    Raises:
        FileNotFoundError: If image file doesn't exist
        ValueError: If image cannot be loaded
    """
    path = Path(image_path)
    
    if not path.exists():
        raise FileNotFoundError(f"Image file not found: {path}")
    
    image = cv2.imread(str(path))
    
    if image is None:
        raise ValueError(f"Failed to load image from {path}")
    
    print(f"Loaded image from {path}: {image.shape[1]}x{image.shape[0]}")
    return image
