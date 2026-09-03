#![doc = include_str!("../README.md")]
#![doc = r#"
## Quick start

Constructors return `Result` values because terminal and native widget
initialization can fail:

```no_run
let window = curdk::Window::new()?;
let screen = curdk::Screen::new(&window)?;
let entry = curdk::Entry::new(&screen, curdk::CENTER, curdk::CENTER, "Name", "Input: ")?;
entry.set_value("initial")?;
screen.refresh();
# Ok::<(), curdk::Error>(())
```
"#]

use {
    curdk_sys::*,
    std::{
        error::Error as StdError,
        ffi::{CStr, CString, NulError, c_char, c_int},
        ptr::NonNull,
        rc::Rc,
        str::Utf8Error,
    },
};

pub use curdk_sys::{BOTTOM, CENTER, LEFT, RIGHT, TOP};
const SHADOW: i32 = false as i32;
const BOX: i32 = false as i32;

#[derive(Debug)]
pub enum Error {
    NullHandle(&'static str),
    InteriorNul(NulError),
    ValueOutOfRange(&'static str, u64),
    InvalidUtf8(Utf8Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullHandle(name) => write!(formatter, "CDK returned a null {name} handle"),
            Self::InteriorNul(error) => error.fmt(formatter),
            Self::ValueOutOfRange(name, value) => {
                write!(
                    formatter,
                    "{name} value {value} exceeds the C integer range"
                )
            }
            Self::InvalidUtf8(error) => error.fmt(formatter),
        }
    }
}

impl StdError for Error {}

impl From<NulError> for Error {
    fn from(error: NulError) -> Self {
        Self::InteriorNul(error)
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Self::InvalidUtf8(error)
    }
}

fn checked_c_int(value: u64, name: &'static str) -> Result<c_int, Error> {
    c_int::try_from(value).map_err(|_| Error::ValueOutOfRange(name, value))
}

unsafe fn owned_c_string(ptr: *const c_char, name: &'static str) -> Result<String, Error> {
    let ptr = NonNull::new(ptr as *mut c_char).ok_or(Error::NullHandle(name))?;
    Ok(unsafe { CStr::from_ptr(ptr.as_ptr()) }.to_str()?.to_owned())
}

struct WindowOwner {
    ptr: NonNull<WINDOW>,
}

struct ScreenOwner {
    ptr: NonNull<CDKSCREEN>,
    _window: Rc<WindowOwner>,
}

impl Drop for ScreenOwner {
    fn drop(&mut self) {
        unsafe {
            destroyCDKScreen(self.ptr.as_ptr());
            endCDK();
        }
    }
}

macro_rules! impl_object {
    ($name:ident, $ptr:ident) => {
        pub struct $name {
            ptr: NonNull<$ptr>,
            _screen: Rc<ScreenOwner>,
        }

        impl $name {
            #[allow(dead_code)]
            fn from_raw(screen: &Screen, ptr: *mut $ptr) -> Result<Self, Error> {
                Ok(Self {
                    ptr: NonNull::new(ptr).ok_or(Error::NullHandle(stringify!($name)))?,
                    _screen: Rc::clone(&screen.owner),
                })
            }

            fn as_raw(&self) -> *mut $ptr {
                self.ptr.as_ptr()
            }
        }
    };
}

macro_rules! impl_cdk {
    ($name:ident, $ptr:ident) => {
        impl_object!($name, $ptr);
        paste::paste! {
            impl $name {
                pub fn set_box(&self, bx: bool) {
                    unsafe { [<setCDK $name Box>](self.as_raw(), bx as i32) }
                }
                pub fn bx(&self) -> bool {
                    unsafe { [<getCDK $name Box>](self.as_raw()) != 0 }
                }
                pub fn draw(&self) {
                    unsafe {
                        let ptr = self.as_raw();
                        if let Some(func) = (*ptr).obj.fn_.as_ref().unwrap().drawObj {
                            func(&mut (*ptr).obj, self.bx() as i32);
                        }
                    }
                }
            }
        }
    };
}

pub struct Window {
    owner: Rc<WindowOwner>,
}

impl Window {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            owner: Rc::new(WindowOwner {
                ptr: NonNull::new(unsafe { initscr() }).ok_or(Error::NullHandle("Window"))?,
            }),
        })
    }

    fn as_raw(&self) -> *mut WINDOW {
        self.owner.ptr.as_ptr()
    }
}

