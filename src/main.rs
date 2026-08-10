use std::collections::HashMap;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, List, ListState};
use ratatui::{Frame, Terminal};

type Tui = Terminal<CrosstermBackend<std::io::Stdout>>;

fn main() -> std::io::Result<()> {
    let start_dir = std::env::current_dir()?;

    let mut terminal = enter_tui()?;
    let result = run(&mut terminal, start_dir);
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

fn list_parent(current_dir: &Path) -> std::io::Result<Vec<String>> {
    match current_dir.parent() {
        Some(parent) => list_dir(parent),
        None => Ok(Vec::new()),
    }
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

fn run(terminal: &mut Tui, start_dir: PathBuf) -> std::io::Result<()> {
    let mut current_dir = start_dir;
    let mut entries = list_dir(&current_dir)?;
    let mut parent_entries = list_parent(&current_dir)?;

    // Por ahora almacenemos un HashMap con el path y el último indice,
    // de este modo al subir o bajar en el árbol preservamos la posición
    // del cursor al regresar.
    // Eventualmente almacenaremos más información del directorio.
    let mut positions: HashMap<PathBuf, usize> = HashMap::new();

    let mut state = ListState::default();
    state.select(Some(0));

    loop {
        terminal.draw(|frame| draw(frame, &current_dir, &parent_entries, &entries, &mut state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('j') | KeyCode::Down => state.select_next(),
                KeyCode::Char('k') | KeyCode::Up => state.select_previous(),
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                    descend(
                        &mut current_dir,
                        &mut entries,
                        &mut parent_entries,
                        &mut positions,
                        &mut state,
                    )?;
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    ascend(
                        &mut current_dir,
                        &mut entries,
                        &mut parent_entries,
                        &mut positions,
                        &mut state,
                    )?;
                }
                _ => {}
            }
        }
    }
}

fn descend(
    current_dir: &mut PathBuf,
    entries: &mut Vec<String>,
    parent_entries: &mut Vec<String>,
    positions: &mut HashMap<PathBuf, usize>,
    state: &mut ListState,
) -> std::io::Result<()> {
    let index = match state.selected() {
        Some(index) => index,
        None => return Ok(()),
    };
    let name = match entries.get(index) {
        Some(name) => name,
        None => return Ok(()),
    };

    let target = current_dir.join(name);
    if !target.is_dir() {
        return Ok(());
    }

    positions.insert(current_dir.clone(), index);
    *entries = list_dir(&target)?;
    *current_dir = target;
    *parent_entries = list_parent(current_dir)?;

    let restored = positions.get(current_dir.as_path()).copied().unwrap_or(0);
    state.select(Some(restored));
    Ok(())
}

fn ascend(
    current_dir: &mut PathBuf,
    entries: &mut Vec<String>,
    parent_entries: &mut Vec<String>,
    positions: &mut HashMap<PathBuf, usize>,
    state: &mut ListState,
) -> std::io::Result<()> {
    let parent = match current_dir.parent() {
        Some(parent) => parent.to_path_buf(),
        None => return Ok(()),
    };
    positions.insert(current_dir.clone(), state.selected().unwrap_or(0));
    *entries = list_dir(&parent)?;
    *current_dir = parent;
    *parent_entries = list_parent(current_dir)?;

    let restored = positions.get(current_dir.as_path()).copied().unwrap_or(0);
    state.select(Some(restored));

    Ok(())
}

fn draw(
    frame: &mut Frame,
    current_dir: &Path,
    parent_entries: &[String],
    entries: &[String],
    state: &mut ListState,
) {
    let [parent_area, current_area, preview_area] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(50),
    ])
    .areas(frame.area());

    // Parent Pane
    let parent_title = match current_dir.parent() {
        Some(parent) => format!(" {} ", parent.display()),
        None => " / ".to_string(),
    };
    let parent_list = List::new(parent_entries.iter().map(String::as_str))
        .block(Block::bordered().title(parent_title));
    frame.render_widget(parent_list, parent_area);

    // Current Pane
    let current_list = List::new(entries.iter().map(String::as_str))
        .block(Block::bordered().title(format!(" {} ", current_dir.display())))
        .highlight_symbol("> ");
    frame.render_stateful_widget(current_list, current_area, state);

    let preview = Block::bordered().title(" preview ");
    frame.render_widget(preview, preview_area);
}
