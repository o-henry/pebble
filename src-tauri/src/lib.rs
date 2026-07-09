mod app_status;

use app_status::AppStatus;

#[tauri::command]
fn get_app_status() -> AppStatus {
    AppStatus::pre_alpha()
}

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_app_status])
        .run(tauri::generate_context!())
}

#[cfg(test)]
mod tests {
    use super::get_app_status;

    #[test]
    fn app_status_keeps_capture_and_ai_disabled() {
        let status = get_app_status();

        assert_eq!(status.phase, "pre-alpha");
        assert!(status.scaffold_ready);
        assert!(!status.capture_enabled);
        assert!(!status.ai_enabled);
    }
}
