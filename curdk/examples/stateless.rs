fn main() {
    let win = curdk::Window::new();
    let scr = curdk::Screen::new(&win);
    curdk::Label::new(&scr, curdk::CENTER, 5, "message").draw();
    curdk::Button::new(&scr, curdk::CENTER, 20, "title").draw();
    curdk::Buttonbox::new(&scr, curdk::CENTER, 12, 1, 20, 1, 2, &["YES", "NOT"]).draw();
    scr.refresh();
    loop {
        //~ scr.refresh();
        std::thread::sleep(std::time::Duration::from_millis(410))
    }
}
