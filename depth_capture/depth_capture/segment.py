"""YOLOE segmentation for object selection and masking.

Uses Ultralytics YOLOE for real-time open-vocabulary segmentation.
https://docs.ultralytics.com/models/yoloe/
"""

import numpy as np
import cv2


class YOLOESegmenter:
    """YOLOE segmentation with text prompt support."""
    
    def __init__(self, model_name: str = "yoloe-11l-seg.pt"):
        """Initialize YOLOE model.
        
        Args:
            model_name: YOLOE model name (downloads automatically)
                       Options: yoloe-11s-seg.pt, yoloe-11m-seg.pt, yoloe-11l-seg.pt
        """
        print(f"Loading YOLOE model: {model_name}...")
        
        try:
            from ultralytics import YOLO
            
            self.model = YOLO(model_name)
            print(f"YOLOE loaded successfully")
            
        except ImportError as e:
            raise ImportError(
                "Ultralytics not installed. Run: uv sync\n"
                f"Error: {e}"
            )
    
    def segment(self, image: np.ndarray, text_prompt: str) -> np.ndarray:
        """Segment objects matching text prompt.
        
        Args:
            image: BGR image (HxWx3, uint8) from OpenCV
            text_prompt: Text description (e.g., "car", "person", "dog")
            
        Returns:
            Binary mask (HxW, bool) where True = selected object pixels
        """
        print(f"Segmenting: '{text_prompt}'")
        
        # Convert BGR to RGB
        image_rgb = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
        
        # Set the class to detect
        classes = [text_prompt]
        self.model.set_classes(classes, self.model.get_text_pe(classes))
        
        # Run inference
        results = self.model.predict(
            image_rgb,
            device="mps",
            verbose=False,
        )
        
        # Extract masks
        if results and len(results) > 0 and results[0].masks is not None:
            masks = results[0].masks.data.cpu().numpy()
            
            if len(masks) > 0:
                # Combine all detected masks
                combined = np.any(masks > 0.5, axis=0)
                
                # Resize mask to match input image if needed
                if combined.shape != image.shape[:2]:
                    combined = cv2.resize(
                        combined.astype(np.uint8),
                        (image.shape[1], image.shape[0]),
                        interpolation=cv2.INTER_NEAREST
                    ).astype(bool)
                
                pixel_count = combined.sum()
                print(f"  Found {len(masks)} instance(s), {pixel_count:,} pixels selected")
                return combined
        
        print(f"  Warning: No objects found matching '{text_prompt}'")
        return np.zeros(image.shape[:2], dtype=bool)
    
    def segment_multi(self, image: np.ndarray, prompts: list[str]) -> np.ndarray:
        """Segment multiple object types at once.
        
        Args:
            image: BGR image (HxWx3, uint8)
            prompts: List of text prompts (e.g., ["car", "person"])
            
        Returns:
            Binary mask (HxW, bool) combining all matches
        """
        print(f"Segmenting: {prompts}")
        
        image_rgb = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
        
        # Set multiple classes
        self.model.set_classes(prompts, self.model.get_text_pe(prompts))
        
        results = self.model.predict(
            image_rgb,
            device="mps",
            verbose=False,
        )
        
        if results and len(results) > 0 and results[0].masks is not None:
            masks = results[0].masks.data.cpu().numpy()
            
            if len(masks) > 0:
                combined = np.any(masks > 0.5, axis=0)
                
                if combined.shape != image.shape[:2]:
                    combined = cv2.resize(
                        combined.astype(np.uint8),
                        (image.shape[1], image.shape[0]),
                        interpolation=cv2.INTER_NEAREST
                    ).astype(bool)
                
                print(f"  Found {len(masks)} instance(s)")
                return combined
        
        print(f"  Warning: No objects found")
        return np.zeros(image.shape[:2], dtype=bool)
