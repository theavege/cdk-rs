#![doc = include_str!("../README.md")]

use {
    curdk_sys::*,
    std::{
        ffi::{CString, c_int},
        ptr::NonNull,
    },
};

pub trait ObjectExt: Sized {
    type T: 'static;
    fn as_raw(&self) -> *mut Self::T;
    fn from_raw(ptr: *mut Self::T) -> Self;
}

pub struct Window(Option<std::ptr::NonNull<WINDOW>>);

impl ObjectExt for Window {
    type T = WINDOW;
    fn as_raw(&self) -> *mut Self::T {
        self.0.unwrap().as_ptr()
    }
    fn from_raw(obj: *mut Self::T) -> Self {
        Self(NonNull::new(obj))
    }
}

impl Window {
    pub fn new() -> Self {
        Self::from_raw(unsafe { initscr() })
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Screen(Option<std::ptr::NonNull<CDKSCREEN>>);

impl ObjectExt for Screen {
    type T = CDKSCREEN;
    fn as_raw(&self) -> *mut Self::T {
        self.0.unwrap().as_ptr()
    }
    fn from_raw(obj: *mut Self::T) -> Self {
        Self(NonNull::new(obj))
    }
}

impl Screen {
    pub fn new(win: &Window) -> Self {
        Self::from_raw(unsafe {
            let scr = initCDKScreen(win.as_raw());
            initCDKColor();
            scr
        })
    }
    pub fn draw(&self) {
        unsafe {
            drawCDKScreen(self.as_raw());
        }
    }
    pub fn refresh(&self) {
        unsafe {
            refreshCDKScreen(self.as_raw());
        }
    }
    pub fn erase(&self) {
        unsafe {
            eraseCDKScreen(self.as_raw());
        }
    }
    pub fn exit(&self) {
        unsafe {
            exitOKCDKScreen(self.as_raw());
        }
    }
    pub fn reset(&self) {
        unsafe {
            resetCDKScreen(self.as_raw());
        }
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        unsafe {
            destroyCDKScreen(self.as_raw());
            endCDK();
        }
    }
}

pub struct Label(Option<std::ptr::NonNull<CDKLABEL>>);

impl ObjectExt for Label {
    type T = CDKLABEL;
    fn as_raw(&self) -> *mut Self::T {
        self.0.unwrap().as_ptr()
    }
    fn from_raw(obj: *mut Self::T) -> Self {
        Self(NonNull::new(obj))
    }
}

impl Label {
    pub fn new(cdkscreen: &Screen, xpos: u32, ypos: u32, message_: &str) -> Self {
        let message = CString::new(message_).expect("CString::new failed");
        Self::from_raw(unsafe {
            newCDKLabel(
                cdkscreen.as_raw(),
                xpos as c_int,
                ypos as c_int,
                &message.as_ptr(),
                1,
                0,
                0,
            )
        })
    }
    pub fn set_message(&self, message_: &str) {
        let message = CString::new(message_).expect("CString::new failed");
        unsafe { setCDKLabelMessage(self.as_raw(), &message.as_ptr(), 1) }
    }
}

impl Drop for Label {
    fn drop(&mut self) {
        unsafe {
            let mut obj: CDKLABEL = self.0.unwrap().read();
            _destroyCDKObject(&mut obj.obj);
        }
    }
}
