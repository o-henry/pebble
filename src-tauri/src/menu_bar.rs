use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager,
};

use crate::{
    pebble_session::{self, PebbleSessionState},
    platform_capture, region_selector_window,
};

const SELECT_REGION_ID: &str = "select-region";
const QUIT_PEBBLE_ID: &str = "quit-pebble";
const TRAY_ID: &str = "pebble-menu-bar";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuBarAction {
    SelectRegion,
    Quit,
}

pub fn setup(app: &mut App) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let select_region = MenuItem::with_id(
        app,
        SELECT_REGION_ID,
        "SELECT REGION...",
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, QUIT_PEBBLE_ID, "QUIT", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&select_region, &separator, &quit])?;
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if is_primary_click(&event) {
                set_attention(tray.app_handle(), false);
                show_pebble(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| {
            if let Some(action) = menu_bar_action(event.id().as_ref()) {
                handle_menu_bar_action(app, action);
            }
        })
        .build(app)?;

    Ok(())
}

fn menu_bar_action(id: &str) -> Option<MenuBarAction> {
    match id {
        SELECT_REGION_ID => Some(MenuBarAction::SelectRegion),
        QUIT_PEBBLE_ID => Some(MenuBarAction::Quit),
        _ => None,
    }
}

fn handle_menu_bar_action(app: &AppHandle, action: MenuBarAction) {
    match action {
        MenuBarAction::SelectRegion => {
            if platform_capture::request_screen_capture_access() {
                let _ = region_selector_window::open_region_selector_window(app, None);
            } else {
                show_pebble(app);
            }
        }
        MenuBarAction::Quit => app.exit(0),
    }
}

fn show_pebble(app: &AppHandle) {
    let state = app.state::<PebbleSessionState>();
    let _ = pebble_session::show_pebble_shell(app, state.inner());
}

pub fn set_attention(app: &AppHandle, active: bool) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let bytes = if active {
        include_bytes!("../icons/tray-icon-alert.png").as_slice()
    } else {
        include_bytes!("../icons/tray-icon.png").as_slice()
    };
    if let Ok(icon) = tauri::image::Image::from_bytes(bytes) {
        let _ = tray.set_icon_with_as_template(Some(icon), !active);
    }
}

fn is_primary_click(event: &TrayIconEvent) -> bool {
    matches!(
        event,
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use super::{menu_bar_action, MenuBarAction};

    #[test]
    fn accepts_only_known_menu_actions() {
        assert_eq!(
            menu_bar_action("select-region"),
            Some(MenuBarAction::SelectRegion)
        );
        assert_eq!(menu_bar_action("quit-pebble"), Some(MenuBarAction::Quit));
        assert_eq!(menu_bar_action("show-pebble"), None);
        assert_eq!(menu_bar_action("open-url"), None);
    }
}
