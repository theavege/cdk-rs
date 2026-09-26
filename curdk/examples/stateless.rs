fn main() -> Result<(), curdk::Error> {
    let win = curdk::Window::new()?;
    let scr = curdk::Screen::new(&win)?;
    let mut val = 0;
    let lbl = curdk::Label::new(
        &scr,
        curdk::CENTER,
        curdk::TOP,
        "</B/24>Enter a value<!B!24>",
        curdk::Border::NONE,
    )?;
    let entry = curdk::Entry::new(
        &scr,
        curdk::CENTER,
        curdk::CENTER,
        "Value",
        "Input: ",
        curdk::Border::NONE,
    )?;
    entry.set_value("initial")?;
    let btn = curdk::Button::new(
        &scr,
        curdk::CENTER,
        curdk::BOTTOM,
        "Button",
        curdk::Border::NONE,
    )?;
    scr.refresh();
    let value = entry.activate()?;
    lbl.set_message(&format!("</B/24>{value}<!B!24>"))?;
    scr.refresh();
    let mut rslt = 0;
    while rslt != -1 {
        //~ std::thread::sleep(std::time::Duration::from_millis(410))
        rslt = btn.activate();
        val += 1;
        lbl.set_message(&format!("</B/48>{val}<!B!48>"))?;
        scr.refresh();
    }
    Ok(())
}
