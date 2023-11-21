use anyhow::{Ok, Result};
use crossterm::{
    event::{self, Event, KeyCode, MouseButton},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::io::{self, stdout};



///Button
struct Button {
    button_text: String,
    is_selected: bool,
    action: Box<dyn Fn()>,   
}

///Tracks cursor state
struct Cursor {
    ///Index of selected area
    selected: usize,
}

///Implemenatation for Button
impl Button {
    fn new(button_text: String, action: Box<dyn Fn()>) -> Self {
        Button {
            button_text,
            is_selected: false,
            action
        }
    }
}

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
fn draw_ui(frame: &mut Frame/*, cursor_state: &Cursor*/) {
    //Create a main layout
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(6),
            Constraint::Percentage(75),
            Constraint::Min(1),
        ])
        .split(frame.size());

    //Title bar
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("ByteCrypt").cyan(),
        main_layout[0],
    );

    //Primary Section
    let interaction_layout= Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(main_layout[1]);

    //Menu layout
    let menu_layout= Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(interaction_layout[0]);

    //Create inner layout and place it in the center of main_layout
    let sub_menu_left= Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Min(3)])
        .split(menu_layout[0]);




    //<--------------------------------------Fix this section
    /*let left_buttons = ["Menu Option 1", "Menu Option 2"];
    
    for(i, &button_text) in left_buttons.iter().enumerate() {
        let mut paragraph = Paragraph::new(button_text)
            .alignment(Alignment::Center)
            .white()
            .block(Block::default()
            .borders(Borders::ALL)
            .magenta());

        if cursor_state.selected == i {
            paragraph = paragraph.style(Style::default()
                .add_modifier(Modifier::REVERSED));
        }
        frame.render_widget(paragraph, sub_menu_left[i]);
    }*/
    //<----------------------------------------

    //Create the two left buttons
    frame.render_widget( //Button 1
        Paragraph::new("Menu Option 1")
            .alignment(Alignment::Center)
            .white()
            .block(Block::new()
            .borders(Borders::ALL)
            .magenta()),
        sub_menu_left[0],
    );

    frame.render_widget( //Button 2
        Paragraph::new("Menu Option 2")
            .alignment(Alignment::Center)
            .white()
            .block(Block::new()
            .borders(Borders::ALL)
            .magenta()),
        sub_menu_left[1],
    );

    //Create inner layout and place it in the center of main_layout
    let sub_menu_right= Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Min(3)])
        .split(menu_layout[1]);

    //Create the two buttons
    frame.render_widget( //Button 2
        Paragraph::new("Menu Option 3")
            .alignment(Alignment::Center)
            .white()
            .block(Block::new()
            .borders(Borders::ALL)
            .magenta()),
        sub_menu_right[0],
    );

    frame.render_widget( //Button 2
        Paragraph::new("Menu Option 4")
            .alignment(Alignment::Center)
            .white()
            .block(Block::new()
            .borders(Borders::ALL)
            .magenta()),
        sub_menu_right[1],
    );

    //Information Display
    frame.render_widget( //Button 2
        Block::default()
            .borders(Borders::ALL)
            .title(" Information: ")
            .magenta(),
        interaction_layout[1],
    );

    //Directory Layout
    let directory_layout= Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[2]);
    
    //Right sub-widget for inner_layout
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title("Left Directory")
            .magenta(),
        directory_layout[0],
    );

    //Right sub-widget for inner_layout
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title("Left Directory")
            .magenta(),
        directory_layout[1],
    );

    //Add the status bar at the bottom of the main_layout
    frame.render_widget(
        Block::new()
            .borders(Borders::TOP)
            .title("Status Bar")
            .cyan(),
        main_layout[3],
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
