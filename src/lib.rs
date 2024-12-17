use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

static THRESHOLD: OnceLock<Mutex<(u8, u8)>> = OnceLock::new();
static SIGNAL_PATH: OnceLock<PathBuf> = OnceLock::new();

// A wrapper around any `Write` object that replaces `\n` with `\r\n`
struct LineEndingFix<W: Write> {
    inner: W,
}

impl<W: Write> LineEndingFix<W> {
    fn new(writer: W) -> Self {
        LineEndingFix { inner: writer }
    }
}

impl<W: Write> Write for LineEndingFix<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Replace all "\n" with "\r\n" safely
        let mut cursor = 0;
        while cursor < buf.len() {
            if buf[cursor] == b'\n' {
                self.inner.write_all(b"\r\n")?;
            } else {
                self.inner.write_all(&[buf[cursor]])?;
            }
            cursor += 1;
        }
        Ok(buf.len()) // Indicate that all bytes were "written"
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

pub fn on() {
    let mut data = THRESHOLD.get_or_init(|| Mutex::new((0, 0))).lock().unwrap();
    *data = (0, 0);
}

pub fn off() {
    let mut data = THRESHOLD.get_or_init(|| Mutex::new((0, 0))).lock().unwrap();
    *data = (10, 10);
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
    let path = SIGNAL_PATH.get_or_init(|| {
        let pid = std::process::id();
        PathBuf::from(format!("dbg_step_{}", pid))
    });
    if path.is_file() {
        let _ = std::fs::remove_file(path);
        println!(
            "{{*}}: On because '{}' was found in the current working directory.",
            path.display()
        );
        on();
    }
    let level = level.min(9);
    let (mut print_threshold, mut pause_threshold) = get();
    if level >= print_threshold {
        let mut stdout = LineEndingFix::new(stdout().into_raw_mode().unwrap());
        write!(stdout, "{{{}}}: ", level).unwrap();
        for _ in level..9 {
            write!(stdout, "  ").unwrap();
        }
        writeln!(stdout, "{}", msg().as_ref()).unwrap();
        stdout.flush().unwrap();
        if level < pause_threshold {
            return;
        }
        let show_control_panel = |print_threshold, pause_threshold, stdout: &mut dyn Write| {
            write!(
                    stdout,
                    "\r{{{},{},{}}}: Keys [0-9] set limit, \u{21E7}[0-9] set pause limit, 'q' quits, ' ' continues ", print_threshold, pause_threshold, path.display())
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
