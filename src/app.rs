//! The application: retro-styled UI, live preview and the media actions.

use std::time::Instant;

use eframe::egui::{self, Color32, RichText, TextureHandle, TextureOptions};

use crate::camera::{self, CameraEntry, Capture};
use crate::frame::{SharedFrame, Transform};
use crate::i18n;
use crate::i18n::t;
use crate::recorder::{self, Recorder};
use crate::settings::{CameraSettings, Settings};
use crate::theme;
use crate::util;

const IMAGE_EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

pub struct MicroscopeApp {
    settings: Settings,
    cameras: Vec<CameraEntry>,
    selected_camera: usize,
    capture: Option<Capture>,

    formats: Vec<(u32, u32, u32)>,
    resolutions: Vec<(u32, u32)>,
    selected_resolution: usize,
    framerates: Vec<u32>,
    selected_fps: usize,

    rotation: u16,
    flip_h: bool,
    flip_v: bool,

    texture: Option<TextureHandle>,
    last_frame: Option<SharedFrame>,
    image_size: Option<egui::Vec2>,
    zoom: f32,
    offset: egui::Vec2,
    fit: bool,

    recorder: Option<Recorder>,
    record_start: Option<Instant>,
    record_frames: u64,

    measured_fps: f32,
    actual_resolution: Option<(usize, usize)>,
    status: String,
    status_until: Option<Instant>,

    show_settings: bool,
}

