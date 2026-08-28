fn main() {
    let win = curdk::Window::new();
    let scr = curdk::Screen::new(&win);
    let _lbl = curdk::Label::new(&scr, curdk::CENTER, 5, "message");
    let _btn = curdk::Button::new(&scr, curdk::CENTER, 20, "title");
    let _btns = curdk::Buttonbox::new(&scr, curdk::CENTER, 12, 1, 20, 1, 2, &["YES", "NOT"]);
    scr.refresh();
    loop {
        //~ scr.refresh();
        std::thread::sleep(std::time::Duration::from_millis(410))
    }
}
