//! Camera discovery and the capture thread (nokhwa: V4L2 / MediaFoundation / AVFoundation).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{ApiBackend, CameraFormat, CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;

use crate::frame::{SharedFrame, Transform};

/// A camera found on the system.
///
/// Many USB cameras expose **two** device nodes with the same name: a real
/// capture node and a metadata node. They are grouped here, and the working
/// node (`active_index`) is resolved when the camera is probed.
pub struct CameraEntry {
    pub key: String,
    pub name: String,
    pub indices: Vec<CameraIndex>,
    pub active_index: Option<CameraIndex>,
}

/// Enumerate the connected cameras, grouped by name and sorted by device number.
pub fn enumerate() -> Vec<CameraEntry> {
    let mut entries: Vec<CameraEntry> = Vec::new();
    for info in nokhwa::query(ApiBackend::Auto).unwrap_or_default() {
        let name = info.human_name();
        if let Some(existing) = entries.iter_mut().find(|entry| entry.name == name) {
            existing.indices.push(info.index().clone());
        } else {
            entries.push(CameraEntry {
                key: name.clone(),
                name,
                indices: vec![info.index().clone()],
                active_index: None,
            });
        }
    }

    for entry in &mut entries {
        entry.indices.sort_by_key(|index| index_number(index));
        entry.indices.dedup();
    }
    entries.sort_by_key(|entry| entry.indices.first().map(index_number).unwrap_or(u32::MAX));
    entries
}

fn index_number(index: &CameraIndex) -> u32 {
    match index {
        CameraIndex::Index(value) => *value,
        CameraIndex::String(_) => u32::MAX,
    }
}

/// Resolve the working device node and return its supported formats.
///
/// The metadata node of a camera fails to open, so the candidate nodes are
/// tried in order until one exposes a usable format list.
pub fn probe_formats(entry: &mut CameraEntry) -> Result<Vec<CameraFormat>, String> {
    let mut last_error = String::from("no compatible device node");
    for index in &entry.indices {
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
        match Camera::new(index.clone(), requested) {
            Ok(mut camera) => match camera.compatible_camera_formats() {
                Ok(formats) if !formats.is_empty() => {
                    entry.active_index = Some(index.clone());
                    return Ok(formats);
                }
                Ok(_) => last_error = format!("{index:?}: no formats reported"),
                Err(error) => last_error = format!("{index:?}: {error}"),
            },
            Err(error) => last_error = format!("{index:?}: {error}"),
        }
    }
    Err(last_error)
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

/// Start capturing from `index` using the exact `format` selected by the user.
pub fn start(index: CameraIndex, format: Option<CameraFormat>, transform: Transform) -> Capture {
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
                format,
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
    format: Option<CameraFormat>,
    stop: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<SharedFrame>>>,
    transform: Arc<Mutex<Transform>>,
    recorder: Arc<Mutex<Option<SyncSender<SharedFrame>>>>,
    opened: Arc<Mutex<Option<(usize, usize)>>>,
    error: Arc<Mutex<Option<String>>>,
    measured: Arc<Mutex<f32>>,
) {
    // Use the exact format chosen by the user so the device never negotiates an
    // unsupported resolution (which would result in a black "NO SIGNAL" view).
    let requested = match format {
        Some(format) => RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(format)),
        None => RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
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