pub struct Screen {
    owner: Rc<ScreenOwner>,
}

impl Screen {
    pub fn new(win: &Window) -> Result<Self, Error> {
        let ptr = unsafe { initCDKScreen(win.as_raw()) };
        let ptr = NonNull::new(ptr).ok_or(Error::NullHandle("Screen"))?;
        unsafe { initCDKColor() };
        Ok(Self {
            owner: Rc::new(ScreenOwner {
                ptr,
                _window: Rc::clone(&win.owner),
            }),
        })
    }

    fn as_raw(&self) -> *mut CDKSCREEN {
        self.owner.ptr.as_ptr()
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

impl_cdk!(Alphalist, CDKALPHALIST);
impl_cdk!(Button, CDKBUTTON);
impl Button {
    pub fn new(cdkscreen: &Screen, xpos: u32, ypos: u32, message_: &str) -> Result<Self, Error> {
        let message = CString::new(message_)?;
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKButton(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                message.as_ptr(),
                None,
                BOX,
                SHADOW,
            )
        })
    }
    pub fn activate(&self) -> i32 {
        unsafe { activateCDKButton(self.as_raw(), std::ptr::null_mut()) }
    }
}
impl_cdk!(Buttonbox, CDKBUTTONBOX);
impl Buttonbox {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        height: u32,
        width: u32,
        rows: u32,
        cols: u32,
        buttons_: &[&str],
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        let rows = checked_c_int(rows as u64, "rows")?;
        let cols = checked_c_int(cols as u64, "columns")?;
        let button_count = checked_c_int(buttons_.len() as u64, "button count")?;
        let button_strings = buttons_
            .iter()
            .map(|arg| CString::new(*arg))
            .collect::<Result<Vec<CString>, NulError>>()?;
        let mut buttons = button_strings
            .iter()
            .map(|arg| arg.as_ptr() as *mut i8)
            .collect::<Vec<*mut i8>>();
        Self::from_raw(cdkscreen, unsafe {
            newCDKButtonbox(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                height,
                width,
                std::ptr::null(),
                rows,
                cols,
                buttons.as_mut_ptr(),
                button_count,
                curdk_sys::A_NORMAL,
                BOX,
                SHADOW,
            )
        })
    }
}
impl_cdk!(Calendar, CDKCALENDAR);
impl_cdk!(Dialog, CDKDIALOG);
impl Dialog {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        message_: &str,
        rows: u32,
        buttons_: &[&str],
    ) -> Result<Self, Error> {
        let message = CString::new(message_)?;
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let rows = checked_c_int(rows as u64, "rows")?;
        let button_count = checked_c_int(buttons_.len() as u64, "button count")?;
        let mut message_ptr = message.as_ptr() as *mut c_char;
        let button_strings = buttons_
            .iter()
            .map(|arg| CString::new(*arg))
            .collect::<Result<Vec<CString>, NulError>>()?;
        let mut buttons = button_strings
            .iter()
            .map(|arg| arg.as_ptr() as *mut i8)
            .collect::<Vec<*mut i8>>();
        Self::from_raw(cdkscreen, unsafe {
            newCDKDialog(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                &mut message_ptr,
                rows,
                buttons.as_mut_ptr(),
                button_count,
                curdk_sys::A_NORMAL,
                0,
                BOX,
                SHADOW,
            )
        })
    }

    pub fn activate(&self) -> i32 {
        unsafe { activateCDKDialog(self.as_raw(), std::ptr::null_mut()) }
    }
}
impl_cdk!(DScale, CDKDSCALE);
impl_cdk!(Entry, CDKENTRY);
impl Entry {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
    ) -> Result<Self, Error> {
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let label_length = checked_c_int(label_.len() as u64, "label length")?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKEntry(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                curdk_sys::A_NORMAL,
                0,
                0,
                0,
                label_length,
                label_length,
                BOX,
                SHADOW,
            )
        })
    }

    pub fn activate(&self) -> Result<String, Error> {
        let value = unsafe { activateCDKEntry(self.as_raw(), std::ptr::null_mut()) };
        unsafe { owned_c_string(value, "Entry value") }
    }

    pub fn value(&self) -> Result<String, Error> {
        let value = unsafe { getCDKEntryValue(self.as_raw()) };
        unsafe { owned_c_string(value, "Entry value") }
    }

    pub fn set_value(&self, value_: &str) -> Result<(), Error> {
        let value = CString::new(value_)?;
        unsafe { setCDKEntryValue(self.as_raw(), value.as_ptr()) };
        Ok(())
    }

    pub fn set_min(&self, min: u32) -> Result<(), Error> {
        let min = checked_c_int(min as u64, "minimum length")?;
        unsafe { setCDKEntryMin(self.as_raw(), min) };
        Ok(())
    }

    pub fn set_max(&self, max: u32) -> Result<(), Error> {
        let max = checked_c_int(max as u64, "maximum length")?;
        unsafe { setCDKEntryMax(self.as_raw(), max) };
        Ok(())
    }
}
impl_cdk!(Fselect, CDKFSELECT);
impl_cdk!(Viewer, CDKVIEWER);
impl_cdk!(FScale, CDKFSCALE);
impl_cdk!(FSlider, CDKFSLIDER);
impl_cdk!(Graph, CDKGRAPH);
impl Graph {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        height: u32,
        width: u32,
        title_: &str,
        xtitle_: &str,
        ytitle_: &str,
    ) -> Result<Self, Error> {
        let title = CString::new(title_)?;
        let xtitle = CString::new(xtitle_)?;
        let ytitle = CString::new(ytitle_)?;
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKGraph(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                height,
                width,
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
    ) -> Result<Self, Error> {
        let title = CString::new(title_)?;
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        let orient = checked_c_int(orient as u64, "orientation")?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKHistogram(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                height,
                width,
                orient,
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
    pub fn new(cdkscreen: &Screen, xpos: u32, ypos: u32, message_: &str) -> Result<Self, Error> {
        let message = CString::new(message_)?;
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let mut message_ptr = message.as_ptr() as *mut c_char;
        Self::from_raw(cdkscreen, unsafe {
            newCDKLabel(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                &mut message_ptr,
                1,
                BOX,
                SHADOW,
            )
        })
    }
    pub fn set_message(&self, message_: &str) -> Result<(), Error> {
        let message = CString::new(message_)?;
        let mut message_ptr = message.as_ptr() as *mut c_char;
        unsafe { setCDKLabelMessage(self.as_raw(), &mut message_ptr, 1) };
        Ok(())
    }
    /// Waits  for  a  user  to press a key.
    ///
    /// This function initializes the EFL libraries, creates the window using the provided
    /// function, and starts the main event loop.
    pub fn wait(&self, key: char) -> char {
        unsafe { waitCDKLabel(self.as_raw(), key as c_char) as u8 as char }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interior_nul_is_reported() {
        let error = Error::from(CString::new("invalid\0message").unwrap_err());
        assert!(matches!(error, Error::InteriorNul(_)));
    }

    #[test]
    fn null_handle_has_context() {
        let error = Error::NullHandle("Label");
        assert_eq!(error.to_string(), "CDK returned a null Label handle");
    }

    #[test]
    fn c_integer_conversion_rejects_overflow() {
        assert_eq!(
            checked_c_int(c_int::MAX as u64, "width").unwrap(),
            c_int::MAX
        );
        assert!(matches!(
            checked_c_int(c_int::MAX as u64 + 1, "width"),
            Err(Error::ValueOutOfRange("width", _))
        ));
    }

    #[test]
    #[ignore = "requires an interactive terminal"]
    fn widget_keeps_screen_alive() -> Result<(), Error> {
        let window = Window::new()?;
        let label = {
            let screen = Screen::new(&window)?;
            Label::new(&screen, CENTER, TOP, "label")?
        };
        label.set_box(false);
        Ok(())
    }

    #[test]
    #[ignore = "requires an interactive terminal"]
    fn dialog_returns_a_selection() -> Result<(), Error> {
        let window = Window::new()?;
        let screen = Screen::new(&window)?;
        let dialog = Dialog::new(&screen, CENTER, CENTER, "Continue?", 1, &["Yes", "No"])?;
        assert!(dialog.activate() >= 0);
        Ok(())
    }
}
