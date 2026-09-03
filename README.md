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
