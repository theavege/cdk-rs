#![doc = include_str!("../README.md")]
#![doc = r#"
## Quick start

Constructors return `Result` values because terminal and native widget
initialization can fail:

```no_run
let window = curdk::Window::new()?;
let screen = curdk::Screen::new(&window)?;
let entry = curdk::Entry::new(&screen, curdk::CENTER, curdk::CENTER, "Name", "Input: ", curdk::Border::NONE)?;
entry.set_value("initial")?;
screen.refresh();
# Ok::<(), curdk::Error>(())
```
"#]

use {
    curdk_sys::*,
    std::{
        cell::{Cell, RefCell},
        collections::HashMap,
        error::Error as StdError,
        ffi::{CStr, CString, NulError, c_char, c_int, c_void},
        ptr::NonNull,
        rc::Rc,
        str::Utf8Error,
    },
};

pub use curdk_sys::{BOTTOM, CENTER, LEFT, RIGHT, TOP};

/// Box and drop-shadow flags passed to CDK widget constructors.
///
/// `NONE` is an unframed widget, `BOXED` draws a border, and `SHADOWED`
/// adds a drop shadow. Graph, Menu, Window, and Screen do not take a border.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Border {
    pub boxed: bool,
    pub shadow: bool,
}

impl Border {
    pub const NONE: Self = Self {
        boxed: false,
        shadow: false,
    };
    pub const BOXED: Self = Self {
        boxed: true,
        shadow: false,
    };
    pub const SHADOWED: Self = Self {
        boxed: true,
        shadow: true,
    };

    fn boxed_flag(self) -> c_int {
        self.boxed as c_int
    }

    fn shadow_flag(self) -> c_int {
        self.shadow as c_int
    }
}

pub trait Widget {
    fn set_box(&self, bx: bool);
    fn is_boxed(&self) -> bool;
    fn draw(&self);
}

pub trait NumericWidget {
    type Value: Copy;

    fn value(&self) -> Self::Value;
    fn set_value(&self, value: Self::Value);
    fn range(&self) -> (Self::Value, Self::Value);
    fn set_range(&self, low: Self::Value, high: Self::Value);
}

#[derive(Debug, PartialEq, Eq)]
pub enum Activation {
    Selected(usize),
    Cancelled,
}

#[derive(Debug)]
pub enum Error {
    NullHandle(&'static str),
    InteriorNul(NulError),
    ValueOutOfRange(&'static str, u64),
    InvalidUtf8(Utf8Error),
    InvalidActivation(i32),
    LengthMismatch(&'static str, usize, usize),
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
            Self::InvalidActivation(value) => {
                write!(
                    formatter,
                    "CDK returned an invalid activation value {value}"
                )
            }
            Self::LengthMismatch(name, expected, actual) => {
                write!(formatter, "{name} has length {actual}, expected {expected}")
            }
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

fn activation(value: i32) -> Result<Activation, Error> {
    match value {
        -1 => Ok(Activation::Cancelled),
        value if value >= 0 => Ok(Activation::Selected(value as usize)),
        value => Err(Error::InvalidActivation(value)),
    }
}

unsafe fn owned_c_string(ptr: *const c_char, name: &'static str) -> Result<String, Error> {
    let ptr = NonNull::new(ptr as *mut c_char).ok_or(Error::NullHandle(name))?;
    Ok(unsafe { CStr::from_ptr(ptr.as_ptr()) }.to_str()?.to_owned())
}

struct CStringArray {
    _strings: Vec<CString>,
    pointers: Vec<*mut c_char>,
}

impl CStringArray {
    fn new(values: &[&str]) -> Result<Self, Error> {
        let strings = values
            .iter()
            .map(|value| CString::new(*value))
            .collect::<Result<Vec<CString>, NulError>>()?;
        let pointers = strings
            .iter()
            .map(|value| value.as_ptr() as *mut c_char)
            .collect();
        Ok(Self {
            _strings: strings,
            pointers,
        })
    }

    fn as_mut_ptr(&mut self) -> *mut *mut c_char {
        self.pointers.as_mut_ptr()
    }
}

fn one_based_strings(values: &[&str]) -> Result<CStringArray, Error> {
    let mut prefixed = Vec::with_capacity(values.len() + 1);
    prefixed.push("");
    prefixed.extend(values.iter().copied());
    CStringArray::new(&prefixed)
}

fn one_based_ints(values: &[u32], name: &'static str) -> Result<Vec<c_int>, Error> {
    let mut integers = Vec::with_capacity(values.len() + 1);
    integers.push(0);
    for value in values {
        integers.push(checked_c_int(*value as u64, name)?);
    }
    Ok(integers)
}

struct CallbackSlot {
    handler: Box<dyn FnMut(u32) -> i32>,
}

#[derive(Default)]
struct WidgetCallbacks {
    keys: HashMap<u32, Box<CallbackSlot>>,
    pre: Option<Box<CallbackSlot>>,
    post: Option<Box<CallbackSlot>>,
}

unsafe extern "C" fn callback_trampoline(
    _ty: EObjectType,
    _object: *mut c_void,
    data: *mut c_void,
    input: chtype,
) -> c_int {
    if data.is_null() {
        return 0;
    }
    let slot = unsafe { &mut *(data as *mut CallbackSlot) };
    (slot.handler)(input)
}

struct WindowOwner {
    ptr: NonNull<WINDOW>,
    ended: Cell<bool>,
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
        self._window.ended.set(true);
    }
}

impl Drop for WindowOwner {
    fn drop(&mut self) {
        if !self.ended.replace(true) {
            unsafe { endwin() };
        }
    }
}

macro_rules! impl_object {
    ($name:ident, $ptr:ident) => {
        pub struct $name {
            ptr: NonNull<$ptr>,
            _screen: Rc<ScreenOwner>,
            callbacks: RefCell<WidgetCallbacks>,
        }

        impl $name {
            const OBJECT_TYPE: EObjectType = paste::paste! { [<EObjectType_v $name:upper>] };

            #[allow(dead_code)]
            fn from_raw(screen: &Screen, ptr: *mut $ptr) -> Result<Self, Error> {
                Ok(Self {
                    ptr: NonNull::new(ptr).ok_or(Error::NullHandle(stringify!($name)))?,
                    _screen: Rc::clone(&screen.owner),
                    callbacks: RefCell::new(WidgetCallbacks::default()),
                })
            }

            fn as_raw(&self) -> *mut $ptr {
                self.ptr.as_ptr()
            }

            fn as_obj(&self) -> *mut CDKOBJS {
                unsafe { std::ptr::addr_of_mut!((*self.as_raw()).obj) }
            }

            fn as_void(&self) -> *mut c_void {
                self.as_raw() as *mut c_void
            }

            /// Bind `key` to a Rust callback. Return `1` from the callback to consume the key.
            pub fn bind_key<F>(&self, key: u32, handler: F) -> Result<(), Error>
            where
                F: FnMut(u32) -> i32 + 'static,
            {
                let key_code = checked_c_int(key as u64, "key")?;
                self.unbind_key(key)?;
                let mut slot = Box::new(CallbackSlot {
                    handler: Box::new(handler),
                });
                let data = slot.as_mut() as *mut CallbackSlot as *mut c_void;
                self.callbacks.borrow_mut().keys.insert(key, slot);
                unsafe {
                    bindCDKObject(
                        Self::OBJECT_TYPE,
                        self.as_void(),
                        key_code as chtype,
                        Some(callback_trampoline),
                        data,
                    );
                }
                Ok(())
            }

            pub fn unbind_key(&self, key: u32) -> Result<(), Error> {
                let key_code = checked_c_int(key as u64, "key")?;
                unsafe {
                    unbindCDKObject(Self::OBJECT_TYPE, self.as_void(), key_code as chtype);
                }
                self.callbacks.borrow_mut().keys.remove(&key);
                Ok(())
            }

            pub fn has_binding(&self, key: u32) -> Result<bool, Error> {
                let key_code = checked_c_int(key as u64, "key")?;
                Ok(
                    unsafe {
                        isCDKObjectBind(Self::OBJECT_TYPE, self.as_void(), key_code as chtype)
                    } as c_int
                        != 0,
                )
            }

            /// Called before CDK applies a key. Return `1` to continue, `0` to ignore it.
            pub fn set_preprocess<F>(&self, handler: F)
            where
                F: FnMut(u32) -> i32 + 'static,
            {
                let mut slot = Box::new(CallbackSlot {
                    handler: Box::new(handler),
                });
                let data = slot.as_mut() as *mut CallbackSlot as *mut c_void;
                self.callbacks.borrow_mut().pre = Some(slot);
                unsafe {
                    setCDKObjectPreProcess(self.as_obj(), Some(callback_trampoline), data);
                }
            }

            pub fn clear_preprocess(&self) {
                unsafe { setCDKObjectPreProcess(self.as_obj(), None, std::ptr::null_mut()) };
                self.callbacks.borrow_mut().pre = None;
            }

            /// Called after CDK applies a key. Return `1` to continue, `0` to stop.
            pub fn set_postprocess<F>(&self, handler: F)
            where
                F: FnMut(u32) -> i32 + 'static,
            {
                let mut slot = Box::new(CallbackSlot {
                    handler: Box::new(handler),
                });
                let data = slot.as_mut() as *mut CallbackSlot as *mut c_void;
                self.callbacks.borrow_mut().post = Some(slot);
                unsafe {
                    setCDKObjectPostProcess(self.as_obj(), Some(callback_trampoline), data);
                }
            }

            pub fn clear_postprocess(&self) {
                unsafe { setCDKObjectPostProcess(self.as_obj(), None, std::ptr::null_mut()) };
                self.callbacks.borrow_mut().post = None;
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    let mut object = self.ptr.as_ptr().read();
                    _destroyCDKObject(&mut object.obj);
                }
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
                        if let Some(functions) = (*ptr).obj.fn_.as_ref() {
                            if let Some(func) = functions.drawObj {
                                func(&mut (*ptr).obj, self.bx() as i32);
                            }
                        }
                    }
                }
            }
        }
        impl Widget for $name {
            fn set_box(&self, bx: bool) {
                self.set_box(bx);
            }
            fn is_boxed(&self) -> bool {
                self.bx()
            }
            fn draw(&self) {
                self.draw();
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
                ended: Cell::new(false),
            }),
        })
    }

    fn as_raw(&self) -> *mut WINDOW {
        self.owner.ptr.as_ptr()
    }
}

