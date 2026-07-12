use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager,
};

use crate::pebble_session::{self, PebbleSessionState};

const TRAY_ID: &str = "pebble-menu-bar";
const QUIT_MENU_ID: &str = "quit-pebble";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuBarAction {
    Show,
    Quit,
}

pub fn setup(app: &mut App) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;
    let menu = MenuBuilder::new(app).text(QUIT_MENU_ID, "종료").build()?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if let Some(action) = menu_action(event.id().as_ref()) {
                handle_menu_bar_action(app, action);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let Some(action) = tray_action(&event) {
                handle_menu_bar_action(tray.app_handle(), action);
            }
        })
        .build(app)?;

    Ok(())
}

fn tray_action(event: &TrayIconEvent) -> Option<MenuBarAction> {
    match event {
        TrayIconEvent::Click {
            button,
            button_state,
            ..
        } => click_action(*button, *button_state),
        _ => None,
    }
}

fn click_action(button: MouseButton, button_state: MouseButtonState) -> Option<MenuBarAction> {
    match (button, button_state) {
        (MouseButton::Left, MouseButtonState::Up) => Some(MenuBarAction::Show),
        _ => None,
    }
}

fn menu_action(id: &str) -> Option<MenuBarAction> {
    (id == QUIT_MENU_ID).then_some(MenuBarAction::Quit)
}

fn handle_menu_bar_action(app: &AppHandle, action: MenuBarAction) {
    match action {
        MenuBarAction::Show => {
            set_attention(app, false);
            show_pebble(app);
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

#[cfg(test)]
mod tests {
    use super::{click_action, menu_action, MenuBarAction, QUIT_MENU_ID};
    use tauri::tray::{MouseButton, MouseButtonState};

    #[test]
    fn left_click_shows_and_right_click_waits_for_the_menu() {
        assert_eq!(
            click_action(MouseButton::Left, MouseButtonState::Up),
            Some(MenuBarAction::Show)
        );
        assert_eq!(
            click_action(MouseButton::Left, MouseButtonState::Down),
            None
        );
        assert_eq!(
            click_action(MouseButton::Right, MouseButtonState::Down),
            None
        );
        assert_eq!(click_action(MouseButton::Right, MouseButtonState::Up), None);
        assert_eq!(
            click_action(MouseButton::Middle, MouseButtonState::Up),
            None
        );
    }

    #[test]
    fn only_the_explicit_quit_menu_item_exits() {
        assert_eq!(menu_action(QUIT_MENU_ID), Some(MenuBarAction::Quit));
        assert_eq!(menu_action("show"), None);
        assert_eq!(menu_action(""), None);
    }
}
