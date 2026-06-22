mod audio;

use audio::AudioCapture;
use std::sync::Mutex;
use tauri::Manager;

struct AppState {
    audio: Mutex<AudioCapture>,
}

#[tauri::command]
fn start_capture(state: tauri::State<AppState>) -> Result<String, String> {
    let mut audio = state.audio.lock().map_err(|e| e.to_string())?;
    audio.start()?;
    Ok("recording".to_string())
}

#[tauri::command]
async fn stop_and_send(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let wav = {
        let audio = state.audio.lock().map_err(|e| e.to_string())?;
        audio.stop_and_get_wav()?
    };

    let client = reqwest::Client::new();
    let part = reqwest::multipart::Part::bytes(wav)
        .file_name("capture.wav".to_string())
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new().part("file".to_string(), part);

    let resp = client
        .post("http://127.0.0.1:3457/transcribe")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Send error: {}", e))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("JSON error: {}", e))?;

    let text = body["text"].as_str().unwrap_or("").to_string();
    Ok(text)
}

#[tauri::command]
fn toggle_window(window: tauri::Window) {
    if window.is_visible().unwrap_or(false) {
        window.hide().unwrap();
    } else {
        window.show().unwrap();
        window.set_focus().unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            audio: Mutex::new(AudioCapture::new()),
        })
        .invoke_handler(tauri::generate_handler![
            toggle_window,
            start_capture,
            stop_and_send,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
