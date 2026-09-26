fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let log = curdk::Swindow::new(
        &screen,
        curdk::CENTER,
        curdk::CENTER,
        10,
        48,
        "Build log",
        64,
        curdk::Border::BOXED,
    )?;

    log.set_contents(&[
        "compiling curdk-sys v0.0.1",
        "compiling curdk v0.0.1",
        "Finished `dev` profile",
    ])?;
    log.add("running 23 tests", curdk::BOTTOM)?;
    log.add("test result: ok.", curdk::BOTTOM)?;
    screen.refresh();
    log.activate();
    screen.exit();
    Ok(())
}
