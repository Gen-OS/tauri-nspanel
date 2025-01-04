mod macros;
pub mod raw_nspanel;
pub mod raw_overlaypanel;

use std::{collections::HashMap, sync::Mutex};

use cocoa::base::id;
use objc_id::ShareId;
use raw_nspanel::RawNSPanel;
use raw_overlaypanel::RawOverlayPanel;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime, WebviewWindow,
};

pub extern crate block;
pub extern crate cocoa;
pub extern crate objc;
pub extern crate objc_foundation;
pub extern crate objc_id;
pub extern crate tauri;

pub type Panel = ShareId<RawNSPanel>;
pub type OverlayPanel = ShareId<RawOverlayPanel>;

#[derive(Default)]
pub struct Store {
    panels: HashMap<String, ShareId<RawNSPanel>>,
}

#[derive(Default)]
pub struct OverlayPanelStore {
    panels: HashMap<String, ShareId<RawOverlayPanel>>,
}


pub struct WebviewPanelManager(pub Mutex<Store>);

impl Default for WebviewPanelManager {
    fn default() -> Self {
        Self(Mutex::new(Store::default()))
    }
}

pub struct OverlayPanelManager(pub Mutex<OverlayPanelStore>);

impl Default for OverlayPanelManager {
    fn default() -> Self {
        Self(Mutex::new(OverlayPanelStore::default()))
    }
}

pub trait ManagerExt<R: Runtime> {
    fn get_webview_panel(&self, label: &str) -> Result<ShareId<RawNSPanel>, Error>;
    fn get_webview_overlay_panel(&self, label: &str) -> Result<ShareId<RawOverlayPanel>, Error>;
}

#[derive(Debug)]
pub enum Error {
    PanelNotFound,
}

impl<R: Runtime, T: Manager<R>> ManagerExt<R> for T {
    fn get_webview_panel(&self, label: &str) -> Result<ShareId<RawNSPanel>, Error> {
        let manager = self.state::<self::WebviewPanelManager>();
        let manager = manager.0.lock().unwrap();

        match manager.panels.get(label) {
            Some(panel) => Ok(panel.clone()),
            None => Err(Error::PanelNotFound),
        }
    }

    fn get_webview_overlay_panel(&self, label: &str) -> Result<ShareId<RawOverlayPanel>, Error> {
        let manager = self.state::<self::OverlayPanelManager>();
        let manager = manager.0.lock().unwrap();

        match manager.panels.get(label) {
            Some(panel) => Ok(panel.clone()),
            None => Err(Error::PanelNotFound),
        }
    }
}

#[derive(Default)]
pub struct WebviewPanelConfig {
    pub delegate: Option<id>,
}

pub trait WebviewWindowExt<R: Runtime> {
    fn to_panel(&self) -> tauri::Result<ShareId<RawNSPanel>>;
    fn to_overlay_panel(&self) -> tauri::Result<ShareId<RawOverlayPanel>>;
}

impl<R: Runtime> WebviewWindowExt<R> for WebviewWindow<R> {
    fn to_panel(&self) -> tauri::Result<ShareId<RawNSPanel>> {
        let panel = RawNSPanel::from_window(self.to_owned());
        let shared_panel = panel.share();
        let manager = self.state::<self::WebviewPanelManager>();

        manager
            .0
            .lock()
            .unwrap()
            .panels
            .insert(self.label().into(), shared_panel.clone());

        Ok(shared_panel)
    }
    fn to_overlay_panel(&self) -> tauri::Result<ShareId<RawOverlayPanel>> {
        let panel = RawOverlayPanel::from_window(self.to_owned());
        let shared_panel = panel.share();
        let manager = self.state::<self::OverlayPanelManager>();

        manager
            .0
            .lock()
            .unwrap()
            .panels
            .insert(self.label().into(), shared_panel.clone());

        Ok(shared_panel)
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("nspanel")
        .setup(|app, _api| {
            app.manage(self::WebviewPanelManager::default());
            app.manage(self::OverlayPanelManager::default());
            Ok(())
        })
        .build()
}