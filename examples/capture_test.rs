//! Debug helper: open a camera with an exact MJPEG format and grab frames.
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;

fn main() {
    let index: u32 = std::env::args()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
    let mut camera = Camera::new(CameraIndex::Index(index), requested).unwrap();
    let formats = camera.compatible_camera_formats().unwrap();
    let target = formats
        .iter()
        .find(|f| matches!(f.format(), FrameFormat::MJPEG))
        .cloned()
        .or_else(|| formats.first().cloned())
        .unwrap();
    println!(
        "target: {:?} {}x{}@{}",
        target.format(),
        target.resolution().width(),
        target.resolution().height(),
        target.frame_rate()
    );
    drop(camera);

    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(target));
    let mut camera = Camera::new(CameraIndex::Index(index), requested).unwrap();
    camera.open_stream().unwrap();
    for i in 0..5 {
        let buffer = camera.frame().unwrap();
        let image = buffer.decode_image::<RgbFormat>().unwrap();
        println!("frame {}: {}x{}", i, image.width(), image.height());
    }
    camera.stop_stream().unwrap();
    println!("OK");
}
