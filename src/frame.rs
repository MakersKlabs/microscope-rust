//! A captured frame and the rotate/mirror transform.

use std::sync::Arc;

/// A decoded RGB frame.
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>,
}

/// Frames are shared between the UI and the recorder without copying.
pub type SharedFrame = Arc<Frame>;

/// Rotation and mirroring applied to every frame.
#[derive(Clone, Copy, Default)]
pub struct Transform {
    pub rotation: u16,
    pub flip_h: bool,
    pub flip_v: bool,
}

impl Transform {
    /// Apply the transform to an RGB image.
    pub fn apply(&self, image: image::RgbImage) -> Frame {
        let mut image = match self.rotation % 360 {
            90 => image::imageops::rotate90(&image),
            180 => image::imageops::rotate180(&image),
            270 => image::imageops::rotate270(&image),
            _ => image,
        };
        if self.flip_h {
            image::imageops::flip_horizontal_in_place(&mut image);
        }
        if self.flip_v {
            image::imageops::flip_vertical_in_place(&mut image);
        }
        let (width, height) = image.dimensions();
        Frame {
            width: width as usize,
            height: height as usize,
            rgb: image.into_raw(),
        }
    }
}
