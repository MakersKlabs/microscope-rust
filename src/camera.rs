//! Camera discovery and the capture thread (nokhwa: V4L2 / MediaFoundation / AVFoundation).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType,
    Resolution,
};
use nokhwa::Camera;

use crate::frame::{SharedFrame, Transform};

/// A camera found on the system.
pub struct CameraEntry {
    pub key: String,
    pub name: String,
    pub index: CameraIndex,
}

/// Enumerate the connected cameras.
pub fn enumerate() -> Vec<CameraEntry> {
    nokhwa::query(ApiBackend::Auto)
        .unwrap_or_default()
        .into_iter()
        .map(|info| CameraEntry {
            key: info.human_name(),
            name: info.human_name(),
            index: info.index().clone(),
        })
        .collect()
}

/// Open the camera briefly to read its supported (width, height, fps) list.
pub fn probe_formats(index: &CameraIndex) -> Result<Vec<(u32, u32, u32)>, String> {
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
    let mut camera = Camera::new(index.clone(), requested).map_err(|error| error.to_string())?;
    let formats = camera
        .compatible_camera_formats()
        .map_err(|error| error.to_string())?;
    let mut list: Vec<(u32, u32, u32)> = formats
        .iter()
        .map(|format| {
            (
                format.resolution().width(),
                format.resolution().height(),
                format.frame_rate(),
            )
        })
        .collect();
    list.sort_by(|a, b| (b.0 * b.1, b.2).cmp(&(a.0 * a.1, a.2)));
    list.dedup();
    Ok(list)
}

/// Handles to a running capture thread.
pub struct Capture {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    latest: Arc<Mutex<Option<SharedFrame>>>,
    transform: Arc<Mutex<Transform>>,
    recorder: Arc<Mutex<Option<SyncSender<SharedFrame>>>>,
    opened: Arc<Mutex<Option<(usize, usize)>>>,
    error: Arc<Mutex<Option<String>>>,
    fps: Arc<Mutex<f32>>,
}

impl Capture {
    /// Ask the thread to stop and wait for it.
    pub fn stop_and_join(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }

    /// Take the most recent frame, if any.
    pub fn take_frame(&self) -> Option<SharedFrame> {
        self.latest.lock().ok().and_then(|mut slot| slot.take())
    }

    /// Take a pending error message, if any.
    pub fn take_error(&self) -> Option<String> {
        self.error.lock().ok().and_then(|mut slot| slot.take())
    }

    /// Take the resolution reported by the camera on start-up.
    pub fn take_opened(&self) -> Option<(usize, usize)> {
        self.opened.lock().ok().and_then(|mut slot| slot.take())
    }

    pub fn measured_fps(&self) -> f32 {
        self.fps.lock().map(|value| *value).unwrap_or(0.0)
    }

    pub fn set_transform(&self, value: Transform) {
        if let Ok(mut slot) = self.transform.lock() {
            *slot = value;
        }
    }

    pub fn set_recorder(&self, sender: Option<SyncSender<SharedFrame>>) {
        if let Ok(mut slot) = self.recorder.lock() {
            *slot = sender;
        }
    }
}

/// Start capturing from `index`. A width/height of `0` selects the highest resolution.
pub fn start(
    index: CameraIndex,
    width: u32,
    height: u32,
    fps: u32,
    transform: Transform,
) -> Capture {
    let stop = Arc::new(AtomicBool::new(false));
    let latest = Arc::new(Mutex::new(None));
    let shared_transform = Arc::new(Mutex::new(transform));
    let recorder = Arc::new(Mutex::new(None));
    let opened = Arc::new(Mutex::new(None));
    let error = Arc::new(Mutex::new(None));
    let measured = Arc::new(Mutex::new(0.0f32));

    let thread_stop = stop.clone();
    let thread_latest = latest.clone();
    let thread_transform = shared_transform.clone();
    let thread_recorder = recorder.clone();
    let thread_opened = opened.clone();
    let thread_error = error.clone();
    let thread_measured = measured.clone();

    let handle = std::thread::Builder::new()
        .name("capture".to_string())
        .spawn(move || {
            run(
                index,
                width,
                height,
                fps,
                thread_stop,
                thread_latest,
                thread_transform,
                thread_recorder,
                thread_opened,
                thread_error,
                thread_measured,
            );
        })
        .ok();

    Capture {
        stop,
        join: handle,
        latest,
        transform: shared_transform,
        recorder,
        opened,
        error,
        fps: measured,
    }
}

#[allow(clippy::too_many_arguments)]
fn run(
    index: CameraIndex,
    width: u32,
    height: u32,
    fps: u32,
    stop: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<SharedFrame>>>,
    transform: Arc<Mutex<Transform>>,
    recorder: Arc<Mutex<Option<SyncSender<SharedFrame>>>>,
    opened: Arc<Mutex<Option<(usize, usize)>>>,
    error: Arc<Mutex<Option<String>>>,
    measured: Arc<Mutex<f32>>,
) {
    let requested = if width > 0 && height > 0 {
        let format = CameraFormat::new(
            Resolution::new(width, height),
            FrameFormat::MJPEG,
            if fps > 0 { fps } else { 30 },
        );
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(format))
    } else {
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution)
    };

    let mut camera = match Camera::new(index, requested) {
        Ok(camera) => camera,
        Err(err) => {
            set_error(&error, err.to_string());
            return;
        }
    };
    if let Err(err) = camera.open_stream() {
        set_error(&error, err.to_string());
        return;
    }

    let mut count: u32 = 0;
    let mut last = std::time::Instant::now();

    while !stop.load(Ordering::Relaxed) {
        let buffer = match camera.frame() {
            Ok(buffer) => buffer,
            Err(err) => {
                set_error(&error, err.to_string());
                break;
            }
        };
        let decoded = match buffer.decode_image::<RgbFormat>() {
            Ok(image) => image,
            Err(err) => {
                set_error(&error, err.to_string());
                break;
            }
        };

        let current = transform.lock().map(|value| *value).unwrap_or_default();
        let frame = Arc::new(current.apply(decoded));

        if let Ok(mut slot) = opened.lock() {
            if slot.is_none() {
                *slot = Some((frame.width, frame.height));
            }
        }
        if let Ok(mut slot) = latest.lock() {
            *slot = Some(frame.clone());
        }
        if let Ok(guard) = recorder.lock() {
            if let Some(sender) = guard.as_ref() {
                let _ = sender.try_send(frame);
            }
        }

        count += 1;
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last).as_secs_f32();
        if elapsed >= 1.0 {
            if let Ok(mut value) = measured.lock() {
                *value = count as f32 / elapsed;
            }
            count = 0;
            last = now;
        }
    }

    let _ = camera.stop_stream();
}

fn set_error(slot: &Arc<Mutex<Option<String>>>, message: String) {
    if let Ok(mut value) = slot.lock() {
        *value = Some(message);
    }
}
