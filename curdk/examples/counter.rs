#[derive(Debug)]
struct Model {
    value: i32,
    running: bool,
}

enum Msg {
    SetValue(String),
    Increment,
    Decrement,
    Quit,
}

fn update(model: &mut Model, message: Msg) {
    match message {
        Msg::SetValue(value) => {
            if let Ok(value) = value.trim().parse() {
                model.value = value;
            }
        }
        Msg::Increment => model.value += 1,
        Msg::Decrement => model.value -= 1,
        Msg::Quit => model.running = false,
    }
}

fn view(
    model: &Model,
    label: &curdk::Label,
    entry: &curdk::Entry,
    screen: &curdk::Screen,
) -> Result<(), curdk::Error> {
    label.set_message(&format!("</B/24>Counter: {}<!B!24>", model.value))?;
    entry.set_value(&model.value.to_string())?;
    screen.refresh();
    Ok(())
}

fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let label = curdk::Label::new(&screen, curdk::CENTER, curdk::TOP, "Counter: 0")?;
    let entry = curdk::Entry::new(&screen, curdk::CENTER, curdk::CENTER, "Value", "Value: ")?;
    let controls = curdk::Buttonbox::new(
        &screen,
        curdk::CENTER,
        curdk::BOTTOM,
        5,
        32,
        1,
        3,
        &["-", "+", "Quit"],
    )?;
    let mut model = Model {
        value: 0,
        running: true,
    };

    while model.running {
        view(&model, &label, &entry, &screen)?;
        update(&mut model, Msg::SetValue(entry.activate()?));
        view(&model, &label, &entry, &screen)?;

        match controls.activate_result()? {
            curdk::Activation::Selected(0) => update(&mut model, Msg::Decrement),
            curdk::Activation::Selected(1) => update(&mut model, Msg::Increment),
            curdk::Activation::Selected(2) | curdk::Activation::Cancelled => {
                update(&mut model, Msg::Quit)
            }
            curdk::Activation::Selected(_) => update(&mut model, Msg::Quit),
        }
    }

    screen.exit();
    Ok(())
}
