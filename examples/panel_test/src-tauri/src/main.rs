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

#[tauri::command]
async fn create_overlay_window(
    app: tauri::AppHandle,
    options: WindowOptions,
) -> Result<(), String> {
    
    let window = WebviewWindowBuilder::new(
        &app,
        options.title.clone(),
        WebviewUrl::App("/overlay".into())
    )
    .inner_size(options.width, options.height)
    .position(options.x, options.y)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .focused(false)
    .resizable(false)
    .build()
    .map_err(|e| e.to_string())?;
    println!("Overlay window created successfully");
    
    app.clone().run_on_main_thread(move || {
        let window: WebviewWindow = app.app_handle().get_webview_window(&options.title).unwrap();
        println!("Webview window retrieved successfully");
        let _panel = window.to_overlay_panel().unwrap();
    });
    println!("Overlay window converted to panel on main thread successfully");
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