use crate::region_selector_window::{RegionSelectorWindowShell, REGION_SELECTOR_LABEL};

#[test]
fn region_selector_shell_is_transparent_and_capture_free() {
    let shell = RegionSelectorWindowShell::transparent_overlay();

    assert_eq!(shell.label, REGION_SELECTOR_LABEL);
    assert!(shell.visual_overlay);
    assert!(shell.native_transparent);
    assert!(shell.always_on_top);
    assert!(!shell.capture_active);
}
