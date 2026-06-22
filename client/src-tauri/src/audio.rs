use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::io::Write;

pub struct AudioCapture {
    recording: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl AudioCapture {
    pub fn new() -> Self {
        AudioCapture {
            recording: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "No input device".to_string())?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Config error: {}", e))?;

        println!("Audio device: {}", device.name().unwrap_or("?".to_string()));
        println!("Sample rate: {} Hz", config.sample_rate().0);

        let recording = self.recording.clone();
        recording.store(true, Ordering::SeqCst);
        let buffer = self.buffer.clone();

        let err_fn = |err| eprintln!("Audio err: {}", err);

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if recording.load(Ordering::SeqCst) {
                        let mut buf = buffer.lock().unwrap();
                        buf.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| format!("Stream error: {}", e))?;

        stream.play().map_err(|e| format!("Play error: {}", e))?;
        std::mem::forget(stream);
        Ok(())
    }

    pub fn stop_and_get_wav(&self) -> Result<Vec<u8>, String> {
        self.recording.store(false, Ordering::SeqCst);
        let samples = self.buffer.lock().unwrap().clone();
        
        if samples.is_empty() {
            return Err("No audio captured".to_string());
        }

        let mut wav_bytes = Vec::new();
        let sample_rate = 16000u32;
        let data_len = samples.len() * 2;
        let file_len = 44 + data_len;
        
        wav_bytes.write_all(b"RIFF").unwrap();
        wav_bytes.write_all(&(file_len as u32 - 8).to_le_bytes()).unwrap();
        wav_bytes.write_all(b"WAVE").unwrap();
        wav_bytes.write_all(b"fmt ").unwrap();
        wav_bytes.write_all(&16u32.to_le_bytes()).unwrap();
        wav_bytes.write_all(&1u16.to_le_bytes()).unwrap();
        wav_bytes.write_all(&1u16.to_le_bytes()).unwrap();
        wav_bytes.write_all(&sample_rate.to_le_bytes()).unwrap();
        wav_bytes.write_all(&(sample_rate * 2).to_le_bytes()).unwrap();
        wav_bytes.write_all(&2u16.to_le_bytes()).unwrap();
        wav_bytes.write_all(&16u16.to_le_bytes()).unwrap();
        wav_bytes.write_all(b"data").unwrap();
        wav_bytes.write_all(&(data_len as u32).to_le_bytes()).unwrap();
        
        for &s in &samples {
            let sample = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
            wav_bytes.write_all(&sample.to_le_bytes()).unwrap();
        }

        self.buffer.lock().unwrap().clear();
        Ok(wav_bytes)
    }

    pub fn is_recording(&self) -> bool {
        self.recording.load(Ordering::SeqCst)
    }
}
