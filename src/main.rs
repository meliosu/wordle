use std::collections::HashSet;
use std::io::Write;
use std::time::Duration;

use crossterm::style::{Color, ContentStyle, PrintStyledContent, StyledContent, Stylize};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::Print,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use lazy_static::lazy_static;
use rand::seq::SliceRandom;

fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, Show);
        println!("thread {info}");
    }));

    let mut stdout = std::io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let mut wordle = Wordle::new();

    let won = loop {
        render_wordle(&wordle)?;

        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => break false,

            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) if c.is_ascii_alphabetic() => {
                wordle.input(c);
            }

            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => {
                wordle.erase();
            }

            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                wordle.guess();
            }

            _ => {}
        }

        if let Some(won) = wordle.won() {
            std::thread::sleep(Duration::from_secs(1));
            break won;
        }
    };

    terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, Show)?;

    if won {
        println!("you have won!");
    } else {
        println!("The answer was {}.", wordle.answer.to_ascii_uppercase());
        println!("Maybe try again later...");
    }

    Ok(())
}

fn render_wordle(wordle: &Wordle) -> std::io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let (width, height) = (21, 13);
    let (x, y) = ((cols - width) / 2, (rows - height) / 2);

    let top = "╔═══╦═══╦═══╦═══╦═══╗";
    let mid = "║   ║   ║   ║   ║   ║";
    let int = "╠═══╬═══╬═══╬═══╬═══╣";
    let bot = "╚═══╩═══╩═══╩═══╩═══╝";

    let mut stdout = std::io::stdout();

    let rows = {
        let mut rows: Vec<&str> = std::iter::repeat([mid, int]).take(6).flatten().collect();
        rows.pop();
        rows.push(bot);
        rows.insert(0, top);

        rows
    };

    // print grid
    for (y, row) in (y..).zip(rows) {
        queue!(stdout, MoveTo(x, y), Print(row))?;
    }

    // print previous guesses
    for (y, guess) in (y + 1..).step_by(2).zip(&wordle.guesses) {
        for (idx, (x, c)) in (x + 2..).step_by(4).zip(guess.chars()).enumerate() {
            let color = if guess[idx..idx + 1] == wordle.answer[idx..idx + 1] {
                Color::Green
            } else if wordle.answer.contains(c) {
                Color::Yellow
            } else {
                Color::Grey
            };

            let style = ContentStyle::new().with(color);

            queue!(
                stdout,
                MoveTo(x, y),
                PrintStyledContent(StyledContent::new(style, c.to_ascii_uppercase()))
            )?;
        }
    }

    // print current guess
    for (x, c) in (x + 2..).step_by(4).zip(wordle.curr.chars()) {
        let y = y + 2 * wordle.guesses.len() as u16 + 1;
        queue!(stdout, MoveTo(x, y), Print(c.to_ascii_uppercase()))?;
    }

    stdout.flush()?;
    Ok(())
}

struct Wordle {
    answer: String,
    curr: String,
    guesses: Vec<String>,
}

lazy_static! {
    static ref GUESSES: HashSet<&'static str> = include_str!("../guesses").lines().collect();
    static ref ANSWERS: Vec<&'static str> = include_str!("../answers").lines().collect();
}

impl Wordle {
    fn new() -> Self {
        let answer = ANSWERS.choose(&mut rand::thread_rng()).unwrap();

        Self {
            answer: answer.to_string(),
            curr: String::new(),
            guesses: Vec::new(),
        }
    }

    fn input(&mut self, c: char) {
        if self.curr.len() < 5 {
            self.curr.push(c.to_ascii_lowercase());
        }
    }

    fn erase(&mut self) {
        self.curr.pop();
    }

    fn guess(&mut self) {
        if self.curr.len() == 5 && GUESSES.contains(self.curr.as_str()) {
            self.guesses.push(std::mem::take(&mut self.curr));
        }
    }

    fn won(&self) -> Option<bool> {
        if self.guesses.last() == Some(&self.answer) {
            Some(true)
        } else if self.guesses.len() == 6 {
            Some(false)
        } else {
            None
        }
    }
}
