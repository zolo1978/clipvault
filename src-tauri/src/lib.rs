mod commands;
mod error;
mod models;
mod repositories;
mod services;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = AppState::default_db_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let app_state = AppState::new(db_path.to_str().unwrap_or("clipvault.db"))
        .expect("failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed
                        && shortcut == &tauri_plugin_global_shortcut::Shortcut::new(Some(tauri_plugin_global_shortcut::Modifiers::SUPER | tauri_plugin_global_shortcut::Modifiers::SHIFT), tauri_plugin_global_shortcut::Code::KeyV)
                    {
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .manage(app_state)
        .setup(|app| {
            use tauri_plugin_global_shortcut::GlobalShortcutExt;

            let shortcut =
                tauri_plugin_global_shortcut::Shortcut::new(
                    Some(
                        tauri_plugin_global_shortcut::Modifiers::SUPER
                            | tauri_plugin_global_shortcut::Modifiers::SHIFT,
                    ),
                    tauri_plugin_global_shortcut::Code::KeyV,
                );
            app.global_shortcut()
                .register(shortcut)
                .expect("failed to register global shortcut");

            // System tray
            let show_item =
                tauri::menu::MenuItemBuilder::with_id("show", "显示面板").build(app)?;
            let pause_item =
                tauri::menu::MenuItemBuilder::with_id("pause", "暂停监控").build(app)?;
            let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = tauri::menu::MenuBuilder::new(app)
                .items(&[&show_item, &pause_item, &quit_item])
                .build()?;

            tauri::tray::TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "pause" => {
                        // TODO: toggle monitor via state
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Window close → hide
            if let Some(window) = app.get_webview_window("main") {
                let w: tauri::WebviewWindow = window;
                let w2 = w.clone();
                w.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w2.hide();
                    }
                });
            }

            // Initialize and auto-start monitor service
            let handle = app.handle().clone();
            let state = app.state::<AppState>();
            let db = state.db.clone();
            let config = state.config.clone();
            let mut monitor = state.monitor.blocking_lock();
            let mut svc = services::monitor_service::MonitorService::new(
                db, handle, config,
            );
            svc.start().expect("failed to start clipboard monitor");
            *monitor = Some(svc);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_clips,
            commands::search_clips,
            commands::get_clip,
            commands::delete_clip,
            commands::delete_clips,
            commands::toggle_favorite,
            commands::purge_clips,
            commands::get_config,
            commands::update_config,
            commands::clipboard::create_clip,
            commands::clipboard::paste_clip,
            commands::clipboard::view_image_clip,
            commands::clipboard::reveal_path,
            commands::monitor::start_monitor,
            commands::monitor::stop_monitor,
            commands::monitor::monitor_status,
            commands::window::minimize_window,
            commands::window::toggle_maximize,
            commands::window::close_window,
            commands::window::quit_app,
            commands::window::start_drag,
            commands::screenshot::snip_screen,
        ])
        .run(tauri::generate_context!())
        .expect("failed to launch ClipVault");
}
