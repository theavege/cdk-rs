#![cfg(target_os = "linux")]

use std::os::fd::RawFd;

fn run_in_terminal(test: fn() -> Result<(), curdk::Error>) {
    unsafe {
        let mut master: RawFd = -1;
        let mut slave: RawFd = -1;
        assert_eq!(
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null()
            ),
            0
        );

        let pid = libc::fork();
        assert!(pid >= 0);
        if pid == 0 {
            libc::close(master);
            for descriptor in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
                libc::dup2(slave, descriptor);
            }
            libc::close(slave);
            libc::setenv(c"TERM".as_ptr(), c"xterm".as_ptr(), 1);
            let status = if test().is_ok() { 0 } else { 1 };
            libc::_exit(status);
        }

        libc::close(slave);
        let mut status = 0;
        assert_eq!(libc::waitpid(pid, &mut status, 0), pid);
        libc::close(master);
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
    }
}

#[test]
fn bare_window_can_be_dropped() {
    run_in_terminal(|| {
        let _window = curdk::Window::new()?;
        Ok(())
    });
}

#[test]
fn widgets_drop_before_their_screen() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let _label = curdk::Label::new(&screen, curdk::CENTER, curdk::TOP, "label")?;
        Ok(())
    });
}

#[test]
fn widget_keeps_screen_alive() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let label = {
            let screen = curdk::Screen::new(&window)?;
            curdk::Label::new(&screen, curdk::CENTER, curdk::TOP, "label")?
        };
        label.set_box(false);
        Ok(())
    });
}

#[test]
fn scroll_supports_items_and_selection() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let scroll = curdk::Scroll::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            curdk::RIGHT,
            5,
            30,
            "Items",
            &["First", "Second"],
            false,
        )?;
        scroll.set_items(&["Updated"], false)?;
        scroll.set_current(0)?;
        assert_eq!(scroll.current(), 0);
        Ok(())
    });
}

#[test]
fn radio_supports_items_and_selection() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let radio = curdk::Radio::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            curdk::RIGHT,
            5,
            30,
            "Choice",
            &["First", "Second"],
            '*',
            0,
        )?;
        radio.set_items(&["Updated", "Another"])?;
        radio.set_current(1)?;
        assert_eq!(radio.current(), 1);
        Ok(())
    });
}

#[test]
fn selection_supports_multiple_choices() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let selection = curdk::Selection::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            curdk::RIGHT,
            5,
            30,
            "Choose",
            &["First", "Second"],
            &["[ ]", "[X]"],
            &[true, false],
        )?;
        selection.set_current(1)?;
        selection.set_choice(1, true)?;
        assert_eq!(selection.current(), 1);
        assert!(selection.choice(0)?);
        assert!(selection.choice(1)?);
        Ok(())
    });
}

#[test]
fn menu_supports_nested_items() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let menu = curdk::Menu::new(
            &screen,
            &[&["File", "Edit"], &["Open", "Quit"]],
            &[(1, 1), (1, 10)],
            0,
        )?;
        menu.set_current(1, 1)?;
        assert_eq!(menu.current(), (1, 1));
        Ok(())
    });
}

#[test]
fn calendar_supports_date_updates() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let calendar =
            curdk::Calendar::new(&screen, curdk::CENTER, curdk::CENTER, "Date", 4, 7, 2026)?;
        calendar.set_date(15, 8, 2027)?;
        assert_eq!(calendar.date(), (15, 8, 2027));
        Ok(())
    });
}

#[test]
fn scale_supports_numeric_updates() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let scale = curdk::Scale::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            "Scale",
            "Value: ",
            8,
            5,
            0,
            10,
            1,
            2,
        )?;
        scale.set_value(7);
        scale.set_range(-5, 20);
        assert_eq!(scale.value(), 7);
        assert_eq!(scale.range(), (-5, 20));
        Ok(())
    });
}

#[test]
fn slider_supports_numeric_updates() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let slider = curdk::Slider::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            "Slider",
            "Value: ",
            8,
            5,
            0,
            10,
            1,
            2,
        )?;
        slider.set_value(7);
        slider.set_range(-5, 20);
        assert_eq!(slider.value(), 7);
        assert_eq!(slider.range(), (-5, 20));
        Ok(())
    });
}

#[test]
fn floating_scale_supports_numeric_updates() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let scale = curdk::FScale::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            "FScale",
            "Value: ",
            8,
            1.5,
            0.0,
            10.0,
            0.5,
            1.0,
            2,
        )?;
        scale.set_value(3.25);
        scale.set_range(-1.0, 20.0);
        scale.set_digits(3)?;
        assert!((scale.value() - 3.25).abs() < f32::EPSILON);
        assert_eq!(scale.range(), (-1.0, 20.0));
        assert_eq!(scale.digits(), 3);
        Ok(())
    });
}