#[derive(Clone)]
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
    pub fn cancel(&self) {
        unsafe {
            exitCancelCDKScreen(self.as_raw());
        }
    }
    /// Tab between focusable widgets until `exit` or `cancel`.
    ///
    /// Returns `true` when the screen exited OK. Bind a key that calls
    /// [`Screen::exit`] or [`Screen::cancel`] so the loop can finish:
    ///
    /// ```no_run
    /// # fn main() -> Result<(), curdk::Error> {
    /// let window = curdk::Window::new()?;
    /// let screen = curdk::Screen::new(&window)?;
    /// let entry = curdk::Entry::new(
    ///     &screen,
    ///     curdk::CENTER,
    ///     curdk::CENTER,
    ///     "Name",
    ///     "Name: ",
    ///     curdk::Border::BOXED,
    /// )?;
    /// let done = screen.clone();
    /// entry.bind_key(b'x' as u32 & 0x1f, move |_| {
    ///     done.exit();
    ///     1
    /// })?;
    /// let accepted = screen.traverse();
    /// # let _ = accepted;
    /// # Ok(())
    /// # }
    /// ```
    pub fn traverse(&self) -> bool {
        unsafe { traverseCDKScreen(self.as_raw()) != 0 }
    }
    pub fn reset(&self) {
        unsafe {
            resetCDKScreen(self.as_raw());
        }
    }
}

