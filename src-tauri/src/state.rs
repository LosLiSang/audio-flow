use crate::audio::engine::AudioEngine;
use std::sync::{Arc, Mutex};
use tauri::State;

pub struct AppState {
    pub engine: Arc<Mutex<AudioEngine>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(AudioEngine::new())),
        }
    }
}
