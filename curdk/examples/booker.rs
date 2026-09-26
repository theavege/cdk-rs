#[derive(Clone, Copy)]
struct Book {
    title: &'static str,
    author: &'static str,
}

const BOOKS: &[Book] = &[
    Book {
        title: "The Left Hand of Darkness",
        author: "Ursula K. Le Guin",
    },
    Book {
        title: "The Dispossessed",
        author: "Ursula K. Le Guin",
    },
    Book {
        title: "Dune",
        author: "Frank Herbert",
    },
    Book {
        title: "Kindred",
        author: "Octavia E. Butler",
    },
    Book {
        title: "The Fifth Season",
        author: "N. K. Jemisin",
    },
];

struct Model {
    title_query: String,
    author_query: String,
    results: Vec<String>,
    current: usize,
    running: bool,
}

enum Msg {
    SetTitle(String),
    SetAuthor(String),
    Search,
    Next,
    Previous,
    Quit,
}

fn update(model: &mut Model, message: Msg) {
    match message {
        Msg::SetTitle(title) => model.title_query = title,
        Msg::SetAuthor(author) => model.author_query = author,
        Msg::Search => {
            let title_query = model.title_query.trim().to_lowercase();
            let author_query = model.author_query.trim().to_lowercase();
            model.results = BOOKS
                .iter()
                .filter(|book| {
                    book.title.to_lowercase().contains(&title_query)
                        && book.author.to_lowercase().contains(&author_query)
                })
                .map(|book| format!("{} - {}", book.title, book.author))
                .collect();
            model.current = 0;
        }
        Msg::Next => {
            if !model.results.is_empty() {
                model.current = (model.current + 1) % model.results.len();
            }
        }
        Msg::Previous => {
            if !model.results.is_empty() {
                model.current = model
                    .current
                    .checked_sub(1)
                    .unwrap_or(model.results.len() - 1);
            }
        }
        Msg::Quit => model.running = false,
    }
}

fn view(
    model: &Model,
    title: &curdk::Entry,
    author: &curdk::Entry,
    results: &curdk::Selection,
    screen: &curdk::Screen,
) -> Result<(), curdk::Error> {
    title.set_value(&model.title_query)?;
    author.set_value(&model.author_query)?;
    let items: Vec<&str> = if model.results.is_empty() {
        vec!["No matching books"]
    } else {
        model.results.iter().map(String::as_str).collect()
    };
    results.set_items(&items)?;
    results.set_current(model.current as u32)?;
    screen.refresh();
    Ok(())
}

fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let title = curdk::Entry::new(
        &screen,
        curdk::CENTER,
        2,
        "Title",
        "Title: ",
        curdk::Border::NONE,
    )?;
    let author = curdk::Entry::new(
        &screen,
        curdk::CENTER,
        6,
        "Author",
        "Author: ",
        curdk::Border::NONE,
    )?;
    let initial_items: Vec<&str> = BOOKS.iter().map(|book| book.title).collect();
    let results = curdk::Selection::new(
        &screen,
        curdk::CENTER,
        10,
        curdk::RIGHT,
        8,
        54,
        "Results",
        &initial_items,
        &["[ ]", "[X]"],
        &vec![false; initial_items.len()],
        curdk::Border::NONE,
    )?;
    let controls = curdk::Buttonbox::new(
        &screen,
        curdk::CENTER,
        curdk::BOTTOM,
        5,
        42,
        1,
        4,
        &["Search", "Previous", "Next", "Quit"],
        curdk::Border::NONE,
    )?;
    let mut model = Model {
        title_query: String::new(),
        author_query: String::new(),
        results: Vec::new(),
        current: 0,
        running: true,
    };

    while model.running {
        view(&model, &title, &author, &results, &screen)?;
        update(&mut model, Msg::SetTitle(title.activate()?));
        update(&mut model, Msg::SetAuthor(author.activate()?));
        match controls.activate_result()? {
            curdk::Activation::Selected(0) => update(&mut model, Msg::Search),
            curdk::Activation::Selected(1) => update(&mut model, Msg::Previous),
            curdk::Activation::Selected(2) => update(&mut model, Msg::Next),
            curdk::Activation::Selected(3) | curdk::Activation::Cancelled => {
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
    fn search_filters_by_title_and_author() {
        let mut model = Model {
            title_query: "dune".into(),
            author_query: "herbert".into(),
            results: Vec::new(),
            current: 0,
            running: true,
        };
        update(&mut model, Msg::Search);
        assert_eq!(model.results, vec!["Dune - Frank Herbert"]);
    }
}
