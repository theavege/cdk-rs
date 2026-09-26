#[derive(Debug)]
struct Model {
    celsius: String,
    fahrenheit: String,
    running: bool,
}

enum Msg {
    SetCelsius(String),
    SetFahrenheit(String),
    Quit,
}

fn update(model: &mut Model, message: Msg) {
    match message {
        Msg::SetCelsius(value) => {
            if let Ok(celsius) = value.trim().parse::<f64>() {
                model.celsius = value;
                model.fahrenheit = format!("{:.2}", celsius * 9.0 / 5.0 + 32.0);
            }
        }
        Msg::SetFahrenheit(value) => {
            if let Ok(fahrenheit) = value.trim().parse::<f64>() {
                model.fahrenheit = value;
                model.celsius = format!("{:.2}", (fahrenheit - 32.0) * 5.0 / 9.0);
            }
        }
        Msg::Quit => model.running = false,
    }
}

fn view(
    model: &Model,
    celsius: &curdk::Entry,
    fahrenheit: &curdk::Entry,
    screen: &curdk::Screen,
) -> Result<(), curdk::Error> {
    celsius.set_value(&model.celsius)?;
    fahrenheit.set_value(&model.fahrenheit)?;
    screen.refresh();
    Ok(())
}

fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let celsius = curdk::Entry::new(
        &screen,
        curdk::CENTER,
        8,
        "Celsius",
        "C: ",
        curdk::Border::NONE,
    )?;
    let fahrenheit = curdk::Entry::new(
        &screen,
        curdk::CENTER,
        12,
        "Fahrenheit",
        "F: ",
        curdk::Border::NONE,
    )?;
    let controls = curdk::Buttonbox::new(
        &screen,
        curdk::CENTER,
        curdk::BOTTOM,
        5,
        24,
        1,
        1,
        &["Quit"],
        curdk::Border::NONE,
    )?;
    let mut model = Model {
        celsius: "0.00".into(),
        fahrenheit: "32.00".into(),
        running: true,
    };

    while model.running {
        view(&model, &celsius, &fahrenheit, &screen)?;
        update(&mut model, Msg::SetCelsius(celsius.activate()?));
        view(&model, &celsius, &fahrenheit, &screen)?;
        update(&mut model, Msg::SetFahrenheit(fahrenheit.activate()?));
        view(&model, &celsius, &fahrenheit, &screen)?;
        if matches!(
            controls.activate_result()?,
            curdk::Activation::Cancelled | curdk::Activation::Selected(_)
        ) {
            update(&mut model, Msg::Quit);
        }
    }

    screen.exit();
    Ok(())
}
