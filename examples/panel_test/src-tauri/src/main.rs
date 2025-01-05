#![cfg_attr(
    all(not(debug_assertions), target_os = "macos"),
    windows_subsystem = "macos"
)]

use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder, WebviewUrl};
use tauri_nspanel::{
    cocoa::{
        appkit::{
            NSMainMenuWindowLevel, NSWindowCollectionBehavior, NSWindow,
            NSColor,
        },
        base::{id, nil, NO, YES},
    },
    objc::{msg_send, sel, sel_impl, class, declare::ClassDecl, runtime::{Object, Sel}},
    panel_delegate, ManagerExt, WebviewWindowExt as PanelWebviewWindowExt,
};


#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
struct WindowOptions {
    title: String,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
}

use tauri_nspanel::cocoa;

#[tauri::command]
async fn create_overlay_window(
  app: tauri::AppHandle,
  options: WindowOptions,
) -> Result<(), String> {
  app.set_activation_policy(tauri::ActivationPolicy::Prohibited);
  let label = options.title.clone();
  app.clone().run_on_main_thread(move || {
    println!("Overlay window creating on main thread");
    // 1. Build the Tauri window while invisible

    let window = WebviewWindowBuilder::new(
        &app,
        options.title.clone(),
        WebviewUrl::App("/overlay".into())
    )
    .inner_size(options.width, options.height)
    .position(options.x, options.y)
    .decorations(false)
    .focused(false)
    .transparent(true)
    .resizable(false)
    .always_on_top(true)
    .visible(false) // start hidden
    .build()
    .expect("Failed to build window");

    // 2. Convert to overlay panel
    
    let window: WebviewWindow = app.app_handle().get_webview_window(&options.title).unwrap();
    println!("Webview window retrieved successfully");
    let panel = window.to_overlay_panel().unwrap();
    
    // 3. Show as non-activating
    panel.order_front_regardless();
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    // 4. If you want to be extra sure, deactivate the app:
    unsafe {
      use cocoa::appkit::NSApp;
      let ns_app = cocoa::appkit::NSApp();
      let _: () = msg_send![ns_app, deactivate];
    }
  });

  Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_nspanel::init())
        .invoke_handler(tauri::generate_handler![
            show_panel,
            hide_panel,
            close_panel,
            create_overlay_window,
            show_overlay,
            hide_overlay,
            close_overlay
        ])
        .setup(|app| {            
            // Set activation policy to Accessory to prevent the app icon from showing on the dock
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Create the main window using WebviewWindowBuilder
            let main_window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("/".into())
            )
            .title("Overlays")
            .inner_size(128.0, 200.0)
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .always_on_top(true)
            .center()
            .focused(false)
            .visible(false)
            .build()?;

            println!("Main window created successfully");

            // Initialize panel settings
            init(app.app_handle());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init(app_handle: &AppHandle) {
    let window: WebviewWindow = app_handle.get_webview_window("main").unwrap();
    let panel = window.to_overlay_panel().unwrap();    

    println!("Panel created successfully");
    panel.order_front_regardless();
}

#[tauri::command]
fn show_panel(handle: AppHandle) {
    let panel = handle.get_webview_panel("main").unwrap();
    panel.show();
}

#[tauri::command]
fn hide_panel(handle: AppHandle) {
    let panel = handle.get_webview_panel("main").unwrap();
    panel.order_out(None);
}

#[tauri::command]
fn close_panel(handle: AppHandle) {
    let panel = handle.get_webview_panel("main").unwrap();
    panel.released_when_closed(true);
    panel.close();
}

#[tauri::command]
fn show_overlay(handle: AppHandle, label: &str) {
    if let Ok(panel) = handle.get_webview_overlay_panel(label) {
        panel.show();
    }
}

#[tauri::command]
fn hide_overlay(handle: AppHandle, label: &str) {
    if let Ok(panel) = handle.get_webview_overlay_panel(label) {
        panel.order_out(None);
    }
}

#[tauri::command]
fn close_overlay(handle: AppHandle, label: &str) {
    if let Ok(panel) = handle.get_webview_overlay_panel(label) {
        panel.released_when_closed(true);
        panel.close();
    }
}