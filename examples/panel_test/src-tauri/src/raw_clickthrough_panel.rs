use tauri_nspanel::{
    cocoa::{
        appkit::{NSMainMenuWindowLevel, NSWindowCollectionBehavior, NSPanel, NSWindow},
        base::{id, nil, NO, YES},
        foundation::NSRect
    },
    objc::{runtime::{Class, Object, Sel}, declare::ClassDecl, Message, class, msg_send, sel, sel_impl},
    
    objc_id::ShareId
};
use tauri::WebviewWindow;
use std::sync::Once;

const CLASS_NAME: &str = "ClickThroughPanel";

pub fn ensure_click_panel_class() -> &'static Class {
    unsafe {
        // First check if the class already exists
        if let Some(class) = Class::get(CLASS_NAME) {
            return class;
        }

        // If not, create it
        let superclass = class!(NSPanel);
        let mut decl = ClassDecl::new(CLASS_NAME, superclass).unwrap();
        
        // Override canBecomeKeyWindow to return NO
        extern "C" fn can_become_key_window(_this: &Object, _sel: Sel) -> bool {
            false
        }
        decl.add_method(sel!(canBecomeKeyWindow),
            can_become_key_window as extern "C" fn(&Object, Sel) -> bool);

        // Override canBecomeMainWindow to return NO
        extern "C" fn can_become_main_window(_this: &Object, _sel: Sel) -> bool {
            false
        }
        decl.add_method(sel!(canBecomeMainWindow),
            can_become_main_window as extern "C" fn(&Object, Sel) -> bool);

        // Override acceptsFirstMouse to return YES
        extern "C" fn accepts_first_mouse(_this: &Object, _sel: Sel, _event: id) -> bool {
            true
        }
        decl.add_method(sel!(acceptsFirstMouse:),
            accepts_first_mouse as extern "C" fn(&Object, Sel, id) -> bool);
            
        decl.register()
    }
}

pub struct RawClickThroughPanel(pub id);

unsafe impl Message for RawClickThroughPanel {}

impl RawClickThroughPanel {
    pub fn from_window(window: &WebviewWindow) -> Self {
        unsafe {
            let ns_window: id = window.ns_window().unwrap() as _;
            let frame: NSRect = msg_send![ns_window, frame];
            let content_view: id = msg_send![ns_window, contentView];
            
            // Get our custom class
            let panel_class = ensure_click_panel_class();
            
            // Verify the class exists and is registered
            assert!(Class::get(CLASS_NAME).is_some(), "Failed to find ClickThroughPanel class");
            
            let panel: id = msg_send![panel_class, alloc];
            let panel: id = msg_send![panel,
                initWithContentRect:frame
                styleMask:0  // NSBorderlessWindowMask
                backing:2    // NSBackingStoreBuffered
                defer:NO
            ];

            let _: () = msg_send![panel, setLevel:1000];
            let _: () = msg_send![panel, setFloatingPanel:YES];
            let _: () = msg_send![panel, setOpaque:NO];
            let _: () = msg_send![panel, setHasShadow:NO];
            
            // Ensure content view is retained before moving it
            let _: () = msg_send![content_view, retain];
            let _: () = msg_send![panel, setContentView:content_view];
            
            let _: () = msg_send![panel, setAcceptsMouseMovedEvents:YES];
            let _: () = msg_send![panel, setIgnoresMouseEvents:NO];
            let _: () = msg_send![panel, setMovableByWindowBackground:YES];
            
            // Set collection behavior
            let _: () = msg_send![panel, setCollectionBehavior: 
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces.bits() | 
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary.bits()
            ];
            
            // Make visible without making key
            let _: () = msg_send![panel, orderFront:nil];
            
            Self(panel)
        }
    }

    pub fn share(self) -> ShareId<Self> {
        unsafe { ShareId::from_ptr(Box::into_raw(Box::new(self))) }
    }
}

impl std::ops::Deref for RawClickThroughPanel {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0 as *const _) }
    }
}