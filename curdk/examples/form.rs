fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let help = curdk::Label::new(
        &screen,
        curdk::CENTER,
        curdk::TOP,
        "Tab moves between fields. Ctrl-X submits.",
        curdk::Border::NONE,
    )?;
    let name = curdk::Entry::new(
        &screen,
        curdk::CENTER,
        3,
        "Name",
        "Name: ",
        curdk::Border::BOXED,
    )?;
    let city = curdk::Alphalist::new(
        &screen,
        curdk::CENTER,
        7,
        8,
        36,
        "City",
        "City: ",
        &["Helsinki", "Joensuu", "Tampere", "Turku"],
        curdk::Border::BOXED,
    )?;
    let done = screen.clone();
    name.bind_key(b'x' as u32 & 0x1f, move |_| {
        done.exit();
        1
    })?;

    help.draw();
    name.set_value("Ada")?;
    screen.refresh();
    let accepted = screen.traverse();
    let name_value = name.value()?;
    let city_value = city.current();
    println!("Accepted: {accepted}");
    println!("Name: {name_value}");
    println!("City index: {city_value}");
    Ok(())
}
