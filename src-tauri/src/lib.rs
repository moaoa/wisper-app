use hound::WavReader;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

// Create a state struct to hold the WhisperContext
pub struct AppState {
    whisper_context: Arc<Mutex<WhisperContext>>,
}

#[tauri::command]
async fn transcribe_rs(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    println!("RUST: Audio path: {}", path);

    // Read the audio file
    let reader = WavReader::open(&path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    println!("RUST: Audio spec: {:?}", spec);

    // Ensure the audio is 16kHz mono
    if spec.sample_rate != 16000 || spec.channels != 1 {
        return Err(format!(
            "Audio must be 16kHz mono, but got {}kHz with {} channels.",
            spec.sample_rate, spec.channels
        ));
    }

    // Convert samples to f32
    let samples: Vec<i16> = reader
        .into_samples::<i16>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut audio_samples: Vec<f32> = Vec::with_capacity(samples.len());
    for sample in samples {
        audio_samples.push(sample as f32 / 32768.0);
    }

    println!("RUST: Loaded {} audio samples.", audio_samples.len());

    // Lock the context and run transcription
    let whisper_context = state.whisper_context.lock().unwrap();

    let mut state = whisper_context.create_state().map_err(|e| e.to_string())?;

    let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    state
        .full(params, &audio_samples)
        .map_err(|e| e.to_string())?;

    let num_segments = state.full_n_segments();
    println!("RUST: Transcription produced {} segments.", num_segments);

    let result = (0..num_segments)
        .filter_map(|i| state.get_segment(i))
        .collect::<String>();

    println!("RUST: Transcription result: {}", result);
    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let model_path = app_handle
                .path()
                .resolve("models/ggml-tiny.en.bin", tauri::path::BaseDirectory::Resource)
                .expect("Failed to resolve model path");

            println!("RUST: Loading model from: {:?}", model_path);

            let context =
                WhisperContext::new_with_params(&model_path.to_string_lossy(), WhisperContextParameters::default())
                    .expect("failed to load model");

            let state = AppState {
                whisper_context: Arc::new(Mutex::new(context)),
            };
            app.manage(state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_mic_recorder::init())
        .invoke_handler(tauri::generate_handler![transcribe_rs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
