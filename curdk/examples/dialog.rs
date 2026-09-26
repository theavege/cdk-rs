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
        curdk::Border::NONE,
    )?;

    let selection = dialog.activate_result()?;
    screen.exit();
    println!("Dialog result: {selection:?}");
    Ok(())
}
