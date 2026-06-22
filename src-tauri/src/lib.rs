mod ai;
mod analytics;
mod core;
mod database;
mod utils;

use ai::settings::{AiSettingsState, get_ai_settings, save_ai_settings, test_ai_connection};
use ai::service::invoke_llm;
use ai::task_decomposition::decompose_task;
use analytics::service::{
    export_analytics_report, get_analytics_events, get_analytics_summary, get_analytics_trend,
    get_testing_mode, set_testing_mode, show_analytics_window, track_event_command,
};
use core::{
    device::start_device_listening,
    gamepad::{start_gamepad_listing, stop_gamepad_listing},
    prevent_default, setup,
};
use database::{commands::*, db::Database};
use database::daily_report::{generate_daily_report, get_daily_reports};
use tauri::{Manager, WindowEvent, generate_handler};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_custom_window::{
    MAIN_WINDOW_LABEL, PREFERENCE_WINDOW_LABEL, show_preference_window,
};
use utils::fs_extra::copy_dir;

use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();

            let main_window = app.get_webview_window(MAIN_WINDOW_LABEL).unwrap();

            let preference_window = app.get_webview_window(PREFERENCE_WINDOW_LABEL).unwrap();

            setup::default(&app_handle, main_window.clone(), preference_window.clone());

            // Initialize database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");

            let database = Database::new(app_data_dir).expect("failed to initialize database");

            // Load AI settings from DB into managed state
            let ai_settings = {
                let conn = database.conn.lock().unwrap();
                let result = conn.query_row(
                    "SELECT id, provider, model_name, api_key, base_url, is_active FROM ai_settings WHERE is_active = 1 LIMIT 1",
                    [],
                    |row| {
                        Ok(ai::settings::AiSettings {
                            id: row.get(0)?,
                            provider: row.get(1)?,
                            model_name: row.get(2)?,
                            api_key: row.get(3)?,
                            base_url: row.get(4)?,
                            is_active: row.get(5)?,
                        })
                    },
                );
                result.unwrap_or_default()
            };

            app.manage(database);
            app.manage(AiSettingsState(Mutex::new(ai_settings)));

            Ok(())
        })
        .invoke_handler(generate_handler![
            copy_dir,
            start_device_listening,
            start_gamepad_listing,
            stop_gamepad_listing,
            create_task,
            update_task,
            delete_task,
            complete_task,
            get_today_tasks,
            get_task_history,
            get_activity_logs,
            decompose_task,
            invoke_llm,
            get_ai_settings,
            save_ai_settings,
            test_ai_connection,
            generate_daily_report,
            get_daily_reports,
            track_event_command,
            get_analytics_summary,
            get_analytics_trend,
            get_analytics_events,
            export_analytics_report,
            get_testing_mode,
            set_testing_mode,
            show_analytics_window,
        ])
        .plugin(tauri_plugin_admin_status::init())
        .plugin(tauri_plugin_custom_window::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_pinia::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(prevent_default::init())
        .plugin(tauri_plugin_single_instance::init(
            |app_handle, _argv, _cwd| {
                show_preference_window(app_handle);
            },
        ))
        .plugin(
            tauri_plugin_log::Builder::new()
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .filter(|metadata| !metadata.target().contains("gilrs"))
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_locale::init())
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();

                api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, event| match event {
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen { .. } => {
            show_preference_window(app_handle);
        }
        _ => {
            let _ = app_handle;
        }
    });
}


