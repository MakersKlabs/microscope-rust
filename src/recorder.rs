//! H.264 video recorder: OpenH264 encoder + pure-Rust MP4 muxer.
//!
//! Uses OpenH264 (BSD, Cisco) instead of x264, so the application stays under a
//! permissive license, and the `mp4` crate to mux without FFmpeg.

use std::fs::File;
use std::io::BufWriter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use openh264::encoder::{
    BitRate, Complexity, Encoder, EncoderConfig, FrameRate, RateControlMode, UsageType,
};
use openh264::formats::YUVBuffer;

use crate::frame::SharedFrame;

/// Statistics returned when the recording stops.
pub struct RecorderStats {
    pub frames: u64,
    #[allow(dead_code)]
    pub dropped: u64,
}

/// Handles to a running recorder thread.
pub struct Recorder {
    pub path: String,
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<Result<RecorderStats, String>>>,
}

impl Recorder {
    /// Stop the thread, flush the MP4 and return the statistics.
    pub fn stop_and_join(&mut self) -> Result<RecorderStats, String> {
        self.stop.store(true, Ordering::Relaxed);
        match self.join.take() {
            Some(handle) => handle
                .join()
                .unwrap_or_else(|_| Err("recorder thread panicked".to_string())),
            None => Err("recorder already stopped".to_string()),
        }
    }
}

/// Start recording. Returns the recorder handle and the channel used to submit frames.
pub fn start(
    path: String,
    width: usize,
    height: usize,
    fps: u32,
    bitrate_kbps: u32,
) -> Result<(Recorder, SyncSender<SharedFrame>), String> {
    let (sender, receiver) = sync_channel::<SharedFrame>(8);
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    let thread_path = path.clone();

    let handle = std::thread::Builder::new()
        .name("recorder".to_string())
        .spawn(move || {
            run(
                thread_path,
                width,
                height,
                fps,
                bitrate_kbps,
                receiver,
                thread_stop,
            )
        })
        .map_err(|error| error.to_string())?;

    Ok((
        Recorder {
            path,
            stop,
            join: Some(handle),
        },
        sender,
    ))
}

