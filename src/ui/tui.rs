use anyhow::{Ok, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::io::{self, stdout};

///Loads the TUI
pub fn load_tui() -> anyhow::Result<()> {
    //Set up the interface
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut should_quit = false;
    while !should_quit {
        //Draw terminal
        terminal.draw(draw_ui)?;
        should_quit = event_handler()?;
    }

    //Close out of the interface
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

///Create the UI
fn draw_ui(frame: &mut Frame) {
    //Create a main layout
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.size());

    //Place the Title bar in main layout
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("ByteCrypt"),
        main_layout[0],
    );

    //Create inner layout and place it in the center of main_layout
    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[1]);

    //Create inner sub-widgets
    //Left sub-widget for inner_layout
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("Left"),
        inner_layout[0],
    );
    //Right sub-widget for inner_layout
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("Right"),
        inner_layout[1],
    );

    //Add the status bar at the bottom of the main_layout
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Status Bar"),
        main_layout[2],
    );
}

///Handles input events for the TUI
fn event_handler() -> anyhow::Result<bool> {
    //16ms ~60fps
    if event::poll(std::time::Duration::from_millis(16))? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    return Ok(false);
}
