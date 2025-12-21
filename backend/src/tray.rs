//! System tray icon support for Windows
//!
//! Creates a system tray icon with a context menu for:
//! - Opening the web UI
//! - Quitting the application

#[cfg(windows)]
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};

#[cfg(windows)]
use std::sync::mpsc;

/// Tray icon command sent from the tray thread
#[derive(Debug, Clone)]
pub enum TrayCommand {
    OpenBrowser,
    Quit,
}

/// Initialize and run the system tray icon
/// Returns a receiver for tray commands
#[cfg(windows)]
pub fn init_tray(port: u16) -> Option<mpsc::Receiver<TrayCommand>> {
    use std::thread;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        // Create menu items
        let menu = Menu::new();

        let open_item = MenuItem::new("Open GameVault", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        let open_id = open_item.id().clone();
        let quit_id = quit_item.id().clone();

        menu.append(&open_item).ok();
        menu.append(&quit_item).ok();

        // Create tray icon (using embedded icon or default)
        let icon = load_icon();

        let _tray = match TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(format!("GameVault - localhost:{}", port))
            .with_icon(icon)
            .build()
        {
            Ok(tray) => tray,
            Err(e) => {
                tracing::warn!("Failed to create tray icon: {}", e);
                return;
            }
        };

        tracing::info!("System tray icon initialized");

        // Event loop for menu events
        let menu_receiver = MenuEvent::receiver();
        loop {
            if let Ok(event) = menu_receiver.recv() {
                if event.id == open_id {
                    let _ = tx.send(TrayCommand::OpenBrowser);
                } else if event.id == quit_id {
                    let _ = tx.send(TrayCommand::Quit);
                    break;
                }
            }
        }
    });

    Some(rx)
}

/// Load the application icon for the tray
#[cfg(windows)]
fn load_icon() -> tray_icon::Icon {
    // Create a simple 16x16 purple/accent colored icon
    // This is a placeholder - in production, embed a proper .ico file
    let size = 16u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            // Create a simple game controller-like icon
            // Purple background with lighter center
            let dist_x = (x as i32 - 8).abs();
            let dist_y = (y as i32 - 8).abs();
            let dist = ((dist_x * dist_x + dist_y * dist_y) as f32).sqrt();

            if dist < 6.0 {
                // Inner circle - lighter purple
                rgba.extend_from_slice(&[138, 43, 226, 255]); // Purple
            } else if dist < 7.5 {
                // Border
                rgba.extend_from_slice(&[75, 0, 130, 255]); // Dark purple
            } else {
                // Transparent
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    tray_icon::Icon::from_rgba(rgba, size, size).expect("Failed to create icon")
}

/// Stub for non-Windows platforms
#[cfg(not(windows))]
pub fn init_tray(_port: u16) -> Option<mpsc::Receiver<TrayCommand>> {
    None
}

#[cfg(not(windows))]
use std::sync::mpsc;