fn run(
    path: String,
    width: usize,
    height: usize,
    fps: u32,
    bitrate_kbps: u32,
    receiver: Receiver<SharedFrame>,
    stop: Arc<AtomicBool>,
) -> Result<RecorderStats, String> {
    let (width, height) = (width & !1, height & !1);
    if width == 0 || height == 0 {
        return Err("invalid frame size".to_string());
    }

    let config = EncoderConfig::new()
        .bitrate(BitRate::from_bps(
            bitrate_kbps.saturating_mul(1000).max(100_000),
        ))
        .max_frame_rate(FrameRate::from_hz(if fps > 0 { fps as f32 } else { 30.0 }))
        .usage_type(UsageType::CameraVideoRealTime)
        .rate_control_mode(RateControlMode::Bitrate)
        .complexity(Complexity::Low)
        .skip_frames(true);

    let mut encoder = Encoder::with_api_config(openh264::OpenH264API::from_source(), config)
        .map_err(|error| error.to_string())?;

    let file = File::create(&path).map_err(|error| error.to_string())?;
    let mut writer: Option<mp4::Mp4Writer<BufWriter<File>>> = None;
    let mut fallback_file = Some(file);
    let mut sps: Option<Vec<u8>> = None;
    let mut pps: Option<Vec<u8>> = None;

    let mut frames: u64 = 0;
    let mut dropped: u64 = 0;
    let mut last_time: u64 = 0;
    let started = Instant::now();
    const TIMESCALE: u32 = 1000;

    loop {
        let frame = match receiver.recv_timeout(Duration::from_millis(150)) {
            Ok(frame) => frame,
            Err(RecvTimeoutError::Timeout) => {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        };

        let i420 = rgb_to_i420(&frame.rgb, frame.width, frame.height);
        let yuv = YUVBuffer::from_vec(i420, width, height);
        let bitstream = encoder.encode(&yuv).map_err(|error| error.to_string())?;
        let annexb = bitstream.to_vec();
        let (avcc, is_keyframe) = annexb_to_avcc(&annexb, &mut sps, &mut pps);

        if writer.is_none() {
            if let (Some(sequence), Some(picture)) = (sps.clone(), pps.clone()) {
                let file = fallback_file.take().ok_or("missing output file")?;
                let mut mp4 =
                    mp4::Mp4Writer::write_start(BufWriter::new(file), &mp4_config(TIMESCALE))
                        .map_err(|error| error.to_string())?;
                mp4.add_track(&track_config(width, height, sequence, picture))
                    .map_err(|error| error.to_string())?;
                writer = Some(mp4);
            }
        }

        let Some(active) = writer.as_mut() else {
            dropped += 1;
            continue;
        };

        let now = started.elapsed().as_millis() as u64;
        let duration = (now.saturating_sub(last_time)).max(1) as u32;
        let sample = mp4::Mp4Sample {
            start_time: last_time,
            duration,
            rendering_offset: 0,
            is_sync: is_keyframe,
            bytes: mp4::Bytes::from(avcc),
        };
        active
            .write_sample(1, &sample)
            .map_err(|error| error.to_string())?;
        last_time = now;
        frames += 1;
    }

    if let Some(mut mp4) = writer {
        mp4.write_end().map_err(|error| error.to_string())?;
    }

    Ok(RecorderStats { frames, dropped })
}

fn mp4_config(timescale: u32) -> mp4::Mp4Config {
    mp4::Mp4Config {
        major_brand: "isom".parse().unwrap_or_default(),
        minor_version: 512,
        compatible_brands: vec![
            "isom".parse().unwrap_or_default(),
            "iso2".parse().unwrap_or_default(),
            "avc1".parse().unwrap_or_default(),
            "mp41".parse().unwrap_or_default(),
        ],
        timescale,
    }
}

fn track_config(width: usize, height: usize, sps: Vec<u8>, pps: Vec<u8>) -> mp4::TrackConfig {
    mp4::TrackConfig {
        track_type: mp4::TrackType::Video,
        timescale: 1000,
        language: "und".to_string(),
        media_conf: mp4::MediaConfig::AvcConfig(mp4::AvcConfig {
            width: width as u16,
            height: height as u16,
            seq_param_set: sps,
            pic_param_set: pps,
        }),
    }
}

/// Convert an Annex-B H.264 packet from OpenH264 into AVCC (length-prefixed)
/// and collect SPS/PPS parameter sets for the MP4 `avcC` box.
fn annexb_to_avcc(
    packet: &[u8],
    sps: &mut Option<Vec<u8>>,
    pps: &mut Option<Vec<u8>>,
) -> (Vec<u8>, bool) {
    let mut output = Vec::with_capacity(packet.len() + 16);
    let mut keyframe = false;

    for nal in split_annexb(packet) {
        if nal.is_empty() {
            continue;
        }
        let nal_type = nal[0] & 0x1f;
        match nal_type {
            5 => keyframe = true,
            7 if sps.is_none() => *sps = Some(nal.to_vec()),
            8 if pps.is_none() => *pps = Some(nal.to_vec()),
            _ => {}
        }
        output.extend_from_slice(&(nal.len() as u32).to_be_bytes());
        output.extend_from_slice(nal);
    }

    (output, keyframe)
}

/// Split an Annex-B byte stream into NAL units (start codes removed).
fn split_annexb(data: &[u8]) -> Vec<&[u8]> {
    let mut units: Vec<&[u8]> = Vec::new();
    let length = data.len();
    let mut index = 0;

    // Skip to the first start code.
    let mut start = loop {
        if index + 3 > length {
            return units;
        }
        if data[index] == 0 && data[index + 1] == 0 && data[index + 2] == 1 {
            break index + 3;
        }
        index += 1;
    };

    index = start;
    while index + 3 <= length {
        if data[index] == 0 && data[index + 1] == 0 && data[index + 2] == 1 {
            push_nal(&mut units, &data[start..index]);
            index += 3;
            start = index;
        } else {
            index += 1;
        }
    }
    push_nal(&mut units, &data[start..length]);
    units
}

fn push_nal<'a>(units: &mut Vec<&'a [u8]>, nal: &'a [u8]) {
    // Strip trailing zero bytes left over from 4-byte start codes.
    let mut end = nal.len();
    while end > 0 && nal[end - 1] == 0 {
        end -= 1;
    }
    if end > 0 {
        units.push(&nal[..end]);
    }
}

