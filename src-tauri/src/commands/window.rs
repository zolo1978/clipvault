use crate::error::AppError;

#[tauri::command]
pub async fn minimize_window(window: tauri::WebviewWindow) -> Result<(), AppError> {
    window
        .minimize()
        .map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn toggle_maximize(window: tauri::WebviewWindow) -> Result<(), AppError> {
    let maximized = window
        .is_maximized()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if maximized {
        window
            .unmaximize()
            .map_err(|e| AppError::Internal(e.to_string()))
    } else {
        window
            .maximize()
            .map_err(|e| AppError::Internal(e.to_string()))
    }
}

#[tauri::command]
pub async fn close_window(window: tauri::WebviewWindow) -> Result<(), AppError> {
    window
        .hide()
        .map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) -> Result<(), AppError> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn start_drag(window: tauri::WebviewWindow) -> Result<(), AppError> {
    window
        .start_dragging()
        .map_err(|e| AppError::Internal(e.to_string()))
}
