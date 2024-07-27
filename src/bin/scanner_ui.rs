use leed_controller::common::tui_log::{LogWidget, LogWidgetState, TuiLogger};
use leed_controller::scanner::Scanner;
use log::LevelFilter;
use std::collections::VecDeque;
use std::io::{self, stdout};
use std::sync::{Arc, Mutex};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{
    prelude::*,
    style::Color,
    widgets::{canvas::*, *},
};

const MOTORS_PORT: &str = "/dev/ttyUSB1";

fn main() -> io::Result<()> {
    let mut ui = UIState::new();
    TuiLogger::init(LevelFilter::Info, ui.log_state.clone()).expect("Logger init failed");

    let mut scanner = Scanner::new(MOTORS_PORT).expect("Scanner init failed");

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    while handle_ui_events(&mut scanner)? {
        scanner.update();
        ui.update();
        terminal.draw(|frame| {
            render_ui(frame, &scanner, &ui);
        })?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn handle_ui_events(scanner: &mut Scanner) -> io::Result<bool> {
    let poll_time = std::time::Duration::from_millis(50);

    if event::poll(poll_time)? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                if key.code == KeyCode::Char('q') {
                    return Ok(false);
                } else {
                    match key.code {
                        KeyCode::Char('s') => {
                            scanner.start_scan();
                        }
                        KeyCode::Char('c') => scanner.stop_scan(),
                        KeyCode::Char('g') => {
                            scanner.goto_target_pos();
                        }
                        KeyCode::Up => {
                            scanner.target_pos.y += 1;
                        }
                        KeyCode::Down => {
                            scanner.target_pos.y -= 1;
                        }
                        KeyCode::Left => {
                            scanner.target_pos.x -= 1;
                        }

                        KeyCode::Right => {
                            scanner.target_pos.x += 1;
                        }

                        KeyCode::Char('m') => {
                            scanner.adjust_scan_step(0.1);
                        }

                        KeyCode::Char('n') => {
                            scanner.adjust_scan_step(-0.1);
                        }
                        _ => (),
                    }
                    return Ok(true);
                }
            }
        }
    }
    Ok(true)
}

fn render_ui(frame: &mut Frame, scanner: &Scanner, state: &UIState) {
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2),
        ],
    )
    .split(frame.size());

    let top_horiz =
        Layout::new(Direction::Horizontal, [Constraint::Percentage(100)]).split(main_layout[1]);

    let bottom_horiz =
        Layout::new(Direction::Horizontal, [Constraint::Percentage(100)]).split(main_layout[2]);

    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Scanner"),
        main_layout[0],
    );

    if let Ok(log_state) = &mut state.log_state.lock() {
        frame.render_widget(
            Block::default().title("Log".green()).borders(Borders::ALL),
            bottom_horiz[0],
        );
        let inset_area = edge_inset(&bottom_horiz[0], 1);
        frame.render_stateful_widget(LogWidget::default(), inset_area, log_state);
    }

    let ((scan_x, scan_y), (max_x, max_y)) = scanner.get_scan_pos();
    let scan_display = Canvas::default()
        .block(
            Block::default()
                .title(format!(
                    "[Scan] x: {}, y: {} | [Selector] x: {}, y: {} | Step: {:.2}",
                    scan_x,
                    scan_y,
                    scanner.target_pos.x,
                    scanner.target_pos.y,
                    scanner.get_step_size()
                ))
                .borders(Borders::ALL),
        )
        .x_bounds([0.0, max_x as f64])
        .y_bounds([0.0, max_y as f64])
        .paint(|ctx| {
            ctx.draw(&Rectangle {
                x: scanner.target_pos.x as f64,
                y: scanner.target_pos.y as f64,
                width: 1.,
                height: 1.,
                color: Color::Red,
            });

            ctx.draw(&Rectangle {
                x: scan_x as f64,
                y: scan_y as f64,
                width: 1.,
                height: 1.,
                color: Color::White,
            });
        });

    frame.render_widget(scan_display, top_horiz[0]);
}

struct UIState {
    leed_messages: VecDeque<String>,
    log_state: Arc<Mutex<LogWidgetState>>,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            leed_messages: VecDeque::with_capacity(20),
            log_state: Arc::new(Mutex::new(LogWidgetState::default())),
        }
    }

    fn update(&mut self) {
        self.leed_messages.truncate(1000);
    }
}

fn edge_inset(area: &Rect, margin: u16) -> Rect {
    let mut inset_area = *area;
    inset_area.x += margin;
    inset_area.y += margin;
    inset_area.height -= margin;
    inset_area.width -= margin;

    inset_area
}