/// Convert an RGB frame (BT.601, limited range) to I420/YUV420p.
fn rgb_to_i420(rgb: &[u8], src_width: usize, src_height: usize) -> Vec<u8> {
    let width = src_width & !1;
    let height = src_height & !1;
    let mut output = vec![0u8; width * height * 3 / 2];

    let (luma, chroma) = output.split_at_mut(width * height);
    let (u_plane, v_plane) = chroma.split_at_mut(width * height / 4);

    for y in 0..height {
        for x in 0..width {
            let pixel = (y * src_width + x) * 3;
            let r = rgb[pixel] as i32;
            let g = rgb[pixel + 1] as i32;
            let b = rgb[pixel + 2] as i32;
            let value = ((66 * r + 129 * g + 25 * b + 128) >> 8) + 16;
            luma[y * width + x] = value.clamp(0, 255) as u8;
        }
    }

    for y in 0..height / 2 {
        for x in 0..width / 2 {
            let mut r_sum = 0i32;
            let mut g_sum = 0i32;
            let mut b_sum = 0i32;
            for dy in 0..2 {
                for dx in 0..2 {
                    let pixel = ((y * 2 + dy) * src_width + (x * 2 + dx)) * 3;
                    r_sum += rgb[pixel] as i32;
                    g_sum += rgb[pixel + 1] as i32;
                    b_sum += rgb[pixel + 2] as i32;
                }
            }
            let r = r_sum / 4;
            let g = g_sum / 4;
            let b = b_sum / 4;
            let u = ((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128;
            let v = ((112 * r - 94 * g - 18 * b + 128) >> 8) + 128;
            u_plane[y * (width / 2) + x] = u.clamp(0, 255) as u8;
            v_plane[y * (width / 2) + x] = v.clamp(0, 255) as u8;
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;

    #[test]
    fn encodes_and_muxes_h264_mp4() {
        let path = std::env::temp_dir().join("microscope_rust_selftest.mp4");
        let _ = std::fs::remove_file(&path);

        let (mut recorder, sender) =
            start(path.to_string_lossy().to_string(), 320, 240, 30, 2000).unwrap();

        for index in 0..30u32 {
            let mut rgb = vec![0u8; 320 * 240 * 3];
            for (n, pixel) in rgb.chunks_exact_mut(3).enumerate() {
                pixel[0] = (n as u32).wrapping_add(index * 5) as u8;
                pixel[1] = (index * 8) as u8;
                pixel[2] = 160;
            }
            let frame = Arc::new(Frame {
                width: 320,
                height: 240,
                rgb,
            });
            sender.send(frame).unwrap();
        }
        drop(sender);

        let stats = recorder.stop_and_join().unwrap();
        assert!(stats.frames > 0, "no frames were written");

        let file = std::fs::File::open(&path).unwrap();
        let size = file.metadata().unwrap().len();
        assert!(size > 1024, "output file too small");

        let reader = mp4::Mp4Reader::read_header(std::io::BufReader::new(file), size).unwrap();
        let tracks: Vec<_> = reader.tracks().values().collect();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].width(), 320);
        assert_eq!(tracks[0].height(), 240);

        let _ = std::fs::remove_file(&path);
    }
}
