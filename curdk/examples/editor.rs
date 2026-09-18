use std::fs;

struct Model {
    path: Option<String>,
    content: String,
    status: String,
    running: bool,
}

enum Msg {
    Open(String),
    SetContent(String),
    Save,
    Quit,
}

fn update(model: &mut Model, message: Msg) {
    match message {
        Msg::Open(path) => match fs::read_to_string(&path) {
            Ok(content) => {
                model.path = Some(path);
                model.content = content;
                model.status = "Opened".into();
            }
            Err(error) => model.status = format!("Open failed: {error}"),
        },
        Msg::SetContent(content) => model.content = content,
        Msg::Save => match model.path.as_deref() {
            Some(path) => match fs::write(path, &model.content) {
                Ok(()) => model.status = "Saved".into(),
                Err(error) => model.status = format!("Save failed: {error}"),
            },
            None => model.status = "Choose a file first".into(),
        },
        Msg::Quit => model.running = false,
    }
}

fn view(
    model: &Model,
    editor: &curdk::Mentry,
    status: &curdk::Label,
    screen: &curdk::Screen,
) -> Result<(), curdk::Error> {
    editor.set_value(&model.content)?;
    let path = model.path.as_deref().unwrap_or("No file selected");
    status.set_message(&format!("{path} | {}", model.status))?;
    screen.refresh();
    Ok(())
}

fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let file_selector = curdk::Fselect::new(
        &screen,
        curdk::CENTER,
        curdk::CENTER,
        12,
        60,
        "Open file",
        "Path: ",
        ".",
    )?;
    let editor = curdk::Mentry::new(&screen, curdk::CENTER, 2, "Editor", "Text: ", 64, 10)?;
    let status = curdk::Label::new(&screen, curdk::CENTER, curdk::TOP, "No file selected")?;
    let controls = curdk::Buttonbox::new(
        &screen,
        curdk::CENTER,
        curdk::BOTTOM,
        5,
        34,
        1,
        3,
        &["Open", "Save", "Quit"],
    )?;
    let mut model = Model {
        path: None,
        content: String::new(),
        status: "Ready".into(),
        running: true,
    };

    while model.running {
        view(&model, &editor, &status, &screen)?;
        update(&mut model, Msg::SetContent(editor.activate()?));
        match controls.activate_result()? {
            curdk::Activation::Selected(0) => {
                update(&mut model, Msg::Open(file_selector.activate()?))
            }
            curdk::Activation::Selected(1) => update(&mut model, Msg::Save),
            curdk::Activation::Selected(2) | curdk::Activation::Cancelled => {
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

    #[test]
    fn invalid_save_is_reported_without_a_path() {
        let mut model = Model {
            path: None,
            content: "text".into(),
            status: String::new(),
            running: true,
        };
        update(&mut model, Msg::Save);
        assert_eq!(model.status, "Choose a file first");
    }
}
