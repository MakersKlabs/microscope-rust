//! Minimal built-in internationalization (English / Italian).

use std::sync::atomic::{AtomicU8, Ordering};

const EN: u8 = 0;
const IT: u8 = 1;

static LANG: AtomicU8 = AtomicU8::new(EN);

/// Returns the language code for the operating system locale.
fn system_language() -> u8 {
    for var in ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(var) {
            let lower = value.to_ascii_lowercase();
            if lower.starts_with("it") {
                return IT;
            }
            if lower.starts_with("en") {
                return EN;
            }
        }
    }
    EN
}

/// Activates a language. `"system"`/`""` follows the OS locale.
pub fn set_language(code: &str) {
    let value = match code {
        "it" => IT,
        "en" => EN,
        _ => system_language(),
    };
    LANG.store(value, Ordering::Relaxed);
}

#[allow(dead_code)]
pub fn current_code() -> &'static str {
    if LANG.load(Ordering::Relaxed) == IT {
        "it"
    } else {
        "en"
    }
}

pub fn is_italian() -> bool {
    LANG.load(Ordering::Relaxed) == IT
}

/// Translate a key into the active language.
pub fn t(key: &str) -> &'static str {
    if is_italian() {
        it(key)
    } else {
        en(key)
    }
}

fn en(key: &str) -> &'static str {
    match key {
        "app_title" => "MICROSCOPE",
        "camera" => "Camera",
        "camera_fallback" => "Camera",
        "resolution" => "Resolution",
        "fps" => "FPS",
        "refresh" => "Refresh",
        "refresh_tip" => "Search again for connected cameras",
        "rotate" => "Rotate",
        "rotate_tip" => "Rotate the image by 90 degrees (clockwise)",
        "mirror_h" => "Mirror H",
        "mirror_h_tip" => "Mirror on the horizontal axis",
        "mirror_v" => "Mirror V",
        "mirror_v_tip" => "Mirror on the vertical axis",
        "fit" => "Fit",
        "fit_tip" => "Fit the image to the window (also right click)",
        "photo" => "Photo",
        "photo_tip" => "Save the current image as PNG",
        "record" => "Record",
        "stop" => "Stop",
        "record_tip" => "Start/stop H.264 video recording",
        "settings" => "Settings",
        "settings_tip" => "Image and video settings",
        "no_camera" => "NO CAMERA",
        "searching" => "SCANNING CAMERAS...",
        "ready" => "READY",
        "recording" => "REC",
        "no_image_to_save" => "NO IMAGE TO SAVE",
        "no_video_stream" => "NO VIDEO STREAM TO RECORD",
        "image_saved" => "IMAGE SAVED",
        "save_failed" => "SAVING FAILED",
        "video_saved" => "VIDEO SAVED",
        "recording_started" => "RECORDING STARTED",
        "save_image_title" => "SAVE IMAGE",
        "save_video_title" => "SAVE VIDEO",
        "group_language" => "LANGUAGE",
        "language_label" => "Language",
        "language_system" => "System",
        "group_image" => "IMAGE",
        "rotation_label" => "Rotation",
        "group_video" => "VIDEO",
        "bitrate_label" => "Bitrate (kbps)",
        "video_hint" => "H.264 (OpenH264) - lower bitrate = smaller files",
        "reset_camera" => "RESET CAMERA",
        "close" => "CLOSE",
        "error_open_camera" => "CANNOT OPEN CAMERA",
        "error_no_frame" => "NO FRAME FROM CAMERA",
        "error_stream" => "VIDEO STREAM INTERRUPTED",
        "error_recording" => "RECORDING ERROR",
        "error_no_camera" => "NO CAMERA SELECTED",
        "zoom" => "ZOOM",
        "credits" => "MICROSCOPE.RUST // 8-BIT VIDEO TERMINAL",
        _ => "???",
    }
}

fn it(key: &str) -> &'static str {
    match key {
        "app_title" => "MICROSCOPIO",
        "camera" => "Camera",
        "camera_fallback" => "Camera",
        "resolution" => "Risoluzione",
        "fps" => "FPS",
        "refresh" => "Aggiorna",
        "refresh_tip" => "Cerca di nuovo le telecamere collegate",
        "rotate" => "Ruota",
        "rotate_tip" => "Ruota l'immagine di 90 gradi (senso orario)",
        "mirror_h" => "Specchia H",
        "mirror_h_tip" => "Specchia sull'asse orizzontale",
        "mirror_v" => "Specchia V",
        "mirror_v_tip" => "Specchia sull'asse verticale",
        "fit" => "Adatta",
        "fit_tip" => "Adatta l'immagine alla finestra (anche click destro)",
        "photo" => "Foto",
        "photo_tip" => "Salva l'immagine corrente in PNG",
        "record" => "Registra",
        "stop" => "Ferma",
        "record_tip" => "Avvia/ferma la registrazione video H.264",
        "settings" => "Impostazioni",
        "settings_tip" => "Impostazioni immagine e video",
        "no_camera" => "NESSUNA TELECAMERA",
        "searching" => "RICERCA TELECAMERE...",
        "ready" => "PRONTO",
        "recording" => "REC",
        "no_image_to_save" => "NESSUNA IMMAGINE DA SALVARE",
        "no_video_stream" => "NESSUN FLUSSO VIDEO DA REGISTRARE",
        "image_saved" => "IMMAGINE SALVATA",
        "save_failed" => "SALVATAGGIO NON RIUSCITO",
        "video_saved" => "VIDEO SALVATO",
        "recording_started" => "REGISTRAZIONE AVVIATA",
        "save_image_title" => "SALVA IMMAGINE",
        "save_video_title" => "SALVA VIDEO",
        "group_language" => "LINGUA",
        "language_label" => "Lingua",
        "language_system" => "Sistema",
        "group_image" => "IMMAGINE",
        "rotation_label" => "Rotazione",
        "group_video" => "VIDEO",
        "bitrate_label" => "Bitrate (kbps)",
        "video_hint" => "H.264 (OpenH264) - bitrate piu' basso = file piu' piccoli",
        "reset_camera" => "RIPRISTINA CAMERA",
        "close" => "CHIUDI",
        "error_open_camera" => "IMPOSSIBILE APRIRE LA CAMERA",
        "error_no_frame" => "NESSUN FRAME DALLA CAMERA",
        "error_stream" => "FLUSSO VIDEO INTERROTTO",
        "error_recording" => "ERRORE DI REGISTRAZIONE",
        "error_no_camera" => "NESSUNA TELECAMERA SELEZIONATA",
        "zoom" => "ZOOM",
        "credits" => "MICROSCOPE.RUST // TERMINALE VIDEO 8-BIT",
        _ => "???",
    }
}
