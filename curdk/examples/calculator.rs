#[derive(Debug)]
struct Model {
    first: String,
    second: String,
    result: String,
    running: bool,
}

enum Msg {
    SetFirst(String),
    SetSecond(String),
    Add,
    Subtract,
    Multiply,
    Divide,
    Quit,
}

fn calculate(model: &mut Model, operation: impl FnOnce(f64, f64) -> Option<f64>) {
    let Ok(first) = model.first.trim().parse::<f64>() else {
        model.result = "Invalid first operand".into();
        return;
    };
    let Ok(second) = model.second.trim().parse::<f64>() else {
        model.result = "Invalid second operand".into();
        return;
    };
    model.result = operation(first, second)
        .map(|result| format!("{result:.2}"))
        .unwrap_or_else(|| "Division by zero".into());
}

fn update(model: &mut Model, message: Msg) {
    match message {
        Msg::SetFirst(value) => model.first = value,
        Msg::SetSecond(value) => model.second = value,
        Msg::Add => calculate(model, |first, second| Some(first + second)),
        Msg::Subtract => calculate(model, |first, second| Some(first - second)),
        Msg::Multiply => calculate(model, |first, second| Some(first * second)),
        Msg::Divide => calculate(model, |first, second| {
            (second != 0.0).then_some(first / second)
        }),
        Msg::Quit => model.running = false,
    }
}

fn view(
    model: &Model,
    first: &curdk::Entry,
    second: &curdk::Entry,
    result: &curdk::Label,
    screen: &curdk::Screen,
) -> Result<(), curdk::Error> {
    first.set_value(&model.first)?;
    second.set_value(&model.second)?;
    result.set_message(&format!("</B/24>Result: {}<!B!24>", model.result))?;
    screen.refresh();
    Ok(())
}

fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let first = curdk::Entry::new(&screen, curdk::CENTER, 4, "First", "A: ")?;
    let second = curdk::Entry::new(&screen, curdk::CENTER, 8, "Second", "B: ")?;
    let result = curdk::Label::new(&screen, curdk::CENTER, 12, "Result: 0.00")?;
    let controls = curdk::Buttonbox::new(
        &screen,
        curdk::CENTER,
        curdk::BOTTOM,
        5,
        42,
        1,
        5,
        &["+", "-", "*", "/", "Quit"],
    )?;
    let mut model = Model {
        first: "0".into(),
        second: "0".into(),
        result: "0.00".into(),
        running: true,
    };

    while model.running {
        view(&model, &first, &second, &result, &screen)?;
        update(&mut model, Msg::SetFirst(first.activate()?));
        update(&mut model, Msg::SetSecond(second.activate()?));
        match controls.activate_result()? {
            curdk::Activation::Selected(0) => update(&mut model, Msg::Add),
            curdk::Activation::Selected(1) => update(&mut model, Msg::Subtract),
            curdk::Activation::Selected(2) => update(&mut model, Msg::Multiply),
            curdk::Activation::Selected(3) => update(&mut model, Msg::Divide),
            curdk::Activation::Selected(4) | curdk::Activation::Cancelled => {
                update(&mut model, Msg::Quit)
            }
            curdk::Activation::Selected(_) => update(&mut model, Msg::Quit),
        }
    }

    screen.exit();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(first: &str, second: &str) -> Model {
        Model {
            first: first.into(),
            second: second.into(),
            result: String::new(),
            running: true,
        }
    }

    #[test]
    fn calculates_arithmetic_operations() {
        let mut model = model("6", "2");
        update(&mut model, Msg::Add);
        assert_eq!(model.result, "8.00");
        update(&mut model, Msg::Divide);
        assert_eq!(model.result, "3.00");
    }

    #[test]
    fn reports_invalid_input_and_zero_division() {
        let mut model = model("6", "0");
        update(&mut model, Msg::Divide);
        assert_eq!(model.result, "Division by zero");
        model.first = "nope".into();
        update(&mut model, Msg::Add);
        assert_eq!(model.result, "Invalid first operand");
    }
}
