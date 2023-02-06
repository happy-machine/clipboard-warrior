use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};
use clipboard::ClipboardProvider; // read and write clipboard
use clipboard::ClipboardContext;
use itertools::Itertools;
use std::fs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};

const DB_PATH: &str = "./clipboarddb.json";

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Serialize, Deserialize, Clone)]
struct Command {
    command: String,
    menu: String
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let MenuItem:String = String::new();
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let command_list = read_db().expect("can fetch command list");
    let mut final_list = vec![];
    let mut menu_options = vec!["Home", "Copy", "Paste", "Delete", "Quit"];
    command_list
        .iter()
        .map(|command| {
            final_list.push(&command.menu[..]);
            command;
        })
        .unique()
        .collect::<Vec<_>>();
    let mut final_list: Vec<_> = final_list.into_iter().unique().collect::<Vec<_>>();
    menu_options
    .iter()
    .map(|command| {
        final_list.push(&command);
        command;
    })
    .collect::<Vec<_>>();
    // let mut active_menu_item:usize = final_list.len() - 5;
    let mut active_menu_item:usize = 0;
    let mut command_list_state = ListState::default();
    command_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let mut index:usize = 0;
            let menu = final_list
                .iter()
                .map(|t| {
                    index +=1;
                    if index < final_list.len() - 4 {
                        let (first, rest) = t.split_at(1);
                        Spans::from(vec![
                            Span::styled(
                                first,
                                Style::default()
                                    .fg(Color::White)
                            ),
                            Span::styled(rest, Style::default().fg(Color::White)),
                        ])
                    } else {
                         let (first, rest) = t.split_at(1);
                        Spans::from(vec![
                            Span::styled(
                                first,
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::UNDERLINED),
                            ),
                            Span::styled(rest, Style::default().fg(Color::White)),
                        ])
                    }
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item)
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().bg(Color::Blue))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            let command_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [Constraint::Percentage(100)].as_ref(),
                )
                .split(chunks[1]);
            if active_menu_item == final_list.len()-5 {
                rect.render_widget(render_home(), chunks[1]);
            } else {
                let (left) = render_commands(&command_list_state, final_list[active_menu_item]);
                rect.render_stateful_widget(left, command_chunk[0], &mut command_list_state);
            }
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    print! ("\x1B[2J\x1B[1;1H");
                    break;
                }
                KeyCode::Char('p') => {
                    add_command_to_db(final_list[active_menu_item].to_string());
                }
                KeyCode::Char('c') => {
                    copy_selected_to_clipboard(&command_list_state, final_list[active_menu_item]);
                }
                KeyCode::Left => {
                    command_list_state.select(Some(0)); 
                    active_menu_item = menu_scroll(",", &final_list, active_menu_item);
                }
                KeyCode::Right => {
                    command_list_state.select(Some(0));
                    active_menu_item = menu_scroll(".", &final_list, active_menu_item);
                }
                KeyCode::Char('d') => {
                    remove_command_at_index(&mut command_list_state, final_list[active_menu_item]).expect("can remove command");
                }
                KeyCode::Char('h') => {
                   active_menu_item = final_list.len() - 5;
                },
                KeyCode::Down => {
                    if let Some(selected) = command_list_state.selected() {
                        let amount_commands = read_db().expect("can fetch command list").iter().filter(|&x| &x.menu == final_list[active_menu_item]).collect::<Vec<_>>().len();
                        if selected >= amount_commands - 1 {
                            command_list_state.select(Some(0));
                        } else {
                            command_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = command_list_state.selected() {
                        let amount_commands = read_db().expect("can fetch command list").iter().filter(|&x| &x.menu == final_list[active_menu_item]).collect::<Vec<_>>().len();
                        if selected > 0 {
                            command_list_state.select(Some(selected - 1));
                        } else {
                            command_list_state.select(Some(amount_commands - 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn menu_scroll<'a>(direction:&str, final_list:&Vec<&str>, mut active_menu_item:usize) -> usize {
    if active_menu_item == final_list.len()-5 {
        active_menu_item = 0
    } else if direction == "." && active_menu_item < final_list.len() - 6 {
        active_menu_item += 1;
    } else if direction == "," && active_menu_item >= 1 {
        active_menu_item -= 1;
    }
    active_menu_item
}

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "______ __ __       __                        __                               __            ",
            Style::default().fg(Color::LightBlue),
        )]),
                Spans::from(vec![Span::styled(
            "|      |  |__.-----|  |--.-----.---.-.----.--|  |    .--.--.--.---.-.----.----|__.-----.----.",
            Style::default().fg(Color::LightBlue),
        )]),
                        Spans::from(vec![Span::styled(
            "|   ---|  |  |  _  |  _  |  _  |  _  |   _|  _  |    |  |  |  |  _  |   _|   _|  |  _  |   _|",
            Style::default().fg(Color::LightBlue),
        )]),
                        Spans::from(vec![Span::styled(
            "|______|__|__|   __|_____|_____|___._|__| |_____|    |________|___._|__| |__| |__|_____|__|  ",
            Style::default().fg(Color::LightBlue),
        )]),
                                Spans::from(vec![Span::styled(
            "             |__|                                                                            ",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Clipboard warrior makes it easy for you to save, retrieve and backup terminal commands.")]),
        Spans::from(vec![Span::raw("Menu options and commands are created from the contents of the local clipboarddb.json file.")]),
        Spans::from(vec![Span::raw("Your favourite commands are pasted from your clipboard and saved to this file.")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Use the arrow keys to navigate between menus and commands. Copy selected to clipboard with 'c'.")]),
        Spans::from(vec![Span::raw("Paste clipboard to current menu with 'p' and delete commands with 'd'.")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("By Happy Machine (https://github.com/happy-machine)")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}

fn copy_selected_to_clipboard<'a>(command_list_state: &ListState, menu: &str) -> () {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    let command_list = read_db().expect("can fetch command list");
    let items: Vec<_> = command_list
        .iter()
        .filter(|&x| &x.menu == menu)
        .map(|command| {
            ListItem::new(Spans::from(vec![Span::styled(
                command.command.clone(),
                Style::default(),
            )]))
        })
        .collect();
    let filtered = command_list
        .iter()
        .filter(|&x| &x.menu == menu)
        .collect::<Vec<_>>();

    let selected = filtered
        .get(
            command_list_state
                .selected()
                .expect("there is always a selected command"),
        )
        .expect("exists")
        .clone();
    let command = selected.command.clone();
    ctx.set_contents(command).unwrap();
}

fn render_commands<'a>(command_list_state: &ListState, menu: &str) -> List<'a> {
    let commands = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Commands")
        .border_type(BorderType::Plain);

    let command_list = read_db().expect("can fetch command list");
    let items: Vec<_> = command_list
        .iter()
        .filter(|&x| &x.menu == menu)
        .map(|command| {
            ListItem::new(Spans::from(vec![Span::styled(
                command.command.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_command = command_list
        .get(
            command_list_state
                .selected()
                .expect("there is always a selected command"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(commands).highlight_style(
        Style::default()
            .bg(Color::Gray)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    enable_raw_mode().expect("can run in raw mode");
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    list
}

fn read_db() -> Result<Vec<Command>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Command> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

fn add_command_to_db(menu: String) -> Result<Vec<Command>, Error> {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    let s: String =  ctx.get_contents().unwrap();
    let mut rng = rand::thread_rng();
    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Command> = serde_json::from_str(&db_content)?;
    let clipboard_command = Command {
        menu: menu,
        command: s
    };

    parsed.push(clipboard_command);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}

fn remove_command_at_index(command_list_state: &mut ListState, menu: &str) -> Result<(), Error> {
    let command_list = read_db().expect("can fetch command list");
    let filtered = command_list
        .iter()
        .filter(|&x| &x.menu == menu)
        .collect::<Vec<_>>();
        let selected = filtered
        .get(
            command_list_state
                .selected()
                .expect("there is always a selected command"),
        )
        .expect("exists")
        .clone();
    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Command> = serde_json::from_str(&db_content)?;
    let mut index=0;
    let mut final_index=0;
    parsed
    .iter()
    .map(|command| {
            if selected.command == &command.command[..] {
                final_index = index;
            } else {
                index += 1;
            }
        })
    .collect::<Vec<_>>();
    parsed.remove(final_index);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
    if let Some(selected) = command_list_state.selected() {
        if selected > 0 {
            command_list_state.select(Some(selected - 1));
        } else {
            command_list_state.select(Some(0));
        }
    }
    Ok(())
}