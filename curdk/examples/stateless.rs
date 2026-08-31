fn main() {
    let win = curdk::Window::new();
    let scr = curdk::Screen::new(&win);
    let mut val = 0;
    let lbl = curdk::Label::new(
        &scr,
        curdk::CENTER,
        curdk::TOP,
        &format!("</B/48>{val}<!B!48>"),
    );
    let btn = curdk::Button::new(&scr, curdk::CENTER, curdk::BOTTOM, "Button");
    scr.refresh();
    //~ lbl.wait('q');
    let mut rslt = 0;
    while rslt != -1 {
        //~ std::thread::sleep(std::time::Duration::from_millis(410))
        rslt = btn.activate();
        val += 1;
        lbl.set_message(&format!("</B/48>{val}<!B!48>"));
        scr.refresh();
    }
}
