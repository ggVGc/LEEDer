use common::controller::{Adjustment, Controller};
use common::protocol::Message;
use common::sniffer::monitor;
use leed_controller::application::{setup_camera, App};
use leed_controller::common;
use leed_controller::common::protocol::Tag;
use leed_controller::common::tui_log::{LogWidget, LogWidgetState, TuiLogger};
use log::{error, info, LevelFilter};
use std::collections::VecDeque;
use std::io::{self, stdout};
use std::sync::{mpsc, Arc, Mutex};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List},
};

const LEED_PORT: &str = "/dev/ttyUSB0";

fn main() -> io::Result<()> {
    let mut ui = UIState::new();
    TuiLogger::init(LevelFilter::Info, ui.log_state.clone()).expect("Could not initlize logger.");

    let mut app = App::new(None);
    // TODO: Use for sending commands to leed controller
    let (leed_send, leed_recv) = mpsc::channel();
    let (leed_listener, leed_responses) = mpsc::channel();

    // TODO: Gracefully exit when thread exits
    if !monitor(LEED_PORT, leed_listener, leed_recv) {
        error!("LEED communication init failed!");
    }

    if setup_camera() {
        info!("Camera initialized");
    } else {
        error!("Camera init failed!");
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    while handle_ui_events(&mut app)? {
        app.leed_controller.update(&leed_send);

        handle_leed_messages(&leed_responses, &mut app.leed_controller, &mut ui);
        app.update();
        ui.update();
        terminal.draw(|frame| {
            render_ui(frame, &mut app, &mut ui);
        })?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn handle_ui_events(app: &mut App) -> io::Result<bool> {
    let poll_time = std::time::Duration::from_millis(50);
    let mut should_continue = true;

    let controls = &mut app.leed_controller.controls;
    let control_inputs = [
        ('a', 'z', &mut controls.beam_energy),
        ('s', 'x', &mut controls.wehnheit),
        ('d', 'c', &mut controls.emission),
        ('f', 'v', &mut controls.filament),
        ('g', 'b', &mut controls.screen),
        ('h', 'n', &mut controls.lens1_3),
        ('j', 'm', &mut controls.lens2),
        ('k', ',', &mut controls.suppressor),
    ];

    if event::poll(poll_time)? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => should_continue = false,
                    _ => {
                        for (up, down, control) in control_inputs {
                            if key.code == KeyCode::Char(up) {
                                info!("{}: +", control.name);
                                control.adjust(Adjustment::Up)
                            } else if key.code == KeyCode::Char(down) {
                                info!("{} -", control.name);
                                control.adjust(Adjustment::Down)
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(should_continue)
}

fn buf_to_msg_string(bytes: &[u8; 6]) -> Option<String> {
    if let Some(msg) = Message::from_bytes(bytes) {
        match msg.tag {
            Tag::ADC1 | Tag::ADC2 | Tag::ADC3 => None,
            _ => Some(format!("{:?}", msg)),
        }
    } else {
        Some(format!("Unhandled message: {:02X?}", bytes))
    }
}

fn handle_leed_messages(
    receiver: &mpsc::Receiver<[u8; 6]>,
    controller: &mut Controller,
    ui: &mut UIState,
) {
    while let Ok(buf) = receiver.try_recv() {
        if let Some(msg) = Message::from_bytes(&buf) {
            let mut logs = VecDeque::new();
            controller.update_from_message(msg, &mut logs);
        }

        if let Some(msg) = buf_to_msg_string(&buf) {
            ui.leed_messages.push_front(msg.to_string());
        }
    }
}

fn render_ui(frame: &mut Frame, app: &mut App, state: &mut UIState) {
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2),
        ],
    )
    .split(frame.size());

    let top_horiz = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(main_layout[1]);

    let bottom_horiz = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(main_layout[2]);

    frame.render_widget(
        Block::new().borders(Borders::TOP).title("LEED"),
        main_layout[0],
    );

    let controller_layout = top_horiz[0];

    if let Ok(log_state) = &mut state.log_state.lock() {
        frame.render_widget(
            Block::default().title("Log".green()).borders(Borders::ALL),
            bottom_horiz[0],
        );
        let inset_area = edge_inset(&bottom_horiz[0], 1);
        frame.render_stateful_widget(LogWidget::default(), inset_area, log_state);
    }

    render_messages(
        frame,
        bottom_horiz[1],
        "LEED Messages",
        state.leed_messages.clone(), // TODO: Avoid clone?
    );

    render_controller(frame, controller_layout, &app.leed_controller);
}

fn render_messages<T>(frame: &mut Frame, area: Rect, title: &str, messages: T)
where
    T: IntoIterator<Item = String>,
{
    let list =
        List::new(messages).block(Block::default().title(title.green()).borders(Borders::ALL));

    frame.render_widget(list, area);
}

fn render_controller(frame: &mut Frame, area: Rect, c: &Controller) {
    let title = "Controls";
    let mut controls_content = Vec::from(
        [
            ("[a/z] Beam Energy", &c.controls.beam_energy),
            ("[s/x] Wehnheit", &c.controls.wehnheit),
            ("[d/c] Emission", &c.controls.emission),
            ("[f/v] Filament", &c.controls.filament),
            ("[g/b] Screen", &c.controls.screen),
            ("[h/n] Lens 1/3 Gain", &c.controls.lens1_3),
            ("j/m] Lens 2 Gain", &c.controls.lens2),
            ("[k/,] Suppressor", &c.controls.suppressor),
        ]
        .map(|(title, value)| format!("{}: {}", title, value)),
    );

    controls_content.extend(
        [
            ("Beam current", c.current.beam),
            ("Emission current", c.current.emission),
            ("Filament current", c.current.filament),
        ]
        .map(|(title, value)| format!("{}: {}", title, value)),
    );

    let list = List::new(controls_content)
        .block(Block::default().title(title.red()).borders(Borders::ALL));

    frame.render_widget(list, area);
}
// let chart = BarChart::default()
//     .block(Block::default().title("BarChart").borders(Borders::ALL))
//     .bar_width(3)
//     .bar_gap(1)
//     .group_gap(3)
//     .bar_style(Style::new().yellow().on_red())
//     .value_style(Style::new().red().bold())
//     .label_style(Style::new().white())
//     .data(&[("B0", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
//     .data(BarGroup::default().bars(&[Bar::default().value(10), Bar::default().value(20)]))
//     .max(4);

// frame.render_widget(chart, bottom_layout[0]);

/*
fn render_test_plot(frame: &mut Frame, area: Rect) {
    let datasets = vec![
        // Scatter chart
        Dataset::default()
            .name("data1")
            .marker(symbols::Marker::Bar)
            .graph_type(GraphType::Line)
            .style(Style::default().cyan())
            .data(&[(0.0, 5.0), (1.0, 6.0), (1.5, 6.434)]),
        // Line chart
        Dataset::default()
            .name("data2")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().magenta())
            .data(&[(4.0, 5.0), (5.0, 8.0), (7.66, 13.5)]),
    ];
    // Create the X axis and define its properties
    let x_axis = Axis::default()
        .title("X Axis".red())
        .style(Style::default().white())
        .bounds([0.0, 10.0])
        .labels(vec!["0.0".into(), "5.0".into(), "10.0".into()]);

    // Create the Y axis and define its properties
    let y_axis = Axis::default()
        .title("Y Axis".red())
        .style(Style::default().white())
        .bounds([0.0, 10.0])
        .labels(vec!["0.0".into(), "5.0".into(), "10.0".into()]);

    // Create the chart and link all the parts together
    let chart = Chart::new(datasets)
        .block(Block::default().title("Chart"))
        .x_axis(x_axis)
        .y_axis(y_axis);

    frame.render_widget(chart, area);
}
*/

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
