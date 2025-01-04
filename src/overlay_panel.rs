use bitflags::bitflags;
use cocoa::{
    appkit::{
        NSBackingStoreBuffered, NSView, NSViewHeightSizable, NSViewWidthSizable,
        NSWindowCollectionBehavior, NSWindowStyleMask,
    },
    base::{id, nil, BOOL, NO, YES},
    foundation::NSRect,
};
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{self, Class, Object, Sel},
    sel, sel_impl, Message,
};
use objc_foundation::INSObject;
use objc_id::{Id, ShareId};
use tauri::{Runtime, WebviewWindow};

extern "C" {
    pub fn object_setClass(obj: id, cls: id) -> id;
}

const CLS_NAME: &str = "RawOverlayPanel";

bitflags! {
    struct NSTrackingAreaOptions: u32 {
        const NSTrackingActiveAlways = 0x80;
        const NSTrackingMouseEnteredAndExited = 0x01;
        const NSTrackingMouseMoved = 0x02;
        const NSTrackingCursorUpdate = 0x04;
    }
}

pub struct RawOverlayPanel;

unsafe impl Sync for RawOverlayPanel {}
unsafe impl Send for RawOverlayPanel {}

impl INSObject for RawOverlayPanel {
    fn class() -> &'static runtime::Class {
        Class::get(CLS_NAME).unwrap_or_else(Self::define_class)
    }
}

impl RawOverlayPanel {
    // Override key window behavior to prevent focus stealing
    extern "C" fn can_become_key_window(_: &Object, _: Sel) -> BOOL {
        NO
    }

    extern "C" fn can_become_main_window(_: &Object, _: Sel) -> BOOL {
        NO
    }

    extern "C" fn accepts_mouse_moved_events(_: &Object, _: Sel) -> BOOL {
        YES
    }

    extern "C" fn ignores_mouse_events(_: &Object, _: Sel) -> BOOL {
        NO
    }

    extern "C" fn dealloc(this: &mut Object, _cmd: Sel) {
        unsafe {
            let superclass = class!(NSPanel);
            let dealloc: extern "C" fn(&mut Object, Sel) =
                msg_send![super(this, superclass), dealloc];
            dealloc(this, _cmd);
        }
    }

    fn define_class() -> &'static Class {
        let mut cls = ClassDecl::new(CLS_NAME, class!(NSPanel))
            .unwrap_or_else(|| panic!("Unable to register {} class", CLS_NAME));

        unsafe {
            // Add methods to handle window behavior
            cls.add_method(
                sel!(canBecomeKeyWindow),
                Self::can_become_key_window as extern "C" fn(&Object, Sel) -> BOOL,
            );

            cls.add_method(
                sel!(canBecomeMainWindow),
                Self::can_become_main_window as extern "C" fn(&Object, Sel) -> BOOL,
            );

            cls.add_method(
                sel!(acceptsMouseMovedEvents),
                Self::accepts_mouse_moved_events as extern "C" fn(&Object, Sel) -> BOOL,
            );

            cls.add_method(
                sel!(ignoresMouseEvents),
                Self::ignores_mouse_events as extern "C" fn(&Object, Sel) -> BOOL,
            );

            cls.add_method(
                sel!(dealloc),
                Self::dealloc as extern "C" fn(&mut Object, Sel),
            );
        }

