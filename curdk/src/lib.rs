#![doc = include_str!("../README.md")]

use {
    curdk_sys::*,
    std::ffi::{CString, c_int, c_uint},
};

pub use curdk_sys::CENTER;
const SHADOW: i32 = false as i32;
const BOX: i32 = false as i32;

unsafe extern "C" fn callback(_btn: *mut CDKBUTTON) {}

pub trait ObjectExt: Sized {
    type T: 'static;
    fn as_raw(&self) -> *mut Self::T;
    fn from_raw(ptr: *mut Self::T) -> Self;
}

macro_rules! impl_object {
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
        impl_object!($name, $ptr);
        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    let mut obj: $ptr = self.0.unwrap().read();
                    _destroyCDKObject(&mut obj.obj);
                }
            }
        }
        paste::paste! {
            impl $name {
                pub fn set_box(&self, bx: bool) {
                    unsafe { [<setCDK $name Box>](self.as_raw(), bx as i32) }
                }
                pub fn set_activate(&self, activate: u32) {
                    unsafe { [<activateCDK $name Box>](self.as_raw(), activate as c_uint) }
                }
            }
        }
    };
}

impl_object!(Window, WINDOW);

impl Window {
    pub fn new() -> Self {
        Self::from_raw(unsafe { initscr() })
    }
}

impl_object!(Screen, CDKSCREEN);

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

impl_cdk!(Alphalist, CDKALPHALIST);
impl_cdk!(Button, CDKBUTTON);
impl Button {
    pub fn new(cdkscreen: &Screen, xpos: u32, ypos: u32, message_: &str) -> Self {
        let message = CString::new(message_).expect("CString::new failed");
        Self::from_raw(unsafe {
            newCDKButton(
                cdkscreen.as_raw(),
                xpos as c_int,
                ypos as c_int,
                message.as_ptr(),
                Some(callback),
                BOX,
                SHADOW,
            )
        })
    }
}
impl_cdk!(Buttonbox, CDKBUTTONBOX);
impl Buttonbox {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        height: u32,
        width: u32,
        rows: u32,
        cols: u32,
        buttons_: &[&str],
    ) -> Self {
        let buttons = buttons_
            .iter()
            .map(|arg| CString::new(*arg).unwrap())
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const i8>>();
        Self::from_raw(unsafe {
            newCDKButtonbox(
                cdkscreen.as_raw(),
                xpos as c_int,
                ypos as c_int,
                height as c_int,
                width as c_int,
                std::ptr::null(),
                rows as c_int,
                cols as c_int,
                buttons.as_ptr(),
                buttons.len() as c_int,
                curdk_sys::A_NORMAL,
                BOX,
                SHADOW,
            )
        })
    }
}
impl_cdk!(Calendar, CDKCALENDAR);
impl_cdk!(Dialog, CDKDIALOG);
impl_cdk!(DScale, CDKDSCALE);
impl_cdk!(Entry, CDKENTRY);
impl_cdk!(Fselect, CDKFSELECT);
impl_cdk!(Viewer, CDKVIEWER);
impl_cdk!(FScale, CDKFSCALE);
impl_cdk!(FSlider, CDKFSLIDER);
impl_cdk!(Graph, CDKGRAPH);
impl Graph {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        height: u32,
        width: u32,
        title_: &str,
        xtitle_: &str,
        ytitle_: &str,
    ) -> Self {
        let title = CString::new(title_).expect("CString::new failed");
        let xtitle = CString::new(xtitle_).expect("CString::new failed");
        let ytitle = CString::new(ytitle_).expect("CString::new failed");
        Self::from_raw(unsafe {
            newCDKGraph(
                cdkscreen.as_raw(),
                xpos as c_int,
                ypos as c_int,
                height as c_int,
                width as c_int,
                title.as_ptr(),
                xtitle.as_ptr(),
                ytitle.as_ptr(),
            )
        })
    }
}
impl_cdk!(Histogram, CDKHISTOGRAM);
impl Histogram {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        height: u32,
        width: u32,
        orient: u32,
        title_: &str,
    ) -> Self {
        let title = CString::new(title_).expect("CString::new failed");
        Self::from_raw(unsafe {
            newCDKHistogram(
                cdkscreen.as_raw(),
                xpos as c_int,
                ypos as c_int,
                height as c_int,
                width as c_int,
                orient as c_int,
                title.as_ptr(),
                BOX,
                SHADOW,
            )
        })
    }
}
impl_cdk!(Itemlist, CDKITEMLIST);
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
                BOX,
                SHADOW,
            )
        })
    }
    pub fn set_message(&self, message_: &str) {
        let message = CString::new(message_).expect("CString::new failed");
        unsafe { setCDKLabelMessage(self.as_raw(), &message.as_ptr(), 1) }
    }
}
impl_cdk!(Marquee, CDKMARQUEE);
impl_cdk!(Matrix, CDKMATRIX);
impl_cdk!(Mentry, CDKMENTRY);
impl_cdk!(Radio, CDKRADIO);
impl_cdk!(Scale, CDKSCALE);
impl_cdk!(Scroll, CDKSCROLL);
impl_cdk!(Selection, CDKSELECTION);
impl_cdk!(Slider, CDKSLIDER);
