fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let date = curdk::Template::new(
        &screen,
        curdk::CENTER,
        curdk::CENTER,
        "ISO date",
        "Date: ",
        "####/##/##",
        "yyyy/mm/dd",
        curdk::Border::BOXED,
    )?;

    date.set_value("20260925")?;
    screen.refresh();
    let value = date.activate()?;
    let mixed = date.mix()?;
    screen.exit();
    println!("Template value: {value}");
    println!("Mixed overlay: {mixed}");
    Ok(())
}