#[test]
fn floating_slider_supports_numeric_updates() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let slider = curdk::FSlider::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            "FSlider",
            "Value: ",
            8,
            1.5,
            0.0,
            10.0,
            0.5,
            1.0,
            2,
        )?;
        slider.set_value(3.25);
        slider.set_range(-1.0, 20.0);
        slider.set_digits(3)?;
        assert!((slider.value() - 3.25).abs() < f32::EPSILON);
        assert_eq!(slider.range(), (-1.0, 20.0));
        assert_eq!(slider.digits(), 3);
        Ok(())
    });
}

#[test]
fn unsigned_scale_supports_numeric_updates() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let scale = curdk::UScale::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            "UScale",
            "Value: ",
            8,
            5,
            0,
            10,
            1,
            2,
        )?;
        scale.set_value(7);
        scale.set_range(2, 20);
        assert_eq!(scale.value(), 7);
        assert_eq!(scale.range(), (2, 20));
        Ok(())
    });
}

#[test]
fn unsigned_slider_supports_numeric_updates() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let slider = curdk::USlider::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            "USlider",
            "Value: ",
            8,
            5,
            0,
            10,
            1,
            2,
        )?;
        slider.set_value(7);
        slider.set_range(2, 20);
        assert_eq!(slider.value(), 7);
        assert_eq!(slider.range(), (2, 20));
        Ok(())
    });
}

#[test]
fn alphalist_supports_items_and_selection() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let alphalist = curdk::Alphalist::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            8,
            24,
            "Cities",
            "City: ",
            &["Helsinki", "Joensuu", "Turku"],
        )?;
        alphalist.set_items(&["Joensuu", "Tampere"])?;
        alphalist.set_current(1)?;
        assert_eq!(alphalist.current(), 1);
        Ok(())
    });
}

#[test]
fn itemlist_supports_items_and_selection() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let itemlist = curdk::Itemlist::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            "Day",
            "Weekday: ",
            &["Mon", "Tue", "Wed"],
            0,
        )?;
        itemlist.set_items(&["Thu", "Fri"], 1)?;
        itemlist.set_current(1)?;
        assert_eq!(itemlist.current(), 1);
        Ok(())
    });
}

#[test]
fn double_scale_supports_numeric_updates() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let scale = curdk::DScale::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            "DScale",
            "Value: ",
            8,
            1.5,
            0.0,
            10.0,
            0.5,
            1.0,
            2,
        )?;
        scale.set_value(3.25);
        scale.set_range(-1.0, 20.0);
        scale.set_digits(3)?;
        assert!((scale.value() - 3.25).abs() < f64::EPSILON);
        assert_eq!(scale.range(), (-1.0, 20.0));
        assert_eq!(scale.digits(), 3);
        Ok(())
    });
}

#[test]
fn marquee_can_be_created() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let _marquee = curdk::Marquee::new(&screen, curdk::CENTER, curdk::CENTER, 24)?;
        Ok(())
    });
}

#[test]
fn matrix_supports_cell_updates() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let matrix = curdk::Matrix::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            2,
            2,
            "Grid",
            &["R1", "R2"],
            &["C1", "C2"],
            &[8, 8],
        )?;
        matrix.set_cell(0, 1, "hello")?;
        assert_eq!(matrix.cell(0, 1)?, "hello");
        Ok(())
    });
}

#[test]
fn swindow_supports_contents() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let swindow = curdk::Swindow::new(&screen, curdk::CENTER, curdk::CENTER, 8, 32, "Log", 32)?;
        swindow.set_contents(&["one", "two"])?;
        swindow.add("three", curdk::BOTTOM)?;
        swindow.jump_to_line(0)?;
        swindow.clear();
        Ok(())
    });
}

#[test]
fn template_supports_value_updates() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let template = curdk::Template::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            "Date",
            "ISO: ",
            "####/##/##",
            "yyyy/mm/dd",
        )?;
        template.set_value("20260923")?;
        assert!(template.value()?.contains("2026"));
        Ok(())
    });
}

#[test]
fn graph_supports_values() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let graph = curdk::Graph::new(
            &screen,
            curdk::CENTER,
            curdk::CENTER,
            8,
            20,
            "Load",
            "t",
            "v",
        )?;
        graph.set_values(&[1, 3, 2], true)?;
        assert_eq!(graph.value(1)?, 3);
        Ok(())
    });
}

#[test]
fn histogram_supports_values() {
    run_in_terminal(|| {
        let window = curdk::Window::new()?;
        let screen = curdk::Screen::new(&window)?;
        let histogram =
            curdk::Histogram::new(&screen, curdk::CENTER, curdk::CENTER, 3, 20, 1, "Usage")?;
        histogram.set_value(0, 100, 40);
        assert_eq!(histogram.value(), 40);
        assert_eq!(histogram.range(), (0, 100));
        Ok(())
    });
}
