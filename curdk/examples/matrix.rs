fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let grid = curdk::Matrix::new(
        &screen,
        curdk::CENTER,
        curdk::CENTER,
        3,
        3,
        "Scores",
        &["Alice", "Bob", "Cara"],
        &["Q1", "Q2", "Q3"],
        &[6, 6, 6],
        curdk::Border::BOXED,
    )?;

    grid.set_cell(0, 0, "10")?;
    grid.set_cell(0, 1, "12")?;
    grid.set_cell(1, 0, "8")?;
    screen.refresh();
    let result = grid.activate_result()?;
    let (row, col) = grid.current();
    let cell = grid.cell(row, col)?;
    screen.exit();
    println!("Activation: {result:?}");
    println!("Last cell ({row}, {col}): {cell}");
    Ok(())
}
