//! Debug helper: list cameras, and optionally open one index.
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cameras = nokhwa::query(ApiBackend::Auto).unwrap_or_default();
    println!("query() -> {} entries", cameras.len());
    for camera in &cameras {
        println!(
            "  index={:?}  name={:?}  desc={:?}  misc={:?}",
            camera.index(),
            camera.human_name(),
            camera.description(),
            camera.misc()
        );
    }
    std::io::stdout().flush().ok();

    if args.len() < 2 {
        return;
    }
    let index: u32 = args[1].parse().unwrap();
    println!("opening index {} ...", index);
    std::io::stdout().flush().ok();
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
    match Camera::new(CameraIndex::Index(index), requested) {
        Ok(mut cam) => match cam.compatible_camera_formats() {
            Ok(list) => {
                println!("OK: {} formats", list.len());
                for f in list.iter().take(30) {
                    println!(
                        "    {}x{}@{} {:?}",
                        f.resolution().width(),
                        f.resolution().height(),
                        f.frame_rate(),
                        f.format()
                    );
                }
            }
            Err(e) => println!("OPEN OK but formats error: {e}"),
        },
        Err(e) => println!("OPEN FAILED: {e}"),
    }
}
