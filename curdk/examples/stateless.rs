fn main() {
    let win = curdk::Window::new();
    let scr = curdk::Screen::new(&win);
    let _lbl = curdk::Label::new(&scr, curdk_sys::CENTER, 5, "text");
    loop {
        scr.refresh();
        std::thread::sleep(std::time::Duration::from_millis(410))
    }
}
