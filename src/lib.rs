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
        writeln!(stdout, "<{}> {}\r", level, msg().as_ref()).unwrap();
        stdout.flush().unwrap();
        if level < pause_threshold {
            return;
        }
        let show_control_panel =
            |print_threshold, pause_threshold, stdout: &mut RawTerminal<Stdout>| {
                write!(
                    stdout,
                    "DBG_STEP(Print:{}, Pause:{}): < > Continue, [0-9] Set print threshold, Shift-[0-9] Set pause threshold ", print_threshold, pause_threshold)
                .unwrap();
                stdout.flush().unwrap();
            };
        'outer: loop {
            let stdin = stdin();
            show_control_panel(print_threshold, pause_threshold, &mut stdout);
            for key in stdin.keys() {
                if let Ok(key) = key {
                    match key {
                        Key::Ctrl('c') => {
                            writeln!(stdout, "Ctrl-c\r").unwrap();
                            set(10, 10, true);
                            break 'outer;
                        }
                        Key::Char(' ') | Key::Char('\n') => {
                            writeln!(stdout, "\r").unwrap();
                            break 'outer;
                        }
                        Key::Char(e) if e >= '0' && e <= '9' => {
                            writeln!(stdout, "{}\r", e).unwrap();
                            (print_threshold, pause_threshold) = set(
                                e.to_digit(10).map(|digit| digit as u8).unwrap_or(0),
                                pause_threshold,
                                true,
                            );
                        }
                        Key::Char(e) if e == '!' => {
                            writeln!(stdout, "Shift-1\r").unwrap();
                            (print_threshold, pause_threshold) = set(print_threshold, 1, false);
                        }
                        Key::Char(e) if e == '@' => {
                            writeln!(stdout, "Shift-2\r").unwrap();
                            (print_threshold, pause_threshold) = set(print_threshold, 2, false);
                        }
                        Key::Char(e) if e == '#' => {
                            writeln!(stdout, "Shift-3\r").unwrap();
                            (print_threshold, pause_threshold) = set(print_threshold, 3, false);
                        }
                        Key::Char(e) if e == '$' => {
                            writeln!(stdout, "Shift-4\r").unwrap();
                            (print_threshold, pause_threshold) = set(print_threshold, 4, false);
                        }
                        Key::Char(e) if e == '%' => {
                            writeln!(stdout, "Shift-5\r").unwrap();
                            (print_threshold, pause_threshold) = set(print_threshold, 5, false);
                        }
                        Key::Char(e) if e == '^' => {
                            writeln!(stdout, "Shift-6\r").unwrap();
                            (print_threshold, pause_threshold) = set(print_threshold, 6, false);
                        }
                        Key::Char(e) if e == '&' => {
                            writeln!(stdout, "Shift-7\r").unwrap();
                            (print_threshold, pause_threshold) = set(print_threshold, 7, false);
                        }
                        Key::Char(e) if e == '*' => {
                            writeln!(stdout, "Shift-8\r").unwrap();
                            (print_threshold, pause_threshold) = set(print_threshold, 8, false);
                        }
                        Key::Char(e) if e == '(' => {
                            writeln!(stdout, "Shift-9\r").unwrap();
                            (print_threshold, pause_threshold) = set(print_threshold, 9, false);
                        }
                        Key::Char(e) => {
                            writeln!(stdout, "{} -- Unexpected input!\r", e).unwrap();
                        }
                        _ => {
                            writeln!(stdout, "  -- Unexpected input!\r").unwrap();
                        }
                    }
                } else {
                    writeln!(stdout, "  -- Unexpected input.\r").unwrap();
                }
                show_control_panel(print_threshold, pause_threshold, &mut stdout);
            }
        }
    }
}
