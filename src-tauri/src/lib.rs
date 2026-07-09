mod app_status;
mod performance_limits;
#[cfg(test)]
mod performance_limits_tests;
mod region_selection;
#[cfg(test)]
mod region_selection_tests;
mod region_selection_types;
mod window_shell;
#[cfg(test)]
mod window_shell_tests;

use app_status::AppStatus;
use performance_limits::{PerformanceLimitRequest, PerformanceLimits, PerformanceValidation};
use region_selection_types::{RegionSelection, RegionSelectionIssue, RegionSelectionRequest};
use window_shell::{WindowShellError, WindowShellSnapshot, WindowShellState};

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

#[tauri::command]
fn resolve_region_selection(
    request: RegionSelectionRequest,
) -> Result<RegionSelection, RegionSelectionIssue> {
    region_selection::select_region(request)
}

#[tauri::command]
fn get_window_shell_snapshot(
    state: tauri::State<'_, WindowShellState>,
) -> Result<WindowShellSnapshot, WindowShellError> {
    state.snapshot()
}

#[tauri::command]
async fn open_test_tile_window(
    app: tauri::AppHandle,
    state: tauri::State<'_, WindowShellState>,
) -> Result<window_shell::TileWindowShell, WindowShellError> {
    window_shell::open_test_tile_window(&app, state.inner())
}

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .manage(WindowShellState::default())
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            get_performance_limits,
            validate_performance_request,
            resolve_region_selection,
            get_window_shell_snapshot,
            open_test_tile_window
        ])
        .run(tauri::generate_context!())
}

#[cfg(test)]
mod tests {
    use super::{
        get_app_status, get_performance_limits,
        performance_limits::{PerformanceLimitErrorCode, PerformanceLimitRequest, RegionSize},
        region_selection_types::{
            LogicalPoint, MonitorGeometry, PhysicalPoint, RegionSelectionRequest,
        },
        resolve_region_selection, validate_performance_request,
        window_shell::{TileMode, WindowShellState},
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

    #[test]
    fn resolve_region_selection_returns_physical_region() {
        let selection = resolve_region_selection(RegionSelectionRequest {
            monitor: MonitorGeometry {
                id: "main".to_string(),
                logical_origin: LogicalPoint { x: 0.0, y: 0.0 },
                physical_origin: PhysicalPoint { x: 0, y: 0 },
                scale_factor: 1.0,
            },
            start: LogicalPoint { x: 10.0, y: 20.0 },
            end: LogicalPoint { x: 210.0, y: 170.0 },
        })
        .expect("region selection");

        assert_eq!(selection.region.width, 200);
        assert_eq!(selection.region.height, 150);
        assert!(selection.warnings.is_empty());
    }

    #[test]
    fn window_shell_snapshot_starts_with_closed_test_tile() {
        let state = WindowShellState::default();
        let snapshot = state.snapshot().expect("window shell snapshot");

        assert_eq!(snapshot.test_tile.mode, TileMode::Closed);
        assert!(!snapshot.test_tile.capture_active);
    }
}
