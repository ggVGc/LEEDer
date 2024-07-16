use leed_controller::common::controller::Controller;
use leed_controller::common::protocol::{Message, MessageTag};
use leed_controller::common::sniffer::monitor2;
use std::collections::VecDeque;
use std::io::{self, stdout};
use std::sync::mpsc;
use std::thread;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};

const MAX_CAP: usize = 100;
const DUMMY_REPEATER: bool = true;

struct Counters {
    leed: i32,
    soft: i32,
}

fn main() -> io::Result<()> {
    let mut controller = Controller::new();

    let mut software_messages: VecDeque<String> = VecDeque::with_capacity(20);
    let mut leed_messages: VecDeque<String> = VecDeque::with_capacity(20);
    let mut counters = Counters { soft: 0, leed: 0 };

    let (soft_send, soft_recv) = mpsc::channel();
    let (leed_send, leed_recv) = mpsc::channel();
    let (soft_listen_in, soft_listen_out) = mpsc::channel();
    let (leed_listen_in, leed_listen_out) = mpsc::channel();

    let soft_port = "/dev/ttyUSB0".to_string();
    let leed_port = "/dev/ttyUSB1".to_string();
    monitor2(soft_port, leed_send, soft_listen_in, soft_recv);

    if DUMMY_REPEATER {
        echo_messages(soft_send, leed_listen_in, leed_recv);
    } else {
        monitor2(leed_port, soft_send, leed_listen_in, leed_recv);
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut running = true;

    while running {
        while let Ok(buf) = soft_listen_out.try_recv() {
            if let Some(msg) = buf_to_msg_string(&buf) {
                software_messages.push_front(format!("[{}] {}", counters.soft.to_string(), msg));
            }
            counters.soft += 1;
        }

        while let Ok(buf) = leed_listen_out.try_recv() {
            if let Some(msg) = Message::from_bytes(&buf) {
                controller.update_from_message(msg, &mut leed_messages);
            }

            if let Some(msg) = buf_to_msg_string(&buf) {
                leed_messages.push_front(format!("[{}] {}", counters.leed.to_string(), msg));
            }
            counters.leed += 1;
        }

        software_messages.truncate(MAX_CAP);
        leed_messages.truncate(MAX_CAP);

        terminal.draw(|frame| {
            ui(
                frame,
                &controller,
                software_messages.clone().into(),
                leed_messages.clone().into(),
            );
        })?;
        running = !handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn handle_events() -> io::Result<bool> {
    let poll_time = std::time::Duration::from_millis(50);

    if event::poll(poll_time)? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn ui(
    frame: &mut Frame,
    controller: &Controller,
    software_messages: Vec<String>,
    leed_messages: Vec<String>,
) {
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2),
        ],
    )
    .split(frame.size());

    frame.render_widget(
        Block::new().borders(Borders::TOP).title("LEED sniffer"),
        main_layout[0],
    );

    let controller_layout = main_layout[1];

    let horiz_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(main_layout[2]);

    render_messages(frame, horiz_layout[0], "From: Software", software_messages);
    render_messages(frame, horiz_layout[1], "LEED Controller", leed_messages);

    render_controller(frame, controller_layout, &controller);
}

fn render_messages(frame: &mut Frame, area: Rect, title: &str, messages: Vec<String>) {
    let list =
        List::new(messages).block(Block::default().title(title.green()).borders(Borders::ALL));

    frame.render_widget(list, area);
}

fn render_controller(frame: &mut Frame, area: Rect, c: &Controller) {
    let title = "Controls";
    let mut controls_content = Vec::from(
        [
            ("Beam Energy", &c.controls.beam_energy),
            ("Suppressor", &c.controls.suppressor),
            ("Lens 2 Set", &c.controls.lens2),
            ("Lens 1/3 Set", &c.controls.lens1_3),
            ("Wehnheit", &c.controls.wehnheit),
            ("Emission", &c.controls.emission),
            ("Filament", &c.controls.filament),
            ("Screen", &c.controls.screen),
        ]
        .map(|(title, value)| format!("{}: {}", title, value.to_string())),
    );

    controls_content.extend(
        [
            ("Beam current", c.current.beam),
            ("Emission current", c.current.emission),
            ("Filament current", c.current.filament),
        ]
        .map(|(title, value)| format!("{}: {}", title, value.to_string())),
    );

    let list = List::new(controls_content)
        .block(Block::default().title(title.red()).borders(Borders::ALL));

    frame.render_widget(list, area);
}

fn buf_to_msg_string(bytes: &[u8; 6]) -> Option<String> {
    if let Some(msg) = Message::from_bytes(&bytes) {
        match msg.tag {
            MessageTag::ADC1 => return None,
            MessageTag::ADC2 => return None,
            MessageTag::ADC3 => return None,
            _ => return Some(format!("{:?}", msg)),
        }
    } else {
        return Some(format!("Unhandled message: {:02X?}", bytes));
    }
}

pub fn echo_messages(
    sender: mpsc::Sender<[u8; 6]>,
    sender2: mpsc::Sender<[u8; 6]>,
    receiver: mpsc::Receiver<[u8; 6]>,
) -> std::thread::JoinHandle<()> {
    thread::spawn(move || loop {
        // let mut buf: [u8; 6] = [0; 6];
        // if let Ok(_) = port.read_exact(&mut buf) {
        //     sender.send(buf).expect("Failed storing message.");
        //     sender2.send(buf).expect("Failed storing message.");
        // }

        if let Ok(data) = receiver.try_recv() {
            sender.send(data).expect("Failed storing message.");
            sender2.send(data).expect("Failed storing message.");
        }
    })
}