impl MicroscopeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        theme::apply(&cc.egui_ctx);
        let mut app = Self {
            settings,
            cameras: Vec::new(),
            selected_camera: 0,
            capture: None,
            formats: Vec::new(),
            resolutions: Vec::new(),
            selected_resolution: 0,
            framerates: Vec::new(),
            selected_fps: 0,
            rotation: 0,
            flip_h: false,
            flip_v: false,
            texture: None,
            last_frame: None,
            image_size: None,
            zoom: 1.0,
            offset: egui::Vec2::ZERO,
            fit: true,
            recorder: None,
            record_start: None,
            record_frames: 0,
            measured_fps: 0.0,
            actual_resolution: None,
            status: t("ready").to_string(),
            status_until: None,
            show_settings: false,
        };
        app.refresh_cameras();
        app
    }

    // ------------------------------------------------------------------
    // Cameras
    // ------------------------------------------------------------------
    fn refresh_cameras(&mut self) {
        self.cameras = camera::enumerate();
        let wanted = self
            .settings
            .selected_camera
            .clone()
            .or_else(|| self.cameras.first().map(|camera| camera.key.clone()));

        let index = wanted
            .and_then(|key| self.cameras.iter().position(|camera| camera.key == key))
            .unwrap_or(0);
        self.selected_camera = index;

        if self.cameras.is_empty() {
            self.set_status(t("no_camera"));
            self.stop_capture();
            return;
        }
        self.on_camera_changed();
    }

    fn on_camera_changed(&mut self) {
        self.stop_recording(true);
        self.stop_capture();
        if self.cameras.is_empty() {
            return;
        }
        let key = self.cameras[self.selected_camera].key.clone();
        self.settings.selected_camera = Some(key.clone());

        let stored = self.settings.camera(&key);
        self.rotation = stored.rotation % 360;
        self.flip_h = stored.flip_h;
        self.flip_v = stored.flip_v;

        // Probe the supported formats (blocks briefly while opening the device).
        let index = self.cameras[self.selected_camera].index.clone();
        match camera::probe_formats(&index) {
            Ok(formats) if !formats.is_empty() => self.formats = formats,
            _ => self.formats = vec![(0, 0, 30)],
        }
        self.rebuild_resolutions(&stored);
        self.start_capture();
        self.save_settings();
    }

    fn rebuild_resolutions(&mut self, stored: &CameraSettings) {
        self.resolutions = self.formats.iter().map(|(w, h, _)| (*w, *h)).collect();
        self.resolutions.dedup();
        self.selected_resolution = stored
            .width
            .zip(stored.height)
            .and_then(|(w, h)| self.resolutions.iter().position(|res| *res == (w, h)))
            .unwrap_or(0);
        self.rebuild_framerates(stored.fps);
    }

    fn rebuild_framerates(&mut self, wanted: Option<u32>) {
        let resolution = self.resolutions.get(self.selected_resolution).copied();
        self.framerates = self
            .formats
            .iter()
            .filter(|(w, h, _)| Some((*w, *h)) == resolution)
            .map(|(_, _, fps)| *fps)
            .collect();
        self.framerates.sort_unstable_by(|a, b| b.cmp(a));
        self.framerates.dedup();
        if self.framerates.is_empty() {
            self.framerates.push(30);
        }
        self.selected_fps = wanted
            .and_then(|fps| self.framerates.iter().position(|value| *value == fps))
            .unwrap_or(0);
    }

    fn transform(&self) -> Transform {
        Transform {
            rotation: self.rotation,
            flip_h: self.flip_h,
            flip_v: self.flip_v,
        }
    }

    fn start_capture(&mut self) {
        self.stop_capture();
        if self.cameras.is_empty() {
            return;
        }
        let (width, height) = self
            .resolutions
            .get(self.selected_resolution)
            .copied()
            .unwrap_or((0, 0));
        let fps = self
            .framerates
            .get(self.selected_fps)
            .copied()
            .unwrap_or(30);
        let index = self.cameras[self.selected_camera].index.clone();
        self.capture = Some(camera::start(index, width, height, fps, self.transform()));
    }

    fn stop_capture(&mut self) {
        if let Some(mut capture) = self.capture.take() {
            capture.stop_and_join();
        }
        self.texture = None;
        self.image_size = None;
        self.actual_resolution = None;
        self.measured_fps = 0.0;
    }

    fn apply_transform(&mut self) {
        if let Some(capture) = &self.capture {
            capture.set_transform(self.transform());
        }
        self.store_camera_settings();
    }

    fn store_camera_settings(&mut self) {
        let Some(camera) = self.cameras.get(self.selected_camera) else {
            return;
        };
        let key = camera.key.clone();
        let (width, height) = self
            .resolutions
            .get(self.selected_resolution)
            .copied()
            .unwrap_or((0, 0));
        let fps = self.framerates.get(self.selected_fps).copied();
        self.settings.set_camera(
            &key,
            CameraSettings {
                rotation: self.rotation,
                flip_h: self.flip_h,
                flip_v: self.flip_v,
                width: Some(width),
                height: Some(height),
                fps,
            },
        );
        self.save_settings();
    }

    fn save_settings(&self) {
        self.settings.save();
    }

    fn set_status(&mut self, message: impl Into<String>) {
        self.status = message.into();
        self.status_until = Some(Instant::now() + std::time::Duration::from_secs(5));
    }

    fn recording(&self) -> bool {
        self.recorder.is_some()
    }

    // ------------------------------------------------------------------
    // Photos and video
    // ------------------------------------------------------------------
    fn save_photo(&mut self) {
        let Some(frame) = self.last_frame.clone() else {
            self.set_status(t("no_image_to_save"));
            return;
        };
        let default_name = format!("microscope_{}.png", util::timestamp());
        let directory = self.settings.image_dir();
        let chosen = rfd::FileDialog::new()
            .set_title(t("save_image_title"))
            .set_directory(&directory)
            .set_file_name(&default_name)
            .add_filter("PNG", &["png"])
            .add_filter("JPEG", &["jpg", "jpeg"])
            .save_file();
        let Some(path) = chosen else { return };
        let path = util::ensure_extension(path, &IMAGE_EXTENSIONS, ".png");

        let image =
            image::RgbImage::from_raw(frame.width as u32, frame.height as u32, frame.rgb.clone());
        let saved = image.map(|img| img.save(&path).is_ok()).unwrap_or(false);
        if saved {
            if let Some(parent) = path.parent() {
                self.settings.last_image_dir = Some(parent.to_string_lossy().to_string());
            }
            self.save_settings();
            self.set_status(format!("{} {}", t("image_saved"), path.display()));
        } else {
            self.set_status(t("save_failed"));
        }
    }

    fn toggle_recording(&mut self) {
        if self.recording() {
            self.stop_recording(false);
        } else {
            self.start_recording();
        }
    }

    fn start_recording(&mut self) {
        let Some(frame) = self.last_frame.clone() else {
            self.set_status(t("no_video_stream"));
            return;
        };
        let default_name = format!("microscope_{}.mp4", util::timestamp());
        let directory = self.settings.video_dir();
        let chosen = rfd::FileDialog::new()
            .set_title(t("save_video_title"))
            .set_directory(&directory)
            .set_file_name(&default_name)
            .add_filter("MP4", &["mp4"])
            .save_file();
        let Some(path) = chosen else { return };
        let path = util::ensure_extension(path, &["mp4"], ".mp4");

        let fps = if self.measured_fps >= 1.0 {
            self.measured_fps.round() as u32
        } else {
            self.framerates
                .get(self.selected_fps)
                .copied()
                .unwrap_or(30)
        };

        match recorder::start(
            path.to_string_lossy().to_string(),
            frame.width,
            frame.height,
            fps,
            self.settings.bitrate_kbps,
        ) {
            Ok((recorder, sender)) => {
                if let Some(capture) = &self.capture {
                    capture.set_recorder(Some(sender));
                }
                self.recorder = Some(recorder);
                self.record_start = Some(Instant::now());
                self.record_frames = 0;
                if let Some(parent) = path.parent() {
                    self.settings.last_video_dir = Some(parent.to_string_lossy().to_string());
                }
                self.save_settings();
                self.set_status(format!("{} {}", t("recording_started"), path.display()));
            }
            Err(error) => {
                self.set_status(format!("{}: {}", t("error_recording"), error));
            }
        }
    }

    fn stop_recording(&mut self, silent: bool) {
        if let Some(capture) = &self.capture {
            capture.set_recorder(None);
        }
        if let Some(mut recorder) = self.recorder.take() {
            let path = recorder.path.clone();
            match recorder.stop_and_join() {
                Ok(stats) => {
                    self.record_frames = stats.frames;
                    if !silent {
                        self.set_status(format!("{} {}", t("video_saved"), path));
                    }
                }
                Err(error) => {
                    if !silent {
                        self.set_status(format!("{}: {}", t("error_recording"), error));
                    }
                }
            }
        }
        self.record_start = None;
    }

    // ------------------------------------------------------------------
    // UI helpers
    // ------------------------------------------------------------------
    fn combo_camera(&mut self, ui: &mut egui::Ui) {
        let labels: Vec<String> = self
            .cameras
            .iter()
            .map(|camera| camera.name.clone())
            .collect();
        let selected_text = labels
            .get(self.selected_camera)
            .cloned()
            .unwrap_or_else(|| t("no_camera").to_string());

        let mut selected = self.selected_camera;
        egui::ComboBox::from_id_salt("camera-combo")
            .selected_text(selected_text)
            .width(210.0)
            .show_ui(ui, |ui| {
                for (index, label) in labels.iter().enumerate() {
                    ui.selectable_value(&mut selected, index, label);
                }
            });
        if selected != self.selected_camera {
            self.selected_camera = selected;
            self.on_camera_changed();
        }
    }

    fn combo_resolution(&mut self, ui: &mut egui::Ui) {
        let labels: Vec<String> = self
            .resolutions
            .iter()
            .map(|(w, h)| {
                if *w == 0 || *h == 0 {
                    "AUTO".to_string()
                } else {
                    format!("{w} x {h}")
                }
            })
            .collect();
        let selected_text = labels
            .get(self.selected_resolution)
            .cloned()
            .unwrap_or_default();
        let mut selected = self.selected_resolution;
        egui::ComboBox::from_id_salt("resolution-combo")
            .selected_text(selected_text)
            .width(130.0)
            .show_ui(ui, |ui| {
                for (index, label) in labels.iter().enumerate() {
                    ui.selectable_value(&mut selected, index, label);
                }
            });
        if selected != self.selected_resolution {
            self.selected_resolution = selected;
            self.rebuild_framerates(None);
            self.start_capture();
            self.store_camera_settings();
        }
    }

    fn combo_fps(&mut self, ui: &mut egui::Ui) {
        let labels: Vec<String> = self.framerates.iter().map(|fps| format!("{fps}")).collect();
        let selected_text = labels.get(self.selected_fps).cloned().unwrap_or_default();
        let mut selected = self.selected_fps;
        egui::ComboBox::from_id_salt("fps-combo")
            .selected_text(selected_text)
            .width(70.0)
            .show_ui(ui, |ui| {
                for (index, label) in labels.iter().enumerate() {
                    ui.selectable_value(&mut selected, index, label);
                }
            });
        if selected != self.selected_fps {
            self.selected_fps = selected;
            self.start_capture();
            self.store_camera_settings();
        }
    }

    fn draw_image(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        let painter = ui.painter();
        painter.rect_filled(rect, egui::CornerRadius::ZERO, Color32::BLACK);
        painter.rect_stroke(
            rect,
            egui::CornerRadius::ZERO,
            egui::Stroke::new(1.0_f32, theme::GREEN_DIM),
            egui::StrokeKind::Inside,
        );

        let Some(image_size) = self.image_size else {
            let blink = ui.input(|input| input.time) * 2.0;
            if (blink as i64) % 2 == 0 {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "NO SIGNAL",
                    egui::FontId::monospace(28.0),
                    theme::GREEN,
                );
            }
            return;
        };

        if response.hovered() {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                self.zoom = (self.zoom * (scroll * 0.0015).exp()).clamp(0.02, 60.0);
                self.fit = false;
            }
        }
        if response.dragged_by(egui::PointerButton::Primary) {
            self.offset += response.drag_delta();
            self.fit = false;
        }
        if response.clicked_by(egui::PointerButton::Secondary) {
            self.fit = true;
        }

        if self.fit {
            let scale = (rect.width() / image_size.x).min(rect.height() / image_size.y);
            self.zoom = scale;
            self.offset = egui::Vec2::ZERO;
        }

        let draw_size = image_size * self.zoom;
        let center = rect.center() + self.offset;
        let image_rect = egui::Rect::from_center_size(center, draw_size);
        if let Some(texture) = &self.texture {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            painter.image(texture.id(), image_rect, uv, Color32::WHITE);
        }
    }

    fn draw_settings(&mut self, ctx: &egui::Context) {
        if !self.show_settings {
            return;
        }
        let mut open = true;
        egui::Window::new(RichText::new(format!("[ {} ]", t("settings"))).color(theme::AMBER))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("-- {} --", t("group_language"))).color(theme::GREEN_DIM),
                );
                let languages = [
                    ("system", t("language_system")),
                    ("en", "English"),
                    ("it", "Italiano"),
                ];
                for (code, label) in languages {
                    if ui
                        .selectable_label(self.settings.language == code, label)
                        .clicked()
                    {
                        self.settings.language = code.to_string();
                        i18n::set_language(code);
                        self.save_settings();
                    }
                }

                ui.separator();
                ui.label(
                    RichText::new(format!("-- {} --", t("group_image"))).color(theme::GREEN_DIM),
                );
                ui.horizontal(|ui| {
                    ui.label(t("rotation_label"));
                    for (label, value) in [("0", 0u16), ("90", 90), ("180", 180), ("270", 270)] {
                        if ui
                            .selectable_label(self.rotation == value, format!("{label}"))
                            .clicked()
                        {
                            self.rotation = value;
                            self.apply_transform();
                        }
                    }
                });
                if ui.checkbox(&mut self.flip_h, t("mirror_h")).changed() {
                    self.apply_transform();
                }
                if ui.checkbox(&mut self.flip_v, t("mirror_v")).changed() {
                    self.apply_transform();
                }

                ui.separator();
                ui.label(
                    RichText::new(format!("-- {} --", t("group_video"))).color(theme::GREEN_DIM),
                );
                ui.horizontal(|ui| {
                    ui.label(t("bitrate_label"));
                    ui.add(egui::Slider::new(
                        &mut self.settings.bitrate_kbps,
                        500..=50000,
                    ));
                });
                ui.label(
                    RichText::new(t("video_hint"))
                        .small()
                        .color(theme::GREEN_DIM),
                );

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(t("reset_camera")).clicked() {
                        self.rotation = 0;
                        self.flip_h = false;
                        self.flip_v = false;
                        self.apply_transform();
                    }
                    if ui.button(t("close")).clicked() {
                        self.show_settings = false;
                    }
                });
            });
        if !open {
            self.show_settings = false;
        }
    }
}

