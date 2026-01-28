use crate::audio::DeviceInfo;
use tauri::State;
use std::collections::HashMap;

#[tauri::command]
pub async fn list_devices(state: State<'_, crate::AppState>) -> Result<Vec<DeviceInfo>, String> {
    let engine = state.engine.lock();
    let devices = engine.device_manager.list_devices().map_err::<String, _>(|e| e.to_string())?;
    Ok(devices)
}

#[tauri::command]
pub async fn get_peak_levels(state: State<'_, crate::AppState>) -> Result<HashMap<String, f32>, String> {
    let engine = state.engine.lock();
    Ok(engine.get_peak_levels())
}
