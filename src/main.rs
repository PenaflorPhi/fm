use std::fs::read_dir;
use std::path::Path;

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{Frame, Terminal};

type Tui = Terminal<CrosstermBackend<std::io::Stdout>>;

fn main() -> std::io::Result<()> {
    let mut terminal = enter_tui()?;
    let entries = list_dir(Path::new("."))?;

    let result = run(&mut terminal, &entries);

    leave_tui()?;
    result
}

fn list_dir(path: &Path) -> std::io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in read_dir(path)? {
        let entry = entry?;
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    Ok(names)
}

fn enter_tui() -> std::io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn leave_tui() -> std::io::Result<()> {
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()
}

fn run(terminal: &mut Tui, entries: &[String]) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, entries))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn draw(frame: &mut Frame, entries: &[String]) {
    let block = Block::bordered().title("fm");
    let listing = Paragraph::new(entries.join("\n")).block(block);
    frame.render_widget(listing, frame.area());
}
