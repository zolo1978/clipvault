use crate::error::AppError;
use crate::models::MonitorStatus;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn start_monitor(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut monitor = state.monitor.lock().await;
    match monitor.as_mut() {
        Some(svc) => svc.start(),
        None => Err(AppError::Internal("monitor service not initialized".into())),
    }
}

#[tauri::command]
pub async fn stop_monitor(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut monitor = state.monitor.lock().await;
    match monitor.as_mut() {
        Some(svc) => svc.stop(),
        None => Err(AppError::Internal("monitor service not initialized".into())),
    }
}

#[tauri::command]
pub async fn monitor_status(state: State<'_, AppState>) -> Result<MonitorStatus, AppError> {
    let monitor = state.monitor.lock().await;
    match monitor.as_ref() {
        Some(svc) => Ok(svc.status()),
        None => Ok(MonitorStatus {
            is_running: false,
            clips_captured: 0,
        }),
    }
}
