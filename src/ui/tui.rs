use anyhow::{Ok, Result};
use crossterm::{
    event::{self, Event, KeyCode, MouseButton},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::io::{self, stdout};


///Tracks cursor state
struct Cursor {
    ///Index of selected area
    selected: usize,
}


///Loads the TUI
pub fn load_tui() -> anyhow::Result<()> {
    //Set up the interface
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut should_quit = false;
    let mut cursor = Cursor { selected: 0 };
    
    while !should_quit {
        //Draw terminal
        terminal.draw(|frame| draw_ui(frame, &cursor))?;
        should_quit = event_handler(&mut cursor)?;
    }

    //Close out of the interface
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

///Create the UI
fn draw_ui(frame: &mut Frame, cursor: &Cursor) {
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

    //Sub menu on the left side of the menu layout
    let sub_menu_left= Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Min(3)])
        .split(menu_layout[0]);

    //Sub menu on the right side of the menu layout
    let sub_menu_right= Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Min(3)])
        .split(menu_layout[1]);


    //Create and implement the buttons
    let button_text  = [
        "Menu Option 1", 
        "Menu Option 2", 
        "Menu Option 3", 
        "Menu Option 4"
    ];
    let sub_menu = [
        sub_menu_left[0], 
        sub_menu_left[1], 
        sub_menu_right[0], 
        sub_menu_right[1]
    ];
    
    for(button, &button_text) in button_text.iter().enumerate() {
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .fg(Color::Magenta);
        
        let inner_style = if cursor.selected == button {
            Style::default().fg(Color::White)
                .bg(Color::Magenta)
        } else {
            Style::default().fg(Color::White)
        };

        let inner_paragraph = Paragraph::new(button_text)
            .alignment(Alignment::Center)
            .style(inner_style);

        frame.render_widget(outer_block, sub_menu[button]);

        let inner_area = {
            let mut area = sub_menu[button];
            area.height = area.height.saturating_sub(2);
            area.width = area.width.saturating_sub(2);
            area.x += 1;
            area.y += 1;
            area
        };

        frame.render_widget(inner_paragraph, inner_area);
    }

    //Information Display
    let button_info = [
        "Menu Option 1 Info",
        "Menu Option 2 Info",
        "Menu Option 3 Info",
        "Menu Option 4 Info",
    ];

    let info_window = Paragraph::new(button_info[cursor.selected])
        .block(Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title(" Information ")
        .title_style(Style::default().fg(Color::Blue)))
        .white()
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(info_window, interaction_layout[1]);

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
            .title("Right Directory")
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
fn event_handler(cursor: &mut Cursor) -> anyhow::Result<bool> {
    //16ms ~60fps
    if event::poll(std::time::Duration::from_millis(16))? {
        if let event::Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if cursor.selected % 2 > 0 {
                        cursor.selected -= 1;
                    }
                },
                KeyCode::Left => {
                    if cursor.selected > 1 {
                        cursor.selected -= 2;
                    }
                }
                KeyCode::Down => {
                    if cursor.selected % 2 == 0 {
                        cursor.selected += 1;
                    }
                }
                KeyCode::Right => {
                    if cursor.selected < 2 {
                        cursor.selected += 2;
                    }
                }
                KeyCode::Enter => {
                    //Key action for enter here
                }
                KeyCode::Char('q') => return Ok(true),
                _ => {}
            }
        }
    }

    return Ok(false);
}
