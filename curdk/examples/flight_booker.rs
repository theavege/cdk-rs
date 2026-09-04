#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Date {
    year: i32,
    month: i32,
    day: i32,
}

impl Date {
    fn display(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug)]
struct Model {
    departure: Date,
    return_date: Date,
    round_trip: bool,
    status: String,
    running: bool,
}

enum Msg {
    SetDeparture(Date),
    SetReturn(Date),
    SetRoundTrip(bool),
    Book,
    Quit,
}

fn update(model: &mut Model, message: Msg) {
    match message {
        Msg::SetDeparture(date) => model.departure = date,
        Msg::SetReturn(date) => model.return_date = date,
        Msg::SetRoundTrip(round_trip) => model.round_trip = round_trip,
        Msg::Book => {
            model.status = if model.round_trip && model.return_date < model.departure {
                "Return date must not precede departure".into()
            } else {
                format!(
                    "Booked {} flight: {} to {}",
                    if model.round_trip {
                        "round-trip"
                    } else {
                        "one-way"
                    },
                    model.departure.display(),
                    model.return_date.display()
                )
            };
        }
        Msg::Quit => model.running = false,
    }
}

fn calendar_date(calendar: &curdk::Calendar) -> Date {
    let (day, month, year) = calendar.date();
    Date { year, month, day }
}

fn view(model: &Model, status: &curdk::Label, screen: &curdk::Screen) -> Result<(), curdk::Error> {
    status.set_message(&format!(
        "{} | Departure: {} | Return: {}",
        model.status,
        model.departure.display(),
        model.return_date.display()
    ))?;
    screen.refresh();
    Ok(())
}

fn main() -> Result<(), curdk::Error> {
    let window = curdk::Window::new()?;
    let screen = curdk::Screen::new(&window)?;
    let departure = curdk::Calendar::new(&screen, 4, 3, "Departure", 1, 1, 2026)?;
    let return_date = curdk::Calendar::new(&screen, 42, 3, "Return", 1, 1, 2026)?;
    let trip_type = curdk::Radio::new(
        &screen,
        curdk::CENTER,
        16,
        curdk::RIGHT,
        4,
        30,
        "Trip type",
        &["One-way", "Round-trip"],
        '*',
        0,
    )?;
    let status = curdk::Label::new(&screen, curdk::CENTER, curdk::TOP, "Ready")?;
    let actions = curdk::Buttonbox::new(
        &screen,
        curdk::CENTER,
        curdk::BOTTOM,
        5,
        30,
        1,
        2,
        &["Book", "Quit"],
    )?;
    let mut model = Model {
        departure: calendar_date(&departure),
        return_date: calendar_date(&return_date),
        round_trip: false,
        status: "Ready".into(),
        running: true,
    };

    while model.running {
        view(&model, &status, &screen)?;
        departure.activate();
        update(&mut model, Msg::SetDeparture(calendar_date(&departure)));
        return_date.activate();
        update(&mut model, Msg::SetReturn(calendar_date(&return_date)));
        trip_type.activate();
        if trip_type.current() == 1 {
            update(&mut model, Msg::SetRoundTrip(true));
        } else {
            update(&mut model, Msg::SetRoundTrip(false));
        }
        match actions.activate_result()? {
            curdk::Activation::Selected(0) => update(&mut model, Msg::Book),
            curdk::Activation::Selected(1) | curdk::Activation::Cancelled => {
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

    fn model() -> Model {
        Model {
            departure: Date {
                year: 2026,
                month: 7,
                day: 10,
            },
            return_date: Date {
                year: 2026,
                month: 7,
                day: 12,
            },
            round_trip: true,
            status: String::new(),
            running: true,
        }
    }

    #[test]
    fn booking_rejects_an_early_return() {
        let mut model = model();
        update(
            &mut model,
            Msg::SetReturn(Date {
                year: 2026,
                month: 7,
                day: 9,
            }),
        );
        update(&mut model, Msg::Book);
        assert_eq!(model.status, "Return date must not precede departure");
    }

    #[test]
    fn booking_accepts_one_way_flights() {
        let mut model = model();
        update(&mut model, Msg::SetRoundTrip(false));
        update(&mut model, Msg::Book);
        assert!(model.status.starts_with("Booked one-way flight"));
    }
}
