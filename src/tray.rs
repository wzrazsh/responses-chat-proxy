use std::sync::atomic::{AtomicBool, Ordering};

use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};
use webbrowser;

static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

pub fn should_exit() -> bool {
    SHOULD_EXIT.load(Ordering::SeqCst)
}

pub fn trigger_exit() {
    SHOULD_EXIT.store(true, Ordering::SeqCst);
}

pub fn setup_tray(bind_addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let open_page_item = MenuItem::with_id("open_page", "Open Test Page", true, None);
    let status_item = MenuItem::with_id("status", &format!("Status: {}", bind_addr), false, None);
    let quit_item = MenuItem::with_id("quit", "Exit Service", true, None);

    let menu = Menu::with_items(&[
        &open_page_item,
        &status_item,
        &PredefinedMenuItem::separator(),
        &quit_item,
    ])?;

    let _tray = TrayIconBuilder::new()
        .with_tooltip("Responses Proxy")
        .with_menu(Box::new(menu))
        .build()?;

    Ok(())
}