use crate::audio::engine::Route;
use tauri::State;

#[tauri::command]
pub async fn add_route(route: Route, state: State<'_, crate::AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock();
    engine.add_route(route).map_err::<String, _>(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_route(input_id: String, output_id: String, state: State<'_, crate::AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock();
    engine.remove_route(&input_id, &output_id).map_err::<String, _>(|e| e.to_string())
}

#[tauri::command]
pub async fn set_gain(device_id: String, gain_db: f32, state: State<'_, crate::AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock();
    engine.set_gain(&device_id, gain_db).map_err::<String, _>(|e| e.to_string())
}

#[tauri::command]
pub async fn start_engine(state: State<'_, crate::AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock();
    engine.start().map_err::<String, _>(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_engine(state: State<'_, crate::AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock();
    engine.stop().map_err::<String, _>(|e| e.to_string())
}
