#[derive(Clone)]
struct Contact {
    name: String,
    email: String,
}

struct Model {
    contacts: Vec<Contact>,
    visible: Vec<usize>,
    current: usize,
    name: String,
    email: String,
    running: bool,
}

enum Msg {
    SetName(String),
    SetEmail(String),
    Search,
    Create,
    Update,
    Delete,
    Select(usize),
    Quit,
}

fn update(model: &mut Model, message: Msg) {
    match message {
        Msg::SetName(name) => model.name = name,
        Msg::SetEmail(email) => model.email = email,
        Msg::Search => {
            let name = model.name.trim().to_lowercase();
            let email = model.email.trim().to_lowercase();
            model.visible = model
                .contacts
                .iter()
                .enumerate()
                .filter(|(_, contact)| {
                    contact.name.to_lowercase().contains(&name)
                        && contact.email.to_lowercase().contains(&email)
                })
                .map(|(index, _)| index)
                .collect();
            model.current = 0;
        }
        Msg::Create => {
            if !model.name.trim().is_empty() && !model.email.trim().is_empty() {
                model.contacts.push(Contact {
                    name: model.name.trim().into(),
                    email: model.email.trim().into(),
                });
                model.visible = (0..model.contacts.len()).collect();
                model.current = model.visible.len().saturating_sub(1);
            }
        }
        Msg::Update => {
            if let Some(&index) = model.visible.get(model.current)
                && !model.name.trim().is_empty()
                && !model.email.trim().is_empty()
            {
                model.contacts[index] = Contact {
                    name: model.name.trim().into(),
                    email: model.email.trim().into(),
                };
            }
        }
        Msg::Delete => {
            if let Some(&index) = model.visible.get(model.current) {
                model.contacts.remove(index);
                model.visible = (0..model.contacts.len()).collect();
                model.current = model.current.min(model.visible.len().saturating_sub(1));
            }
        }
        Msg::Select(index) => {
            if index < model.visible.len() {
                model.current = index;
                if let Some(&contact) = model.visible.get(index) {
                    model.name = model.contacts[contact].name.clone();
                    model.email = model.contacts[contact].email.clone();
                }
            }
        }
        Msg::Quit => model.running = false,
    }
}

fn view(
    model: &Model,
    name: &curdk::Entry,
    email: &curdk::Entry,
    results: &curdk::Selection,
    screen: &curdk::Screen,
) -> Result<(), curdk::Error> {
    name.set_value(&model.name)?;
    email.set_value(&model.email)?;
    let items: Vec<String> = if model.visible.is_empty() {
        vec!["No contacts".into()]
    } else {
        model
            .visible
            .iter()
            .map(|&index| {
                let contact = &model.contacts[index];
                format!("{} - {}", contact.name, contact.email)
            })
            .collect()
    };
    let item_refs: Vec<&str> = items.iter().map(String::as_str).collect();
    results.set_items(&item_refs)?;
    results.set_current(model.current as u32)?;
    screen.refresh();
    Ok(())
}

fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let name = curdk::Entry::new(&screen, curdk::CENTER, 2, "Name", "Name: ")?;
    let email = curdk::Entry::new(&screen, curdk::CENTER, 6, "Email", "Email: ")?;
    let results = curdk::Selection::new(
        &screen,
        curdk::CENTER,
        10,
        curdk::RIGHT,
        8,
        58,
        "Contacts",
        &["Ada Lovelace - ada@example.com"],
        &["[ ]", "[X]"],
        &[false],
    )?;
    let controls = curdk::Buttonbox::new(
        &screen,
        curdk::CENTER,
        curdk::BOTTOM,
        5,
        58,
        1,
        5,
        &["Search", "Create", "Update", "Delete", "Quit"],
    )?;
    let contacts = vec![Contact {
        name: "Ada Lovelace".into(),
        email: "ada@example.com".into(),
    }];
    let mut model = Model {
        visible: vec![0],
        contacts,
        current: 0,
        name: String::new(),
        email: String::new(),
        running: true,
    };

    while model.running {
        view(&model, &name, &email, &results, &screen)?;
        if let curdk::Activation::Selected(index) = results.activate_result()? {
            update(&mut model, Msg::Select(index));
        }
        update(&mut model, Msg::SetName(name.activate()?));
        update(&mut model, Msg::SetEmail(email.activate()?));
        match controls.activate_result()? {
            curdk::Activation::Selected(0) => update(&mut model, Msg::Search),
            curdk::Activation::Selected(1) => update(&mut model, Msg::Create),
            curdk::Activation::Selected(2) => update(&mut model, Msg::Update),
            curdk::Activation::Selected(3) => update(&mut model, Msg::Delete),
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

    #[test]
    fn create_and_search_contacts() {
        let mut model = Model {
            contacts: Vec::new(),
            visible: Vec::new(),
            current: 0,
            name: "Grace Hopper".into(),
            email: "grace@example.com".into(),
            running: true,
        };
        update(&mut model, Msg::Create);
        update(&mut model, Msg::SetName("grace".into()));
        update(&mut model, Msg::SetEmail(String::new()));
        update(&mut model, Msg::Search);
        assert_eq!(model.visible, vec![0]);
    }
}