impl_cdk!(Alphalist, CDKALPHALIST);
impl Alphalist {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        height: u32,
        width: u32,
        title_: &str,
        label_: &str,
        items_: &[&str],
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        let item_count = checked_c_int(items_.len() as u64, "item count")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        let mut items = CStringArray::new(items_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKAlphalist(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                height,
                width,
                title.as_ptr(),
                label.as_ptr(),
                items.as_mut_ptr(),
                item_count,
                ' ' as curdk_sys::chtype,
                curdk_sys::A_NORMAL,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> Result<String, Error> {
        let value = unsafe { activateCDKAlphalist(self.as_raw(), std::ptr::null_mut()) };
        unsafe { owned_c_string(value, "Alphalist value") }
    }

    pub fn current(&self) -> i32 {
        unsafe { getCDKAlphalistCurrentItem(self.as_raw()) }
    }

    pub fn set_current(&self, item: u32) -> Result<(), Error> {
        let item = checked_c_int(item as u64, "item index")?;
        unsafe { setCDKAlphalistCurrentItem(self.as_raw(), item) };
        Ok(())
    }

    pub fn set_items(&self, items_: &[&str]) -> Result<(), Error> {
        let item_count = checked_c_int(items_.len() as u64, "item count")?;
        let mut items = CStringArray::new(items_)?;
        unsafe { setCDKAlphalistContents(self.as_raw(), items.as_mut_ptr(), item_count) };
        Ok(())
    }
}
impl_cdk!(Button, CDKBUTTON);
impl Button {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        message_: &str,
        border: Border,
    ) -> Result<Self, Error> {
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
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }
    pub fn activate(&self) -> i32 {
        unsafe { activateCDKButton(self.as_raw(), std::ptr::null_mut()) }
    }
    pub fn activate_result(&self) -> Result<Activation, Error> {
        activation(self.activate())
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
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        let rows = checked_c_int(rows as u64, "rows")?;
        let cols = checked_c_int(cols as u64, "columns")?;
        let button_count = checked_c_int(buttons_.len() as u64, "button count")?;
        let mut buttons = CStringArray::new(buttons_)?;
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
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> i32 {
        unsafe { activateCDKButtonbox(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn activate_result(&self) -> Result<Activation, Error> {
        activation(self.activate())
    }
}
impl_cdk!(Calendar, CDKCALENDAR);
impl Calendar {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        day: u32,
        month: u32,
        year: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let day = checked_c_int(day as u64, "day")?;
        let month = checked_c_int(month as u64, "month")?;
        let year = checked_c_int(year as u64, "year")?;
        let title = CString::new(title_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKCalendar(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                day,
                month,
                year,
                curdk_sys::A_NORMAL,
                curdk_sys::A_NORMAL,
                curdk_sys::A_NORMAL,
                curdk_sys::A_NORMAL,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> i64 {
        unsafe { activateCDKCalendar(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn date(&self) -> (i32, i32, i32) {
        let mut day = 0;
        let mut month = 0;
        let mut year = 0;
        unsafe { getCDKCalendarDate(self.as_raw(), &mut day, &mut month, &mut year) };
        (day, month, year)
    }

    pub fn set_date(&self, day: u32, month: u32, year: u32) -> Result<(), Error> {
        let day = checked_c_int(day as u64, "day")?;
        let month = checked_c_int(month as u64, "month")?;
        let year = checked_c_int(year as u64, "year")?;
        unsafe { setCDKCalendarDate(self.as_raw(), day, month, year) };
        Ok(())
    }
}
impl_cdk!(Dialog, CDKDIALOG);
impl_object!(Menu, CDKMENU);
impl Menu {
    pub fn new(
        cdkscreen: &Screen,
        menus_: &[&[&str]],
        locations_: &[(u32, u32)],
        menu_pos: u32,
    ) -> Result<Self, Error> {
        if menus_.len() != locations_.len() {
            return Err(Error::LengthMismatch(
                "menu locations",
                menus_.len(),
                locations_.len(),
            ));
        }
        let menu_count = checked_c_int(menus_.len() as u64, "menu item count")?;
        let menu_pos = checked_c_int(menu_pos as u64, "menu position")?;
        let mut menu_items = [[std::ptr::null(); 98]; 98];
        let mut strings = Vec::with_capacity(menus_.iter().map(|menu| menu.len()).sum());
        let mut submenu_sizes = Vec::with_capacity(menus_.len());
        let mut locations = Vec::with_capacity(menus_.len() * 2);
        for (menu_index, menu) in menus_.iter().enumerate() {
            if menu.len() > 98 {
                return Err(Error::ValueOutOfRange(
                    "submenu item count",
                    menu.len() as u64,
                ));
            }
            submenu_sizes.push(menu.len() as c_int);
            locations.push(checked_c_int(locations_[menu_index].0 as u64, "menu row")?);
            locations.push(checked_c_int(
                locations_[menu_index].1 as u64,
                "menu column",
            )?);
            for (item_index, item) in menu.iter().enumerate() {
                strings.push(CString::new(*item)?);
                menu_items[menu_index][item_index] = strings.last().unwrap().as_ptr();
            }
        }
        Self::from_raw(cdkscreen, unsafe {
            newCDKMenu(
                cdkscreen.as_raw(),
                menu_items.as_mut_ptr(),
                menu_count,
                submenu_sizes.as_mut_ptr(),
                locations.as_mut_ptr(),
                menu_pos,
                curdk_sys::A_NORMAL,
                curdk_sys::A_NORMAL,
            )
        })
    }

    pub fn activate(&self) -> i32 {
        unsafe { activateCDKMenu(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn current(&self) -> (i32, i32) {
        let mut menu_item = 0;
        let mut submenu_item = 0;
        unsafe { getCDKMenuCurrentItem(self.as_raw(), &mut menu_item, &mut submenu_item) };
        (menu_item, submenu_item)
    }

    pub fn set_current(&self, menu_item: u32, submenu_item: u32) -> Result<(), Error> {
        let menu_item = checked_c_int(menu_item as u64, "menu item")?;
        let submenu_item = checked_c_int(submenu_item as u64, "submenu item")?;
        unsafe { setCDKMenuCurrentItem(self.as_raw(), menu_item, submenu_item) };
        Ok(())
    }
}
impl Dialog {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        message_: &str,
        rows: u32,
        buttons_: &[&str],
        border: Border,
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
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> i32 {
        unsafe { activateCDKDialog(self.as_raw(), std::ptr::null_mut()) }
    }
    pub fn activate_result(&self) -> Result<Activation, Error> {
        activation(self.activate())
    }
}
impl_cdk!(DScale, CDKDSCALE);
impl DScale {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        field_width: u32,
        start: f64,
        low: f64,
        high: f64,
        increment: f64,
        fast_increment: f64,
        digits: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let field_width = checked_c_int(field_width as u64, "field width")?;
        let digits = checked_c_int(digits as u64, "digits")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKDScale(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                curdk_sys::A_NORMAL,
                field_width,
                start,
                low,
                high,
                increment,
                fast_increment,
                digits,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> f64 {
        unsafe { activateCDKDScale(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn value(&self) -> f64 {
        unsafe { getCDKDScaleValue(self.as_raw()) }
    }

    pub fn set_value(&self, value: f64) {
        unsafe { setCDKDScaleValue(self.as_raw(), value) };
    }

    pub fn range(&self) -> (f64, f64) {
        unsafe {
            (
                getCDKDScaleLowValue(self.as_raw()),
                getCDKDScaleHighValue(self.as_raw()),
            )
        }
    }

    pub fn set_range(&self, low: f64, high: f64) {
        unsafe { setCDKDScaleLowHigh(self.as_raw(), low, high) };
    }

    pub fn digits(&self) -> i32 {
        unsafe { getCDKDScaleDigits(self.as_raw()) }
    }

    pub fn set_digits(&self, digits: u32) -> Result<(), Error> {
        let digits = checked_c_int(digits as u64, "digits")?;
        unsafe { setCDKDScaleDigits(self.as_raw(), digits) };
        Ok(())
    }
}
impl_cdk!(Entry, CDKENTRY);
impl Entry {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        border: Border,
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
                border.boxed_flag(),
                border.shadow_flag(),
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
impl Fselect {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        height: u32,
        width: u32,
        title_: &str,
        label_: &str,
        directory_: &str,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKFselect(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                height,
                width,
                title.as_ptr(),
                label.as_ptr(),
                curdk_sys::A_NORMAL,
                ' ' as curdk_sys::chtype,
                curdk_sys::A_NORMAL,
                c"<B>".as_ptr(),
                c"<F>".as_ptr(),
                c"<L>".as_ptr(),
                c"<S>".as_ptr(),
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
        .and_then(|fselect| {
            fselect.set_directory(directory_)?;
            Ok(fselect)
        })
    }

    pub fn activate(&self) -> Result<String, Error> {
        let value = unsafe { activateCDKFselect(self.as_raw(), std::ptr::null_mut()) };
        unsafe { owned_c_string(value, "Fselect path") }
    }

    pub fn set_directory(&self, directory_: &str) -> Result<(), Error> {
        let directory = CString::new(directory_)?;
        let result = unsafe { setCDKFselectDirectory(self.as_raw(), directory.as_ptr()) };
        if result < 0 {
            return Err(Error::InvalidActivation(result));
        }
        Ok(())
    }

    pub fn directory(&self) -> Result<String, Error> {
        let value = unsafe { getCDKFselectDirectory(self.as_raw()) };
        unsafe { owned_c_string(value, "Fselect directory") }
    }
}
impl_cdk!(Viewer, CDKVIEWER);
impl Viewer {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        height: u32,
        width: u32,
        buttons_: &[&str],
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        let button_count = checked_c_int(buttons_.len() as u64, "button count")?;
        let button_strings = buttons_
            .iter()
            .map(|button| CString::new(*button))
            .collect::<Result<Vec<CString>, NulError>>()?;
        let mut buttons = button_strings
            .iter()
            .map(|button| button.as_ptr() as *mut c_char)
            .collect::<Vec<*mut c_char>>();
        Self::from_raw(cdkscreen, unsafe {
            newCDKViewer(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                height,
                width,
                buttons.as_mut_ptr(),
                button_count,
                curdk_sys::A_NORMAL,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> Result<Activation, Error> {
        activation(unsafe { activateCDKViewer(self.as_raw(), std::ptr::null_mut()) })
    }

    pub fn set_info(&self, title_: &str, lines_: &[&str]) -> Result<(), Error> {
        let title = CString::new(title_)?;
        let line_count = checked_c_int(lines_.len() as u64, "line count")?;
        let line_strings = lines_
            .iter()
            .map(|line| CString::new(*line))
            .collect::<Result<Vec<CString>, NulError>>()?;
        let mut lines = line_strings
            .iter()
            .map(|line| line.as_ptr() as *mut c_char)
            .collect::<Vec<*mut c_char>>();
        let result = unsafe {
            setCDKViewer(
                self.as_raw(),
                title.as_ptr(),
                lines.as_mut_ptr(),
                line_count,
                curdk_sys::A_NORMAL,
                0,
                1,
                self.bx() as i32,
            )
        };
        if result < 0 {
            return Err(Error::InvalidActivation(result));
        }
        Ok(())
    }
}
impl_cdk!(Mentry, CDKMENTRY);
impl Mentry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        width: u32,
        rows: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let width = checked_c_int(width as u64, "field width")?;
        let rows = checked_c_int(rows as u64, "field rows")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKMentry(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                curdk_sys::A_NORMAL,
                ' ' as curdk_sys::chtype,
                curdk_sys::EDisplayType_vHMIXED,
                width,
                rows,
                rows,
                0,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> Result<String, Error> {
        let value = unsafe { activateCDKMentry(self.as_raw(), std::ptr::null_mut()) };
        unsafe { owned_c_string(value, "Mentry value") }
    }

    pub fn value(&self) -> Result<String, Error> {
        let value = unsafe { getCDKMentryValue(self.as_raw()) };
        unsafe { owned_c_string(value, "Mentry value") }
    }

    pub fn set_value(&self, value_: &str) -> Result<(), Error> {
        let value = CString::new(value_)?;
        unsafe { setCDKMentryValue(self.as_raw(), value.as_ptr()) };
        Ok(())
    }
}
impl_cdk!(FScale, CDKFSCALE);
impl FScale {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        field_width: u32,
        start: f32,
        low: f32,
        high: f32,
        increment: f32,
        fast_increment: f32,
        digits: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let field_width = checked_c_int(field_width as u64, "field width")?;
        let digits = checked_c_int(digits as u64, "digits")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKFScale(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                curdk_sys::A_NORMAL,
                field_width,
                start,
                low,
                high,
                increment,
                fast_increment,
                digits,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> f32 {
        unsafe { activateCDKFScale(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn value(&self) -> f32 {
        unsafe { getCDKFScaleValue(self.as_raw()) }
    }

    pub fn set_value(&self, value: f32) {
        unsafe { setCDKFScaleValue(self.as_raw(), value) };
    }

    pub fn range(&self) -> (f32, f32) {
        unsafe {
            (
                getCDKFScaleLowValue(self.as_raw()),
                getCDKFScaleHighValue(self.as_raw()),
            )
        }
    }

    pub fn set_range(&self, low: f32, high: f32) {
        unsafe { setCDKFScaleLowHigh(self.as_raw(), low, high) };
    }

    pub fn digits(&self) -> i32 {
        unsafe { getCDKFScaleDigits(self.as_raw()) }
    }

    pub fn set_digits(&self, digits: u32) -> Result<(), Error> {
        let digits = checked_c_int(digits as u64, "digits")?;
        unsafe { setCDKFScaleDigits(self.as_raw(), digits) };
        Ok(())
    }
}
impl_cdk!(UScale, CDKUSCALE);
impl UScale {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        field_width: u32,
        start: u32,
        low: u32,
        high: u32,
        increment: u32,
        fast_increment: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let field_width = checked_c_int(field_width as u64, "field width")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKUScale(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                curdk_sys::A_NORMAL,
                field_width,
                start,
                low,
                high,
                increment,
                fast_increment,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> u32 {
        unsafe { activateCDKUScale(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn value(&self) -> u32 {
        unsafe { getCDKUScaleValue(self.as_raw()) }
    }

    pub fn set_value(&self, value: u32) {
        unsafe { setCDKUScaleValue(self.as_raw(), value) };
    }

    pub fn range(&self) -> (u32, u32) {
        unsafe {
            (
                getCDKUScaleLowValue(self.as_raw()),
                getCDKUScaleHighValue(self.as_raw()),
            )
        }
    }

    pub fn set_range(&self, low: u32, high: u32) {
        unsafe { setCDKUScaleLowHigh(self.as_raw(), low, high) };
    }
}
impl_cdk!(FSlider, CDKFSLIDER);
impl FSlider {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        field_width: u32,
        start: f32,
        low: f32,
        high: f32,
        increment: f32,
        fast_increment: f32,
        digits: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let field_width = checked_c_int(field_width as u64, "field width")?;
        let digits = checked_c_int(digits as u64, "digits")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKFSlider(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                curdk_sys::A_NORMAL,
                field_width,
                start,
                low,
                high,
                increment,
                fast_increment,
                digits,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> f32 {
        unsafe { activateCDKFSlider(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn value(&self) -> f32 {
        unsafe { getCDKFSliderValue(self.as_raw()) }
    }

    pub fn set_value(&self, value: f32) {
        unsafe { setCDKFSliderValue(self.as_raw(), value) };
    }

    pub fn range(&self) -> (f32, f32) {
        unsafe {
            (
                getCDKFSliderLowValue(self.as_raw()),
                getCDKFSliderHighValue(self.as_raw()),
            )
        }
    }

    pub fn set_range(&self, low: f32, high: f32) {
        unsafe { setCDKFSliderLowHigh(self.as_raw(), low, high) };
    }

    pub fn digits(&self) -> i32 {
        unsafe { getCDKFSliderDigits(self.as_raw()) }
    }

    pub fn set_digits(&self, digits: u32) -> Result<(), Error> {
        let digits = checked_c_int(digits as u64, "digits")?;
        unsafe { setCDKFSliderDigits(self.as_raw(), digits) };
        Ok(())
    }
}
impl_cdk!(USlider, CDKUSLIDER);
impl USlider {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        field_width: u32,
        start: u32,
        low: u32,
        high: u32,
        increment: u32,
        fast_increment: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let field_width = checked_c_int(field_width as u64, "field width")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKUSlider(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                curdk_sys::A_NORMAL,
                field_width,
                start,
                low,
                high,
                increment,
                fast_increment,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> u32 {
        unsafe { activateCDKUSlider(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn value(&self) -> u32 {
        unsafe { getCDKUSliderValue(self.as_raw()) }
    }

    pub fn set_value(&self, value: u32) {
        unsafe { setCDKUSliderValue(self.as_raw(), value) };
    }

    pub fn range(&self) -> (u32, u32) {
        unsafe {
            (
                getCDKUSliderLowValue(self.as_raw()),
                getCDKUSliderHighValue(self.as_raw()),
            )
        }
    }

    pub fn set_range(&self, low: u32, high: u32) {
        unsafe { setCDKUSliderLowHigh(self.as_raw(), low, high) };
    }
}

macro_rules! impl_numeric_widget {
    ($widget:ty, $value:ty) => {
        impl NumericWidget for $widget {
            type Value = $value;

            fn value(&self) -> Self::Value {
                <$widget>::value(self)
            }

            fn set_value(&self, value: Self::Value) {
                <$widget>::set_value(self, value);
            }

            fn range(&self) -> (Self::Value, Self::Value) {
                <$widget>::range(self)
            }

            fn set_range(&self, low: Self::Value, high: Self::Value) {
                <$widget>::set_range(self, low, high);
            }
        }
    };
}

impl_numeric_widget!(Scale, i32);
impl_numeric_widget!(Slider, i32);
impl_numeric_widget!(UScale, u32);
impl_numeric_widget!(USlider, u32);
impl_numeric_widget!(FScale, f32);
impl_numeric_widget!(FSlider, f32);
impl_numeric_widget!(DScale, f64);
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

    pub fn set_values(&self, values: &[i32], start_at_zero: bool) -> Result<(), Error> {
        let count = checked_c_int(values.len() as u64, "value count")?;
        let mut values = values.to_vec();
        let result = unsafe {
            setCDKGraphValues(
                self.as_raw(),
                values.as_mut_ptr(),
                count,
                i32::from(start_at_zero),
            )
        };
        if result < 0 {
            return Err(Error::InvalidActivation(result));
        }
        Ok(())
    }

    pub fn value(&self, index: u32) -> Result<i32, Error> {
        let index = checked_c_int(index as u64, "index")?;
        Ok(unsafe { getCDKGraphValue(self.as_raw(), index) })
    }
}
impl_cdk!(Histogram, CDKHISTOGRAM);
impl Histogram {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        height: u32,
        width: u32,
        orient: u32,
        title_: &str,
        border: Border,
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
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn set_value(&self, low: i32, high: i32, value: i32) {
        unsafe { setCDKHistogramValue(self.as_raw(), low, high, value) };
    }

    pub fn value(&self) -> i32 {
        unsafe { getCDKHistogramValue(self.as_raw()) }
    }

    pub fn range(&self) -> (i32, i32) {
        unsafe {
            (
                getCDKHistogramLowValue(self.as_raw()),
                getCDKHistogramHighValue(self.as_raw()),
            )
        }
    }
}
impl_cdk!(Itemlist, CDKITEMLIST);
impl Itemlist {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        items_: &[&str],
        default_item: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let item_count = checked_c_int(items_.len() as u64, "item count")?;
        let default_item = checked_c_int(default_item as u64, "default item")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        let mut items = CStringArray::new(items_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKItemlist(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                items.as_mut_ptr(),
                item_count,
                default_item,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> i32 {
        unsafe { activateCDKItemlist(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn activate_result(&self) -> Result<Activation, Error> {
        activation(self.activate())
    }

    pub fn current(&self) -> i32 {
        unsafe { getCDKItemlistCurrentItem(self.as_raw()) }
    }

    pub fn set_current(&self, item: u32) -> Result<(), Error> {
        let item = checked_c_int(item as u64, "item index")?;
        unsafe { setCDKItemlistCurrentItem(self.as_raw(), item) };
        Ok(())
    }

    pub fn set_items(&self, items_: &[&str], default_item: u32) -> Result<(), Error> {
        let item_count = checked_c_int(items_.len() as u64, "item count")?;
        let default_item = checked_c_int(default_item as u64, "default item")?;
        let mut items = CStringArray::new(items_)?;
        unsafe {
            setCDKItemlistValues(self.as_raw(), items.as_mut_ptr(), item_count, default_item)
        };
        Ok(())
    }
}
impl_cdk!(Label, CDKLABEL);
impl Label {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        message_: &str,
        border: Border,
    ) -> Result<Self, Error> {
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
                border.boxed_flag(),
                border.shadow_flag(),
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
impl Marquee {
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        width: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let width = checked_c_int(width as u64, "field width")?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKMarquee(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                width,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self, message_: &str, delay: u32, repeat: u32) -> Result<i32, Error> {
        let message = CString::new(message_)?;
        let delay = checked_c_int(delay as u64, "delay")?;
        let repeat = checked_c_int(repeat as u64, "repeat")?;
        Ok(unsafe {
            activateCDKMarquee(
                self.as_raw(),
                message.as_ptr(),
                delay,
                repeat,
                self.bx() as i32,
            )
        })
    }
}
impl_cdk!(Matrix, CDKMATRIX);
impl Matrix {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        view_rows: u32,
        view_cols: u32,
        title_: &str,
        row_titles_: &[&str],
        col_titles_: &[&str],
        col_widths_: &[u32],
        border: Border,
    ) -> Result<Self, Error> {
        if col_titles_.len() != col_widths_.len() {
            return Err(Error::LengthMismatch(
                "matrix column widths",
                col_titles_.len(),
                col_widths_.len(),
            ));
        }
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let view_rows = checked_c_int(view_rows as u64, "view rows")?;
        let view_cols = checked_c_int(view_cols as u64, "view columns")?;
        let actual_rows = checked_c_int(row_titles_.len() as u64, "row count")?;
        let actual_cols = checked_c_int(col_titles_.len() as u64, "column count")?;
        let title = CString::new(title_)?;
        let mut row_titles = one_based_strings(row_titles_)?;
        let mut col_titles = one_based_strings(col_titles_)?;
        let mut col_widths = one_based_ints(col_widths_, "column width")?;
        let mut col_types = vec![0];
        col_types.extend(std::iter::repeat_n(
            EDisplayType_vMIXED as c_int,
            col_titles_.len(),
        ));
        Self::from_raw(cdkscreen, unsafe {
            newCDKMatrix(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                actual_rows,
                actual_cols,
                view_rows,
                view_cols,
                title.as_ptr(),
                row_titles.as_mut_ptr(),
                col_titles.as_mut_ptr(),
                col_widths.as_mut_ptr(),
                col_types.as_mut_ptr(),
                1,
                1,
                ' ' as curdk_sys::chtype,
                ROW as c_int,
                border.boxed_flag(),
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> i32 {
        unsafe { activateCDKMatrix(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn activate_result(&self) -> Result<Activation, Error> {
        activation(self.activate())
    }

    /// Returns the cell at zero-based `(row, col)`.
    pub fn cell(&self, row: u32, col: u32) -> Result<String, Error> {
        let row = checked_c_int(row as u64 + 1, "row")?;
        let col = checked_c_int(col as u64 + 1, "column")?;
        let value = unsafe { getCDKMatrixCell(self.as_raw(), row, col) };
        unsafe { owned_c_string(value, "Matrix cell") }
    }

    /// Sets the cell at zero-based `(row, col)`.
    pub fn set_cell(&self, row: u32, col: u32, value_: &str) -> Result<(), Error> {
        let row = checked_c_int(row as u64 + 1, "row")?;
        let col = checked_c_int(col as u64 + 1, "column")?;
        let value = CString::new(value_)?;
        let result = unsafe { setCDKMatrixCell(self.as_raw(), row, col, value.as_ptr()) };
        if result < 0 {
            return Err(Error::InvalidActivation(result));
        }
        Ok(())
    }

    /// Returns the zero-based current cell position.
    pub fn current(&self) -> (u32, u32) {
        let row = unsafe { getCDKMatrixRow(self.as_raw()) };
        let col = unsafe { getCDKMatrixCol(self.as_raw()) };
        (row.saturating_sub(1) as u32, col.saturating_sub(1) as u32)
    }
}
impl_cdk!(Radio, CDKRADIO);
impl Radio {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        spos: u32,
        height: u32,
        width: u32,
        title_: &str,
        items_: &[&str],
        choice_char: char,
        default_item: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let spos = checked_c_int(spos as u64, "scrollbar position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        let item_count = checked_c_int(items_.len() as u64, "item count")?;
        let default_item = checked_c_int(default_item as u64, "default item")?;
        let title = CString::new(title_)?;
        let mut items = CStringArray::new(items_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKRadio(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                spos,
                height,
                width,
                title.as_ptr(),
                items.as_mut_ptr(),
                item_count,
                choice_char as curdk_sys::chtype,
                default_item,
                curdk_sys::A_NORMAL,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> i32 {
        unsafe { activateCDKRadio(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn activate_result(&self) -> Result<Activation, Error> {
        activation(self.activate())
    }

    pub fn current(&self) -> i32 {
        unsafe { getCDKRadioCurrentItem(self.as_raw()) }
    }

    pub fn set_current(&self, item: u32) -> Result<(), Error> {
        let item = checked_c_int(item as u64, "item index")?;
        unsafe { setCDKRadioCurrentItem(self.as_raw(), item) };
        Ok(())
    }

    pub fn set_items(&self, items_: &[&str]) -> Result<(), Error> {
        let item_count = checked_c_int(items_.len() as u64, "item count")?;
        let mut items = CStringArray::new(items_)?;
        unsafe { setCDKRadioItems(self.as_raw(), items.as_mut_ptr(), item_count) };
        Ok(())
    }
}
impl_cdk!(Scale, CDKSCALE);
impl Scale {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        field_width: u32,
        start: i32,
        low: i32,
        high: i32,
        increment: i32,
        fast_increment: i32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let field_width = checked_c_int(field_width as u64, "field width")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKScale(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                curdk_sys::A_NORMAL,
                field_width,
                start,
                low,
                high,
                increment,
                fast_increment,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> Result<Activation, Error> {
        activation(unsafe { activateCDKScale(self.as_raw(), std::ptr::null_mut()) })
    }

    pub fn value(&self) -> i32 {
        unsafe { getCDKScaleValue(self.as_raw()) }
    }

    pub fn set_value(&self, value: i32) {
        unsafe { setCDKScaleValue(self.as_raw(), value) };
    }

    pub fn range(&self) -> (i32, i32) {
        unsafe {
            (
                getCDKScaleLowValue(self.as_raw()),
                getCDKScaleHighValue(self.as_raw()),
            )
        }
    }

    pub fn set_range(&self, low: i32, high: i32) {
        unsafe { setCDKScaleLowHigh(self.as_raw(), low, high) };
    }
}
impl_cdk!(Scroll, CDKSCROLL);
impl Scroll {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        spos: u32,
        height: u32,
        width: u32,
        title_: &str,
        items_: &[&str],
        numbers: bool,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let spos = checked_c_int(spos as u64, "scrollbar position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        let item_count = checked_c_int(items_.len() as u64, "item count")?;
        let title = CString::new(title_)?;
        let mut items = CStringArray::new(items_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKScroll(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                spos,
                height,
                width,
                title.as_ptr(),
                items.as_mut_ptr(),
                item_count,
                numbers as i32,
                curdk_sys::A_NORMAL,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> i32 {
        unsafe { activateCDKScroll(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn activate_result(&self) -> Result<Activation, Error> {
        activation(self.activate())
    }

    pub fn current(&self) -> i32 {
        unsafe { getCDKScrollCurrent(self.as_raw()) }
    }

    pub fn set_current(&self, item: u32) -> Result<(), Error> {
        let item = checked_c_int(item as u64, "item index")?;
        unsafe { setCDKScrollCurrent(self.as_raw(), item) };
        Ok(())
    }

    pub fn set_items(&self, items_: &[&str], numbers: bool) -> Result<(), Error> {
        let item_count = checked_c_int(items_.len() as u64, "item count")?;
        let mut items = CStringArray::new(items_)?;
        unsafe {
            setCDKScrollItems(
                self.as_raw(),
                items.as_mut_ptr(),
                item_count,
                numbers as i32,
            );
        }
        Ok(())
    }
}
impl_cdk!(Selection, CDKSELECTION);
impl Selection {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        spos: u32,
        height: u32,
        width: u32,
        title_: &str,
        items_: &[&str],
        choice_markers_: &[&str],
        selections_: &[bool],
        border: Border,
    ) -> Result<Self, Error> {
        if items_.len() != selections_.len() {
            return Err(Error::LengthMismatch(
                "selection choices",
                items_.len(),
                selections_.len(),
            ));
        }
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let spos = checked_c_int(spos as u64, "scrollbar position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        let item_count = checked_c_int(items_.len() as u64, "item count")?;
        let title = CString::new(title_)?;
        let item_strings = items_
            .iter()
            .map(|item| CString::new(*item))
            .collect::<Result<Vec<CString>, NulError>>()?;
        let mut items = item_strings
            .iter()
            .map(|item| item.as_ptr() as *mut c_char)
            .collect::<Vec<*mut c_char>>();
        let choice_count = checked_c_int(choice_markers_.len() as u64, "choice marker count")?;
        let choice_strings = choice_markers_
            .iter()
            .map(|choice| CString::new(*choice))
            .collect::<Result<Vec<CString>, NulError>>()?;
        let mut choice_markers = choice_strings
            .iter()
            .map(|choice| choice.as_ptr() as *mut c_char)
            .collect::<Vec<*mut c_char>>();
        let mut selections = selections_
            .iter()
            .map(|selection| i32::from(*selection))
            .collect::<Vec<c_int>>();
        Self::from_raw(cdkscreen, unsafe {
            newCDKSelection(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                spos,
                height,
                width,
                title.as_ptr(),
                items.as_mut_ptr(),
                item_count,
                choice_markers.as_mut_ptr(),
                choice_count,
                curdk_sys::A_NORMAL,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
        .inspect(|selection| {
            unsafe { setCDKSelectionChoices(selection.as_raw(), selections.as_mut_ptr()) };
        })
    }

    pub fn activate(&self) -> i32 {
        unsafe { activateCDKSelection(self.as_raw(), std::ptr::null_mut()) }
    }

    pub fn activate_result(&self) -> Result<Activation, Error> {
        activation(self.activate())
    }

    pub fn current(&self) -> i32 {
        unsafe { getCDKSelectionCurrent(self.as_raw()) }
    }

    pub fn set_current(&self, item: u32) -> Result<(), Error> {
        let item = checked_c_int(item as u64, "item index")?;
        unsafe { setCDKSelectionCurrent(self.as_raw(), item) };
        Ok(())
    }

    pub fn choice(&self, item: u32) -> Result<bool, Error> {
        let item = checked_c_int(item as u64, "item index")?;
        Ok(unsafe { getCDKSelectionChoice(self.as_raw(), item) != 0 })
    }

    pub fn set_choice(&self, item: u32, selected: bool) -> Result<(), Error> {
        let item = checked_c_int(item as u64, "item index")?;
        unsafe { setCDKSelectionChoice(self.as_raw(), item, i32::from(selected)) };
        Ok(())
    }

    pub fn set_items(&self, items_: &[&str]) -> Result<(), Error> {
        let item_count = checked_c_int(items_.len() as u64, "item count")?;
        let item_strings = items_
            .iter()
            .map(|item| CString::new(*item))
            .collect::<Result<Vec<CString>, NulError>>()?;
        let mut items = item_strings
            .iter()
            .map(|item| item.as_ptr() as *mut c_char)
            .collect::<Vec<*mut c_char>>();
        unsafe { setCDKSelectionItems(self.as_raw(), items.as_mut_ptr(), item_count) };
        Ok(())
    }
}
impl_cdk!(Slider, CDKSLIDER);
impl Slider {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        field_width: u32,
        start: i32,
        low: i32,
        high: i32,
        increment: i32,
        fast_increment: i32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let field_width = checked_c_int(field_width as u64, "field width")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKSlider(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                curdk_sys::A_NORMAL,
                field_width,
                start,
                low,
                high,
                increment,
                fast_increment,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> Result<Activation, Error> {
        activation(unsafe { activateCDKSlider(self.as_raw(), std::ptr::null_mut()) })
    }

    pub fn value(&self) -> i32 {
        unsafe { getCDKSliderValue(self.as_raw()) }
    }

    pub fn set_value(&self, value: i32) {
        unsafe { setCDKSliderValue(self.as_raw(), value) };
    }

    pub fn range(&self) -> (i32, i32) {
        unsafe {
            (
                getCDKSliderLowValue(self.as_raw()),
                getCDKSliderHighValue(self.as_raw()),
            )
        }
    }

    pub fn set_range(&self, low: i32, high: i32) {
        unsafe { setCDKSliderLowHigh(self.as_raw(), low, high) };
    }
}
impl_cdk!(Swindow, CDKSWINDOW);
impl Swindow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        height: u32,
        width: u32,
        title_: &str,
        save_lines: u32,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let height = checked_c_int(height as u64, "height")?;
        let width = checked_c_int(width as u64, "width")?;
        let save_lines = checked_c_int(save_lines as u64, "save lines")?;
        let title = CString::new(title_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKSwindow(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                height,
                width,
                title.as_ptr(),
                save_lines,
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) {
        unsafe { activateCDKSwindow(self.as_raw(), std::ptr::null_mut()) };
    }

    pub fn add(&self, info_: &str, insert_pos: u32) -> Result<(), Error> {
        let info = CString::new(info_)?;
        let insert_pos = checked_c_int(insert_pos as u64, "insert position")?;
        unsafe { addCDKSwindow(self.as_raw(), info.as_ptr(), insert_pos) };
        Ok(())
    }

    pub fn set_contents(&self, lines_: &[&str]) -> Result<(), Error> {
        let line_count = checked_c_int(lines_.len() as u64, "line count")?;
        let mut lines = CStringArray::new(lines_)?;
        unsafe { setCDKSwindowContents(self.as_raw(), lines.as_mut_ptr(), line_count) };
        Ok(())
    }

    pub fn jump_to_line(&self, line: u32) -> Result<(), Error> {
        let line = checked_c_int(line as u64, "line")?;
        unsafe { jumpToLineCDKSwindow(self.as_raw(), line) };
        Ok(())
    }

    pub fn clear(&self) {
        unsafe { cleanCDKSwindow(self.as_raw()) };
    }
}
impl_cdk!(Template, CDKTEMPLATE);
impl Template {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cdkscreen: &Screen,
        xpos: u32,
        ypos: u32,
        title_: &str,
        label_: &str,
        plate_: &str,
        overlay_: &str,
        border: Border,
    ) -> Result<Self, Error> {
        let xpos = checked_c_int(xpos as u64, "x position")?;
        let ypos = checked_c_int(ypos as u64, "y position")?;
        let title = CString::new(title_)?;
        let label = CString::new(label_)?;
        let plate = CString::new(plate_)?;
        let overlay = CString::new(overlay_)?;
        Self::from_raw(cdkscreen, unsafe {
            newCDKTemplate(
                cdkscreen.as_raw(),
                xpos,
                ypos,
                title.as_ptr(),
                label.as_ptr(),
                plate.as_ptr(),
                overlay.as_ptr(),
                border.boxed_flag(),
                border.shadow_flag(),
            )
        })
    }

    pub fn activate(&self) -> Result<String, Error> {
        let value = unsafe { activateCDKTemplate(self.as_raw(), std::ptr::null_mut()) };
        unsafe { owned_c_string(value, "Template value") }
    }

    pub fn value(&self) -> Result<String, Error> {
        let value = unsafe { getCDKTemplateValue(self.as_raw()) };
        unsafe { owned_c_string(value, "Template value") }
    }

    pub fn set_value(&self, value_: &str) -> Result<(), Error> {
        let value = CString::new(value_)?;
        unsafe { setCDKTemplateValue(self.as_raw(), value.as_ptr()) };
        Ok(())
    }

    pub fn mix(&self) -> Result<String, Error> {
        let value = unsafe { mixCDKTemplate(self.as_raw()) };
        unsafe { owned_c_string(value, "Template mix") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interior_nul_is_reported() {
        let error = Error::from(CString::new("invalid\0message").unwrap_err());
        assert!(matches!(error, Error::InteriorNul(_)));
    }

    #[test]
    fn border_presets_match_box_and_shadow_flags() {
        assert_eq!(Border::default(), Border::NONE);
        assert_eq!(
            Border::NONE,
            Border {
                boxed: false,
                shadow: false
            }
        );
        assert_eq!(
            Border::BOXED,
            Border {
                boxed: true,
                shadow: false
            }
        );
        assert_eq!(
            Border::SHADOWED,
            Border {
                boxed: true,
                shadow: true
            }
        );
        assert_eq!(Border::NONE.boxed_flag(), 0);
        assert_eq!(Border::BOXED.boxed_flag(), 1);
        assert_eq!(Border::SHADOWED.shadow_flag(), 1);
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
    fn activation_values_are_typed() {
        assert_eq!(activation(2).unwrap(), Activation::Selected(2));
        assert_eq!(activation(-1).unwrap(), Activation::Cancelled);
        assert!(matches!(activation(-2), Err(Error::InvalidActivation(-2))));
    }

    #[test]
    #[ignore = "requires an interactive terminal"]
    fn widget_keeps_screen_alive() -> Result<(), Error> {
        let window = Window::new()?;
        let label = {
            let screen = Screen::new(&window)?;
            Label::new(&screen, CENTER, TOP, "label", Border::NONE)?
        };
        label.set_box(false);
        Ok(())
    }

    #[test]
    #[ignore = "requires an interactive terminal"]
    fn dialog_returns_a_selection() -> Result<(), Error> {
        let window = Window::new()?;
        let screen = Screen::new(&window)?;
        let dialog = Dialog::new(
            &screen,
            CENTER,
            CENTER,
            "Continue?",
            1,
            &["Yes", "No"],
            Border::NONE,
        )?;
        assert!(dialog.activate() >= 0);
        Ok(())
    }
}
