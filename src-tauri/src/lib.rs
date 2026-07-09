mod app_status;
mod performance_limits;
#[cfg(test)]
mod performance_limits_tests;

use app_status::AppStatus;
use performance_limits::{PerformanceLimitRequest, PerformanceLimits, PerformanceValidation};

#[tauri::command]
fn get_app_status() -> AppStatus {
    AppStatus::pre_alpha()
}

#[tauri::command]
fn get_performance_limits() -> PerformanceLimits {
    PerformanceLimits::default()
}

#[tauri::command]
fn validate_performance_request(request: PerformanceLimitRequest) -> PerformanceValidation {
    let limits = PerformanceLimits::default();

    limits.validate(request).into()
}

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            get_performance_limits,
            validate_performance_request
        ])
        .run(tauri::generate_context!())
}

#[cfg(test)]
mod tests {
    use super::{
        get_app_status, get_performance_limits,
        performance_limits::{PerformanceLimitErrorCode, PerformanceLimitRequest, RegionSize},
        validate_performance_request,
    };

    #[test]
    fn app_status_keeps_capture_and_ai_disabled() {
        let status = get_app_status();

        assert_eq!(status.phase, "pre-alpha");
        assert!(status.scaffold_ready);
        assert!(!status.capture_enabled);
        assert!(!status.ai_enabled);
    }

    #[test]
    fn performance_limits_command_returns_hard_limits() {
        let limits = get_performance_limits();

        assert_eq!(limits.default_fps, 1);
        assert_eq!(limits.max_fps, 5);
        assert_eq!(limits.max_active_tiles, 3);
        assert_eq!(limits.max_region.width, 800);
        assert_eq!(limits.max_region.height, 600);
    }

    #[test]
    fn validate_performance_request_returns_typed_errors() {
        let result = validate_performance_request(PerformanceLimitRequest {
            fps: 6,
            active_tile_count: 1,
            region: RegionSize {
                width: 600,
                height: 300,
            },
        });

        assert!(!result.valid);
        assert_eq!(
            result.error.expect("validation error").code,
            PerformanceLimitErrorCode::FpsTooHigh
        );
    }
}
