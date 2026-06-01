use ratatui::{
  crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEvent,
    KeyEventKind,
  },
  prelude::*,
  widgets::{
    Block,
    Borders,
    Tabs,
  },
};
use std::{
  io,
  time::Duration,
};
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  info,
  instrument,
};
use tui_logger::{
  TuiLoggerLevelOutput,
  TuiLoggerSmartWidget,
};

#[instrument]
pub async fn tui_main(
  port: u16,
  host: Option<String>,
  server_cancellation_token: CancellationToken,
) -> Result<(), io::Error> {
  let mut app = CloverTUI::new(server_cancellation_token);
  ratatui::run(move |terminal| app.run(terminal));
  Ok(())
}

type LazyResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct CloverTUI {
  server_cancellation_token: CancellationToken,
}

impl CloverTUI {
  fn new(server_cancellation_token: CancellationToken) -> Self {
    Self {
      server_cancellation_token,
    }
  }

  fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> LazyResult<()> {
    while !self.server_cancellation_token.is_cancelled() {
      terminal.draw(|frame| self.render(frame))?;
      self.handle_events()?;
    }
    Ok(())
  }

  fn render(&mut self, frame: &mut ratatui::Frame) {
    let area = frame.area();
    let buf = frame.buffer_mut();

    let [tabs_area, smart_area, help_area] = Layout::vertical([
      Constraint::Length(3),
      Constraint::Fill(50),
      Constraint::Length(3),
    ])
    .areas(area);

    TuiLoggerSmartWidget::default()
      .style_error(Style::default().fg(Color::Red))
      .style_debug(Style::default().fg(Color::Green))
      .style_warn(Style::default().fg(Color::Yellow))
      .style_trace(Style::default().fg(Color::Magenta))
      .style_info(Style::default().fg(Color::Cyan))
      .output_separator(':')
      .output_timestamp(Some("%H:%M:%S".to_string()))
      .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
      .output_target(true)
      .output_file(true)
      .output_line(true)
      .render(smart_area, buf);

    Text::from(vec!["Q: Quit".into()])
      .style(Color::Gray)
      .centered()
      .render(help_area, buf);
  }

  fn handle_events(&mut self) -> LazyResult<()> {
    if event::poll(Duration::from_millis(30))? {
      match event::read()? {
        // it's important to check that the event is a key press event as
        // crossterm also emits key release and repeat events on Windows.
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
          self.handle_key_event(key_event)
        }
        _ => {}
      };
    }
    Ok(())
  }

  fn handle_key_event(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('q') => self.server_cancellation_token.cancel(),
      _ => {}
    }
  }
}
