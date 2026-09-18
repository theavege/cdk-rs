# cdk-rs

![platform](https://img.shields.io/badge/platform-Linux-blue.svg)

Rust bindings for the [Curses Development Kit](https://invisible-island.net/cdk)

## What is CDK [^1]?

CDK  is  a library  of functions which allow a programmer to quickly create a full screen interactive program with ease. The CDK  widgets support the following features:

Ncurses library.
: Instead  of  using the standard curses library, CDK can take advantage of the colors that Ncurses provides.

Key Bindings.
: Individual keys can be overridden with a callback.

Pre and Post Processing.
: Certain  widgets  allow  the user to trap a character before and after the character has been applied to the widget.  This allows programmers to "filter" character input.

Self Test Widgets.
: With the use of the inject function class and the activate function,  programmers  can  have the widgets test themselves.

Special Display Formats.
: There are special character format commands that can be inserted into any string in CDK and the contents will  get  mapped  to  a chtype  (see  the curses manual page) with character attributes. This allows the programmer to insert format types on each  character if they wish.

The Ability To Build Predefined Screens.
: Widgets can be associated to any given screen. If there is more than one screen defined, then CDK has the ability to "flip" from one screen to another with ease.

# Why CDK?

To develop console-based interfaces, there is a library found on all Linux systems called ncurses. ncurses is to console interfaces what Xlib is to graphical interfaces. It contains everything needed to create console-based interfaces while remaining fairly low-level. Although the standard HOWTO collection includes an Ncurses HOWTO, ncurses is nevertheless not a library that can be approached very intuitively, and a great deal of time must be spent before one can program a satisfactory interface.

There is a higher-level and easier-to-use library that brings together numerous pre-built widgets. It is called CDK, an acronym for Curses Development Kit [^1]. CDK is a library written in C. Nevertheless, it was designed in a strongly object-oriented style, with each widget having methods in the form of associated functions. A rigorous definition of the structures and the casting of pointers also provides a semblance of inheritance.

## Dependencies

- [Linux](.github/workflows/make.sh)
- CDK5 development files, ncurses, and libclang

Terminal and widget constructors return `Result` values. Applications should handle
initialization failures and strings containing NUL bytes instead of relying on panics.

## Tutorial

This is the Rust version of the classic CDK workflow: create a curses window,
attach a CDK screen, add widgets, activate them, and let the owners clean up
the terminal when they leave scope.

### 1. Install dependencies

On Debian or Ubuntu:

```sh
sudo apt install cargo clang libcdk5-dev pkg-config
```

`curdk-sys` uses the CDK5 development installation. If `cdk5-config` is not
on `PATH`, set `CDK5_CONFIG` to its full path before building.

### 2. Create a screen

`Window` initializes the curses terminal. `Screen` connects that terminal to
CDK and owns the lifetime required by its widgets:

```rust,ignore
let window = curdk::Window::new()?;
let screen = curdk::Screen::new(&window)?;
# Ok::<(), curdk::Error>(())
```

Keep the `Window` and `Screen` alive while using widgets. Dropping a widget
destroys its native CDK object; dropping the screen restores the terminal.

### 3. Add and activate widgets

The constructors accept Rust strings and return an error if CDK rejects the
native allocation or a string contains an interior NUL byte:

```rust,ignore
let window = curdk::Window::new()?;
let screen = curdk::Screen::new(&window)?;
let label = curdk::Label::new(&screen, curdk::CENTER, curdk::TOP, "Name")?;
let entry = curdk::Entry::new(&screen, curdk::CENTER, curdk::CENTER, "Input", "Name: ")?;

screen.refresh();
let value = entry.activate()?;
label.set_message(&format!("Hello, {value}!"))?;
screen.refresh();
# Ok::<(), curdk::Error>(())
```

Interactive widgets block in `activate` until the user finishes. Buttons,
button boxes, dialogs, menus, lists, and selections also provide
`activate_result()`, which returns `Activation::Selected(index)` or
`Activation::Cancelled` instead of exposing CDK's raw return code.

### 4. Finish the session

Call `screen.exit()` when the application should leave CDK's main loop. The
normal Rust drop order then releases widgets, the screen, and the terminal.

The repository contains complete examples:

```sh
cargo run -p curdk --example counter
cargo run -p curdk --example temperature_converter
cargo run -p curdk --example crud
cargo run -p curdk --example flight_booker
```

### Tutorial, part 2: compose a screen

The second part of the original CDK tutorial demonstrates the important step
after creating one widget: compose a screen from several focused widgets and
let each activation produce an application message. In Rust, keep the
application state separate from CDK handles, and make the event loop the only
place that changes that state.

```rust,ignore
enum Message {
    Search,
    Select(usize),
    Quit,
}

fn update(model: &mut Model, message: Message) {
    match message {
        Message::Search => model.refresh_results(),
        Message::Select(index) => model.selected = index,
        Message::Quit => model.running = false,
    }
}
```

Create widgets once, then update their contents during each view pass. This
avoids recreating native objects and keeps ownership predictable:

```rust,ignore
let results = curdk::Scroll::new(
    &screen,
    curdk::CENTER,
    curdk::CENTER,
    curdk::RIGHT,
    8,
    48,
    "Results",
    &["First result", "Second result"],
    false,
)?;
let actions = curdk::Buttonbox::new(
    &screen,
    curdk::CENTER,
    curdk::BOTTOM,
    5,
    32,
    1,
    2,
    &["Search", "Quit"],
)?;
```

A typical CDK event loop follows this order:

1. Render model state into labels, entries, and lists.
2. Refresh the screen once.
3. Activate the widget that owns the next interaction.
4. Convert its result into a message.
5. Apply `update` and repeat until the model requests `Quit`.

Use `Widget` when a function only needs common operations such as drawing or
changing the box. Use widget-specific methods for values, selections, and
activation. `Scroll`, `Radio`, and `Selection` accept Rust slices, convert
their strings for the duration of the native call, and expose current-item
accessors. `Selection` additionally exposes per-item boolean choices.

For a complete application-shaped version of this pattern, see the
`counter`, `booker`, `crud`, `calculator`, and `temperature_converter`
examples. This section is adapted from the archived
[second CDK tutorial](https://web.archive.org/web/20110825004635/http://www.unixgarden.com/index.php/programmation/tutoriel-cdk-partie-2).

## Other bindings for CDK

- [Ruby](https://github.com/movitto/cdk)

## Alternatives

- [turbo_vision](https://docs.rs/turbo-vision)
- [cursive](https://docs.rs/cursive)

## Work in process

- [ ] [AlphaList](https://invisible-island.net/cdk/manpage/cdk_alphalist.3.html)
- [ ] [Button](https://invisible-island.net/cdk/manpage/cdk_button.3.html)
- [ ] [ButtonBox](https://invisible-island.net/cdk/manpage/cdk_buttonbox.3.html)
- [ ] [Calendar](https://invisible-island.net/cdk/manpage/cdk_calendar.3.html)
- [ ] [Dialog](https://invisible-island.net/cdk/manpage/cdk_dialog.3.html)
- [ ] [Entry](https://invisible-island.net/cdk/manpage/cdk_entry.3.html) -> Entry Field
- [ ] [FSelect](https://invisible-island.net/cdk/manpage/cdk_fselect.3.html) -> File Selector
- [ ] [FViewer](https://invisible-island.net/cdk/manpage/cdk_viewer.3.html) -> File Viewer
- [ ] [Graph](https://invisible-island.net/cdk/manpage/cdk_graph.3.html)
- [ ] [Histogram](https://invisible-island.net/cdk/manpage/cdk_histogram.3.html)
- [ ] [Scale](https://invisible-island.net/cdk/manpage/cdk_scale.3.html) -> Integer Scale
    - [ ] [DScale](https://invisible-island.net/cdk/manpage/cdk_dscale.3.html) -> DoubleFloat Scale
    - [ ] [FScale](https://invisible-island.net/cdk/manpage/cdk_fscale.3.html) -> Floating Scale
    - [ ] [UScale](https://invisible-island.net/cdk/manpage/cdk_uscale.3.html) -> Unsigned Scale
- [ ] [Slider](https://invisible-island.net/cdk/manpage/cdk_slider.3.html) -> Integer Slider
    - [ ] [FSlider](https://invisible-island.net/cdk/manpage/cdk_fslider.3.html) -> Floating Slider
    - [ ] [USlider](https://invisible-island.net/cdk/manpage/cdk_uslider.3.html) -> Unsigned Slider
- [ ] [ItemList](https://invisible-island.net/cdk/manpage/cdk_itemlist.3.html)
- [ ] [Label](https://invisible-island.net/cdk/manpage/cdk_label.3.html)
- [ ] [Marquee](https://invisible-island.net/cdk/manpage/cdk_marquee.3.html)
- [ ] [Matrix](https://invisible-island.net/cdk/manpage/cdk_matrix.3.html)
- [ ] [Mentry](https://invisible-island.net/cdk/manpage/cdk_mentry.3.html) -> Multiple Line Entry Field
- [ ] [Menu](https://invisible-island.net/cdk/manpage/cdk_menu.3.html) -> Pulldown Menu
- [ ] [Radio](https://invisible-island.net/cdk/manpage/cdk_radio.3.html) -> Radio List
- [ ] [Scroll](https://invisible-island.net/cdk/manpage/cdk_scroll.3.html) -> Scrolling List
- [ ] [Selection](https://invisible-island.net/cdk/manpage/cdk_scroll.3.html) -> Scrolling Selection List
- [ ] [SWindow](https://invisible-island.net/cdk/manpage/cdk_scroll.3.html) -> Scrolling Window

[^1]: [Curses Development Kit](https://invisible-island.net/cdk).