impl eframe::App for MicroscopeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Live preview: pull the newest frame and refresh the texture.
        let (capture_error, captured_fps, captured_opened, captured_frame) = match &self.capture {
            Some(capture) => (
                capture.take_error(),
                capture.measured_fps(),
                capture.take_opened(),
                capture.take_frame(),
            ),
            None => (None, 0.0, None, None),
        };
        if let Some(error) = capture_error {
            self.set_status(format!("{}: {}", t("error_open_camera"), error));
        }
        self.measured_fps = captured_fps;
        if let Some(resolution) = captured_opened {
            self.actual_resolution = Some(resolution);
        }
        if let Some(shared) = captured_frame {
            let image = egui::ColorImage::from_rgb([shared.width, shared.height], &shared.rgb);
            match &mut self.texture {
                Some(texture) => texture.set(image, TextureOptions::LINEAR),
                None => {
                    self.texture = Some(ctx.load_texture("frame", image, TextureOptions::LINEAR));
                    self.fit = true;
                }
            }
            let new_size = egui::vec2(shared.width as f32, shared.height as f32);
            if self.image_size != Some(new_size) {
                self.image_size = Some(new_size);
            }
            self.last_frame = Some(shared);
        }

        // Keyboard shortcuts.
        self.keyboard(ctx);

        let recording = self.recording();
        let status_text = if recording {
            format!("{} {}", t("recording"), self.record_elapsed())
        } else {
            self.status.clone()
        };
        if let Some(until) = self.status_until {
            if Instant::now() > until && !recording {
                self.status = t("ready").to_string();
                self.status_until = None;
            }
        }

        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::default()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::same(6)),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("MICROSCOPE").strong().color(theme::AMBER));
                    ui.separator();
                    ui.label(t("camera"));
                    self.combo_camera(ui);
                    if ui.button(t("refresh")).clicked() {
                        self.refresh_cameras();
                    }
                    ui.separator();
                    ui.label(t("resolution"));
                    self.combo_resolution(ui);
                    ui.label(t("fps"));
                    self.combo_fps(ui);
                    ui.separator();

                    if ui
                        .button(format!("{} {}", t("rotate"), self.rotation))
                        .clicked()
                    {
                        self.rotation = (self.rotation + 90) % 360;
                        self.apply_transform();
                    }
                    if ui.toggle_value(&mut self.flip_h, t("mirror_h")).changed() {
                        self.apply_transform();
                    }
                    if ui.toggle_value(&mut self.flip_v, t("mirror_v")).changed() {
                        self.apply_transform();
                    }
                    ui.separator();

                    if ui.button(t("fit")).clicked() {
                        self.fit = true;
                    }
                    if ui.button(t("photo")).clicked() {
                        self.save_photo();
                    }
                    let record_label = if recording { t("stop") } else { t("record") };
                    if ui.selectable_label(recording, record_label).clicked() {
                        self.toggle_recording();
                    }
                    if ui.button(t("settings")).clicked() {
                        self.show_settings = true;
                    }
                });
            });

        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::default()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::same(4)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let camera_name = self
                        .cameras
                        .get(self.selected_camera)
                        .map(|camera| camera.name.clone())
                        .unwrap_or_else(|| t("no_camera").to_string());
                    ui.label(RichText::new(camera_name).color(theme::GREEN));
                    ui.separator();
                    if let Some((w, h)) = self.actual_resolution {
                        ui.label(RichText::new(format!("{w} x {h}")).color(theme::GREEN_DIM));
                    }
                    ui.label(
                        RichText::new(format!("{:.1} fps", self.measured_fps))
                            .color(theme::GREEN_DIM),
                    );
                    ui.label(
                        RichText::new(format!("{} {:.0}%", t("zoom"), self.zoom * 100.0))
                            .color(theme::GREEN_DIM),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(status_text).color(if recording {
                            theme::RED
                        } else {
                            theme::GREEN
                        }));
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(theme::BG))
            .show(ctx, |ui| {
                self.draw_image(ui);
            });

        self.draw_settings(ctx);
        theme::scanlines(ctx);
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_recording(true);
        self.stop_capture();
        self.save_settings();
    }
}

impl MicroscopeApp {
    fn keyboard(&mut self, ctx: &egui::Context) {
        let (photo, record, fit, zoom_in, zoom_out) = ctx.input(|input| {
            (
                input.modifiers.ctrl && input.key_pressed(egui::Key::S),
                input.modifiers.ctrl && input.key_pressed(egui::Key::R),
                input.key_pressed(egui::Key::F) && !input.modifiers.ctrl,
                input.key_pressed(egui::Key::Plus) || input.key_pressed(egui::Key::Equals),
                input.key_pressed(egui::Key::Minus),
            )
        });
        if photo {
            self.save_photo();
        }
        if record {
            self.toggle_recording();
        }
        if fit {
            self.fit = true;
        }
        if zoom_in {
            self.zoom = (self.zoom * 1.25).clamp(0.02, 60.0);
            self.fit = false;
        }
        if zoom_out {
            self.zoom = (self.zoom / 1.25).clamp(0.02, 60.0);
            self.fit = false;
        }
    }

    fn record_elapsed(&self) -> String {
        let seconds = self
            .record_start
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0);
        let minutes = seconds / 60;
        format!("{:02}:{:02}", minutes, seconds % 60)
    }
}
