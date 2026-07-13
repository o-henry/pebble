use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager,
};

use crate::pebble_session::{self, PebbleSessionState};

const TRAY_ID: &str = "pebble-menu-bar";
const QUIT_MENU_ID: &str = "quit-pebble";
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray-icon.png");
const TRAY_ALERT_ICON: &[u8] = include_bytes!("../icons/tray-icon-alert.png");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuBarAction {
    Show,
    Quit,
}

pub fn setup(app: &mut App) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let icon = tauri::image::Image::from_bytes(TRAY_ICON)?;
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
    let bytes = if active { TRAY_ALERT_ICON } else { TRAY_ICON };
    if let Ok(icon) = tauri::image::Image::from_bytes(bytes) {
        let _ = tray.set_icon_with_as_template(Some(icon), !active);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{click_action, menu_action, MenuBarAction, QUIT_MENU_ID, TRAY_ICON};
    use tauri::tray::{MouseButton, MouseButtonState};

    #[test]
    fn normal_tray_icon_is_a_centered_circle() {
        let decoder = png::Decoder::new(Cursor::new(TRAY_ICON));
        let mut reader = decoder.read_info().expect("tray icon metadata");
        let mut pixels = vec![0; reader.output_buffer_size().expect("tray icon buffer size")];
        let info = reader.next_frame(&mut pixels).expect("tray icon pixels");

        assert_eq!((info.width, info.height), (64, 64));
        assert_eq!(info.color_type, png::ColorType::Rgba);

        let mut bounds = (info.width, info.height, 0, 0);
        for y in 0..info.height {
            for x in 0..info.width {
                let alpha = pixels[((y * info.width + x) * 4 + 3) as usize];
                if alpha > 0 {
                    bounds.0 = bounds.0.min(x);
                    bounds.1 = bounds.1.min(y);
                    bounds.2 = bounds.2.max(x);
                    bounds.3 = bounds.3.max(y);
                }
            }
        }

        assert_eq!(bounds, (8, 8, 55, 55));
        let center_alpha = pixels[((32 * info.width + 32) * 4 + 3) as usize];
        assert_eq!(center_alpha, 0);
    }

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
