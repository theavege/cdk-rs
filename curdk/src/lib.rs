#![doc = include_str!("../README.md")]

use {
    curdk_sys::*,
    std::ffi::{CString, c_int},
};

pub trait ObjectExt: Sized {
    type T: 'static;
    fn as_raw(&self) -> *mut Self::T;
    fn from_raw(ptr: *mut Self::T) -> Self;
}

macro_rules! impl_widget {
    ($name:ident, $ptr:ident) => {
        #[derive(Default)]
        pub struct $name(Option<std::ptr::NonNull<$ptr>>);

        impl ObjectExt for $name {
            type T = $ptr;
            fn as_raw(&self) -> *mut Self::T {
                self.0
                    .expect(concat!("Empty ", stringify!($name), "!"))
                    .as_ptr()
            }

            fn from_raw(ptr: *mut Self::T) -> Self {
                Self(std::ptr::NonNull::new(ptr))
            }
        }
    };
}

macro_rules! impl_cdk {
    ($name:ident, $ptr:ident) => {
        impl_widget!($name, $ptr);
        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    let mut obj: $ptr = self.0.unwrap().read();
                    _destroyCDKObject(&mut obj.obj);
                }
            }
        }
    };
}

impl_widget!(Window, WINDOW);

impl Window {
    pub fn new() -> Self {
        Self::from_raw(unsafe { initscr() })
    }
}

impl_widget!(Screen, CDKSCREEN);

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

impl_cdk!(AlphaList, CDKALPHALIST);
impl_cdk!(Button, CDKBUTTON);
impl_cdk!(ButtonBox, CDKBUTTONBOX);
impl_cdk!(Calendar, CDKCALENDAR);
impl_cdk!(Dialog, CDKDIALOG);
impl_cdk!(Entry, CDKENTRY);
impl_cdk!(Graph, CDKGRAPH);
impl_cdk!(Histogram, CDKHISTOGRAM);
impl_cdk!(ItemList, CDKITEMLIST);
impl_cdk!(Label, CDKLABEL);

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
    pub fn set_box(&self, bx: bool) {
        unsafe { setCDKLabelBox(self.as_raw(), bx as i32) }
    }
}

impl_cdk!(Marquee, CDKMARQUEE);
impl_cdk!(Matrix, CDKMATRIX);
impl_cdk!(Mentry, CDKMENTRY);
impl_cdk!(Menu, CDKMENU);
impl_cdk!(Radio, CDKRADIO);
impl_cdk!(Scale, CDKSCALE);
impl_cdk!(Scroll, CDKSCROLL);
impl_cdk!(Selection, CDKSELECTION);
impl_cdk!(Slider, CDKSLIDER);
impl_cdk!(Viewer, CDKVIEWER);
