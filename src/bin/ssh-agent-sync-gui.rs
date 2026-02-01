#![windows_subsystem = "windows"]

use auto_launch::AutoLaunchBuilder;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tray_icon::{
    TrayIconBuilder,
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem},
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    // 1. Setup Auto-Launch logic
    let app_name: &str = "SSHMonitor";
    let app_path = env::current_exe().unwrap().to_str().unwrap().to_string();
    let auto_gui = AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&app_path)
        .build()
        .unwrap();

    // 2. State Management (Thread-safe booleans)
    let is_running = Arc::new(AtomicBool::new(true));
    let is_running_clone = Arc::clone(&is_running);

    // 3. Create Menu Items
    let tray_menu = Menu::new();
    let check_now = MenuItem::new("Check Now", true, None);

    // Toggle for the task itself
    let task_enabled = CheckMenuItem::new("Monitoring Enabled", true, true, None);

    // Toggle for boot start
    let boot_enabled =
        CheckMenuItem::new("Start at Boot", true, auto_gui.is_enabled().unwrap(), None);

    let quit_item = MenuItem::new("Quit", true, None);

    tray_menu
        .append_items(&[&check_now, &task_enabled, &boot_enabled, &quit_item])
        .unwrap();

    // 4. Background Loop
    thread::spawn(move || {
        loop {
            if is_running_clone.load(Ordering::SeqCst) {
                let _ = get_ssh_keys();
            }
            thread::sleep(Duration::from_secs(600));
        }
    });

    // 5. Event Loop (UI Thread)
    let event_loop = EventLoop::builder().build().unwrap();
    let menu_channel = MenuEvent::receiver();

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("SSH Key Monitor")
        // .with_icon(load_icon()) // (Method from previous response)
        .build()
        .unwrap();

    let _ = event_loop.run(move |_event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_item.id() {
                elwt.exit();
            } else if event.id == check_now.id() {
                let _ = get_ssh_keys();
            } else if event.id == task_enabled.id() {
                let state = task_enabled.is_checked();
                is_running.store(state, Ordering::SeqCst);
            } else if event.id == boot_enabled.id() {
                if boot_enabled.is_checked() {
                    auto_gui.enable().unwrap();
                } else {
                    auto_gui.disable().unwrap();
                }
            }
        }
    });
}