        cls.register()
    }

    pub fn new(frame: NSRect) -> Id<Self> {
        unsafe {
            let panel: id = msg_send![Self::class(), alloc];
            
            const NS_BORDERLESS_WINDOW_MASK: u64 = 0;
            const NS_NONACTIVATING_PANEL_MASK: u64 = 1 << 7;
            let style_mask = NS_BORDERLESS_WINDOW_MASK | NS_NONACTIVATING_PANEL_MASK;

            let panel: id = msg_send![
                panel,
                initWithContentRect:frame
                styleMask:style_mask
                backing:NSBackingStoreBuffered
                defer:NO
            ];

            // Configure panel properties similar to ShapePanel
            let _: () = msg_send![panel, setBackgroundColor: nil];
            let _: () = msg_send![panel, setOpaque: NO];
            let _: () = msg_send![panel, setLevel: 20]; // NSStatusWindowLevel + 1
            let _: () = msg_send![panel, setFloatingPanel: YES];
            let _: () = msg_send![panel, setAcceptsMouseMovedEvents: YES];
            let _: () = msg_send![panel, setIgnoresMouseEvents: NO];
            let _: () = msg_send![panel, setHidesOnDeactivate: NO];
            let _: () = msg_send![panel, setAlphaValue: 1.0];
            let _: () = msg_send![panel, setMovableByWindowBackground: YES];

            Id::from_retained_ptr(panel as *mut Self)
        }
    }

    // Methods from RawNSPanel that we want to keep
    pub fn show(&self) {
        self.order_front_regardless();
    }

    pub fn is_visible(&self) -> bool {
        let flag: BOOL = unsafe { msg_send![self, isVisible] };
        flag == YES
    }

    pub fn order_front_regardless(&self) {
        let _: () = unsafe { msg_send![self, orderFrontRegardless] };
    }

    pub fn order_out(&self, sender: Option<id>) {
        let _: () = unsafe { msg_send![self, orderOut: sender.unwrap_or(nil)] };
    }

    pub fn content_view(&self) -> id {
        unsafe { msg_send![self, contentView] }
    }

    pub fn set_content_view(&self, view: id) {
        let _: () = unsafe { msg_send![self, setContentView: view] };
    }

    pub fn set_level(&self, level: i32) {
        let _: () = unsafe { msg_send![self, setLevel: level] };
    }

    pub fn set_alpha_value(&self, value: f64) {
        let _: () = unsafe { msg_send![self, setAlphaValue: value] };
    }

    pub fn close(&self) {
        let _: () = unsafe { msg_send![self, close] };
    }

    fn add_tracking_area(&self) {
        let view: id = self.content_view();
        let bounds: NSRect = unsafe { NSView::bounds(view) };
        let track_view: id = unsafe { msg_send![class!(NSTrackingArea), alloc] };
        let track_view: id = unsafe {
            msg_send![
                track_view,
                initWithRect: bounds
                options: NSTrackingAreaOptions::NSTrackingActiveAlways.bits()
                    | NSTrackingAreaOptions::NSTrackingMouseEnteredAndExited.bits()
                    | NSTrackingAreaOptions::NSTrackingMouseMoved.bits()
                    | NSTrackingAreaOptions::NSTrackingCursorUpdate.bits()
                owner: view
                userInfo: nil
            ]
        };
        let autoresizing_mask = NSViewWidthSizable | NSViewHeightSizable;
        let () = unsafe { msg_send![view, setAutoresizingMask: autoresizing_mask] };
        let () = unsafe { msg_send![view, addTrackingArea: track_view] };
    }

    pub fn from_window<R: Runtime>(window: WebviewWindow<R>) -> Id<Self> {
        let nswindow: id = window.ns_window().unwrap() as _;
        let nspanel_class: id = unsafe { msg_send![Self::class(), class] };
        unsafe {
            object_setClass(nswindow, nspanel_class);
            let panel = Id::from_retained_ptr(nswindow as *mut RawOverlayPanel);
            
            // Add a tracking area to the panel's content view,
            // so that we can receive mouse events such as mouseEntered and mouseExited
            panel.add_tracking_area();
            
            panel
        }
    }

    pub fn set_frame_origin(&self, x: f64, y: f64) {
        let _: () = unsafe { msg_send![self, setFrameOrigin: (x, y)] };
    }

    pub fn set_frame_size(&self, width: f64, height: f64) {
        let _: () = unsafe { msg_send![self, setFrameSize: (width, height)] };
    }
}

unsafe impl Message for RawOverlayPanel {}