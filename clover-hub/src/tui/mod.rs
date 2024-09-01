use std::{io, thread, time::Duration};
use log::info;
use tui::{
  backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Style}, text::{Span, Spans}, widgets::{Block, Borders, Paragraph}, Terminal
};
use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub async fn tui_main(port: u16, host: Option<String>) -> Result<(), io::Error> {
  info!("Starting TUI...");
  
  // setup terminal
  enable_raw_mode().err();
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture).err();
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  terminal.draw(|f| {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .margin(1)
      .constraints(
        [
          Constraint::Percentage(100),
        ].as_ref()
      )
      .split(f.size());
    let title = Paragraph::new(vec![
      Spans::from(Span::styled(format!("Connecting to {}:{}...", host.unwrap(), port), Style::default().fg(Color::Yellow)))
    ])
      .block(Block::default().title("Clover TUI").borders(Borders::ALL));
    f.render_widget(title, chunks[0]);
  })?;

  thread::sleep(Duration::from_millis(5000));

  // restore terminal
  disable_raw_mode().err();
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  ).err();
  terminal.show_cursor().err();

  Ok(())
}
