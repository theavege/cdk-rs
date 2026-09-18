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
