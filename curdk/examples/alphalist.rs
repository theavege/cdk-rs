fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let cities = curdk::Alphalist::new(
        &screen,
        curdk::CENTER,
        curdk::CENTER,
        12,
        32,
        "Cities",
        "City: ",
        &["Helsinki", "Joensuu", "Oulu", "Tampere", "Turku", "Vaasa"],
        curdk::Border::BOXED,
    )?;

    screen.refresh();
    let selected = cities.activate()?;
    screen.exit();
    println!("Selected city: {selected}");
    Ok(())
}
