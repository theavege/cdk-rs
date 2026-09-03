fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let dialog = curdk::Dialog::new(
        &screen,
        curdk::CENTER,
        curdk::CENTER,
        "Continue?",
        1,
        &["Yes", "No"],
    )?;

    let selection = dialog.activate();
    screen.exit();
    println!("Selected button: {selection}");
    Ok(())
}
