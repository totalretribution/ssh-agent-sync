#![windows_subsystem = "windows"]

use auto_launch::AutoLaunchBuilder;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;
use tray_icon::{
    TrayIconBuilder,
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem},
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};

use ssh_agent_sync::add_keys_to_config;
use ssh_agent_sync::constants;
use ssh_agent_sync::get_ssh_keys;

use rust_embed::Embed;
#[cfg(target_os = "linux")]
use gtk;

#[derive(Embed)]
#[folder = "assets/"]
struct Asset;

struct SyncGuard {
    flag: Arc<AtomicBool>,
}

impl SyncGuard {
    fn try_acquire(flag: &Arc<AtomicBool>) -> Option<Self> {
        if flag
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            Some(Self {
                flag: Arc::clone(flag),
            })
        } else {
            None
        }
    }
}

impl Drop for SyncGuard {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

/// UI commands sent from background threads to the UI thread
enum UiCommand {
    PerformingSync(bool),
}

#[allow(dead_code)]
fn load_icon_from_path(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

#[allow(dead_code)]
fn load_icon_embedded(name: &str) -> tray_icon::Icon {
    let asset = Asset::get(name).expect("Failed to get embedded icon");

    let image = image::load_from_memory(&asset.data)
        .expect("Failed to decode embedded icon")
        .into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    tray_icon::Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon")
}

fn sync_ssh(in_progress: &Arc<AtomicBool>, ui_tx: Option<&Sender<UiCommand>>) {
    if let Some(_guard) = SyncGuard::try_acquire(in_progress) {
        // notify UI to disable "Check Now" while running
        if let Some(tx) = ui_tx {
            let _ = tx.send(UiCommand::PerformingSync(true));
        }

        let mut keys = get_ssh_keys().unwrap_or_default();
        if let Err(e) = add_keys_to_config(&mut keys, false) {
            eprintln!("Failed to add keys to config: {}", e);
        }

        // notify UI to re-enable it after completion
        if let Some(tx) = ui_tx {
            let _ = tx.send(UiCommand::PerformingSync(false));
        }
    } else {
        eprintln!("sync_ssh skipped: already in progress");
    }
}

fn main() {
    let app_path = env::current_exe().unwrap().to_str().unwrap().to_string();
    let auto_gui = AutoLaunchBuilder::new()
        .set_app_name(crate::constants::PROGRAM_NAME)
        .set_app_path(&app_path)
        .build()
        .unwrap();

    // 2. State Management (Thread-safe booleans)
    let is_running = Arc::new(AtomicBool::new(true));
    let is_running_clone = Arc::clone(&is_running);

    // Prevent concurrent runs of sync_ssh
    let sync_in_progress = Arc::new(AtomicBool::new(false));
    let sync_in_progress_clone = Arc::clone(&sync_in_progress);

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

    // Channel for UI commands (e.g., enable/disable menu items)
    let (ui_cmd_tx, ui_cmd_rx) = channel::<UiCommand>();
    let ui_cmd_tx_clone = ui_cmd_tx.clone();

    // 4. Background Loop
    thread::spawn(move || {
        loop {
            if is_running_clone.load(Ordering::SeqCst) {
                sync_ssh(&sync_in_progress_clone, Some(&ui_cmd_tx_clone));
            }
            thread::sleep(Duration::from_secs(600));
        }
    });

    // 5. Event Loop (UI Thread)
    let event_loop = EventLoop::builder().build().unwrap();
    let menu_channel = MenuEvent::receiver();

    // let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.png");

    #[cfg(target_os = "linux")]
    {
        if let Err(e) = gtk::init() {
            eprintln!("Warning: failed to initialize GTK: {:?}", e);
        }
    }

    let icon = load_icon_embedded("icon.png");
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(crate::constants::PROGRAM_NAME)
        .with_icon(icon)
        .build()
        .unwrap();

    struct App {
        menu_channel: tray_icon::menu::MenuEventReceiver,
        quit_item: MenuItem,
        check_now: MenuItem,
        task_enabled: CheckMenuItem,
        boot_enabled: CheckMenuItem,
        is_running: Arc<AtomicBool>,
        sync_in_progress: Arc<AtomicBool>,
        ui_cmd_tx: Sender<UiCommand>,
        ui_cmd_rx: Receiver<UiCommand>,
        auto_gui: auto_launch::AutoLaunch,
    }

    impl ApplicationHandler for App {
        fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _id: winit::window::WindowId,
            _event: WindowEvent,
        ) {
            event_loop.set_control_flow(ControlFlow::Wait);
        }

        fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
            event_loop.set_control_flow(ControlFlow::Wait);

            // Process UI commands from background threads (e.g., enable/disable menu items)
            while let Ok(cmd) = self.ui_cmd_rx.try_recv() {
                match cmd {
                    UiCommand::PerformingSync(enabled) => {
                        let _ = self.check_now.set_enabled(!enabled);
                    }
                }
            }

            if let Ok(event) = self.menu_channel.try_recv() {
                if event.id == self.quit_item.id() {
                    event_loop.exit();
                } else if event.id == self.check_now.id() {
                    // disable immediately to prevent re-clicks while syncing
                    self.check_now.set_enabled(false);
                    sync_ssh(&self.sync_in_progress, Some(&self.ui_cmd_tx));
                } else if event.id == self.task_enabled.id() {
                    let state = self.task_enabled.is_checked();
                    self.is_running.store(state, Ordering::SeqCst);
                } else if event.id == self.boot_enabled.id() {
                    if self.boot_enabled.is_checked() {
                        self.auto_gui.enable().unwrap();
                    } else {
                        self.auto_gui.disable().unwrap();
                    }
                }
            }
        }
    }

    let mut app = App {
        menu_channel: menu_channel.clone(),
        quit_item,
        check_now,
        task_enabled,
        boot_enabled,
        is_running: Arc::clone(&is_running),
        sync_in_progress: Arc::clone(&sync_in_progress),
        ui_cmd_tx: ui_cmd_tx.clone(),
        ui_cmd_rx,
        auto_gui,
    };

    let _ = event_loop.run_app(&mut app);
}
