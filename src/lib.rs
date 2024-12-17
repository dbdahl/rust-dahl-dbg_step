use std::io::Stdout;
use std::io::{stdin, stdout, Write};
use std::sync::{Mutex, OnceLock};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

static THRESHOLD: OnceLock<Mutex<(u8, u8)>> = OnceLock::new();

pub fn reset() {
    let mut data = THRESHOLD.get_or_init(|| Mutex::new((0, 0))).lock().unwrap();
    *data = (0, 0);
}

pub fn set(mut print: u8, mut pause: u8, print_takes_precedence: bool) -> (u8, u8) {
    let mut data = THRESHOLD.get_or_init(|| Mutex::new((0, 0))).lock().unwrap();
    if print_takes_precedence && print > pause {
        pause = print;
    } else if !print_takes_precedence && pause < print {
        print = pause;
    }
    pause = pause.clamp(print, 10);
    print = print.clamp(0, pause);
    *data = (print, pause);
    (print, pause)
}

pub fn get() -> (u8, u8) {
    let data = THRESHOLD.get_or_init(|| Mutex::new((0, 0))).lock().unwrap();
    *data
}

pub fn step<T: AsRef<str>, S: FnOnce() -> T>(msg: S, level: u8) {
    let level = level.min(9);
    let (mut print_threshold, mut pause_threshold) = get();
    if level >= print_threshold {
        let mut stdout = stdout().into_raw_mode().unwrap();
        writeln!(stdout, "{{{}}}: {}\r", level, msg().as_ref()).unwrap();
        stdout.flush().unwrap();
        if level < pause_threshold {
            return;
        }
        let show_control_panel =
            |print_threshold, pause_threshold, stdout: &mut RawTerminal<Stdout>| {
                write!(
                    stdout,
                    "\r{{{},{}}}: Keys [0-9] set limit, \u{21E7}[0-9] set pause limit, 'q' quits, ' ' continues ", print_threshold, pause_threshold)
                .unwrap();
                stdout.flush().unwrap();
            };
        'outer: loop {
            let stdin = stdin();
            show_control_panel(print_threshold, pause_threshold, &mut stdout);
            for key in stdin.keys() {
                if let Ok(key) = key {
                    match key {
                        Key::Char('q') | Key::Ctrl('c') => {
                            write!(stdout, "\r\x1B[2K").unwrap();
                            stdout.flush().unwrap();
                            set(10, 10, true);
                            break 'outer;
                        }
                        Key::Char(' ') | Key::Char('\n') => {
                            write!(stdout, "\r\x1B[2K").unwrap();
                            stdout.flush().unwrap();
                            break 'outer;
                        }
                        Key::Char(e) if e >= '0' && e <= '9' => {
                            (print_threshold, pause_threshold) = set(
                                e.to_digit(10).map(|digit| digit as u8).unwrap_or(0),
                                pause_threshold,
                                true,
                            );
                        }
                        Key::Char(e) if e == '!' => {
                            (print_threshold, pause_threshold) = set(print_threshold, 1, false);
                        }
                        Key::Char(e) if e == '@' => {
                            (print_threshold, pause_threshold) = set(print_threshold, 2, false);
                        }
                        Key::Char(e) if e == '#' => {
                            (print_threshold, pause_threshold) = set(print_threshold, 3, false);
                        }
                        Key::Char(e) if e == '$' => {
                            (print_threshold, pause_threshold) = set(print_threshold, 4, false);
                        }
                        Key::Char(e) if e == '%' => {
                            (print_threshold, pause_threshold) = set(print_threshold, 5, false);
                        }
                        Key::Char(e) if e == '^' => {
                            (print_threshold, pause_threshold) = set(print_threshold, 6, false);
                        }
                        Key::Char(e) if e == '&' => {
                            (print_threshold, pause_threshold) = set(print_threshold, 7, false);
                        }
                        Key::Char(e) if e == '*' => {
                            (print_threshold, pause_threshold) = set(print_threshold, 8, false);
                        }
                        Key::Char(e) if e == '(' => {
                            (print_threshold, pause_threshold) = set(print_threshold, 9, false);
                        }
                        _ => {}
                    }
                }
                show_control_panel(print_threshold, pause_threshold, &mut stdout);
            }
        }
    }
}
