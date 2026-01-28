use crate::audio::DeviceInfo;
use tauri::State;

#[tauri::command]
pub async fn list_devices(state: State<'_, crate::AppState>) -> Result<Vec<DeviceInfo>, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let devices = engine.device_manager.list_devices().map_err(|e| e.to_string())?;
    Ok(devices)
}

#[tauri::command]
pub async fn get_peak_levels(state: State<'_, crate::AppState>) -> Result<std::collections::HashMap<String, f32>, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let levels = engine.get_peak_levels().map_err(|e| e.to_string())?;
    Ok(levels)
}
