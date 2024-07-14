use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serialport::SerialPort;
use std::{
    error::Error,
    io::{self, BufRead, BufReader, Write},
    sync::mpsc::{self, TryRecvError},
    thread,
    time::Duration,
};

const BAUD_RATE: u32 = 38400;

const DEFAULT_STEP_SIZE: f32 = 0.2;
const DEFAULT_AREA: AreaConf = AreaConf {
    // center: (-0.8, 5.5, 58.25), // Lower slot
    center: (-0.8, 5.5, 23.0), // Upper slot
    horiz_range: 12,
    vert_range: 10,
};

struct AreaConf {
    pub center: (f64, f64, f64),
    pub horiz_range: i32,
    pub vert_range: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScanConf {
    center: (f64, f64, f64),
    horiz_range: i32,
    vert_range: i32,
    step_size: f64,
}

impl ScanConf {
    pub fn new(area: AreaConf, step_size: f32) -> Self {
        Self {
            center: area.center,
            horiz_range: area.horiz_range,
            vert_range: area.vert_range,
            step_size: step_size as f64,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "tag")]
enum Msg {
    CurrentPos { x: i32, y: i32 },
    ScanStep { x: i32, y: i32 },
    CurrentConf { conf: ScanConf },
    ScanStarted, // Ack:
}

#[derive(Debug)]
enum Command {
    SetPos(i32, i32),
    StartScan,
    StopScan,
    SetConf(f32),
}

pub struct Callbacks {
    pub scan_start: fn() -> (),
    pub scan_step: fn(step_size: f32, x: i32, y: i32) -> (),
}

pub struct MotorsClient {
    receiver: mpsc::Receiver<Msg>,
    sender: mpsc::Sender<Command>,
    last_pos: (i32, i32),
    callbacks: Callbacks,
    pub step_size: f32,
}

impl MotorsClient {
    pub fn new(port_name: &str, callbacks: Callbacks) -> Result<Self, io::Error> {
        let (msg_writer, msg_receiver) = mpsc::channel();
        let (cmd_writer, cmd_receiver) = mpsc::channel();

        let timeout = Duration::from_millis(100);
        let mut port = serialport::new(port_name.to_string(), BAUD_RATE)
            .timeout(timeout)
            .open()?;

        thread::spawn(move || loop {
            // info!("Requesting motor position");

            // if request_pos(&mut port).is_err() {
            //     error!("Could not request motor position!");
            // } else {
            //     // info!("Requested position");
            // };

            let mut reader = BufReader::new(&mut port);
            let mut response = String::new();

            match reader.read_line(&mut response) {
                Ok(_) => {
                    let parsed_msg: Result<Msg, serde_json::Error> =
                        serde_json::from_str(response.as_str());
                    match parsed_msg {
                        Ok(msg) => msg_writer.send(msg).unwrap(),
                        Err(err) => error!("Received invalid message. Error: {}", err),
                    }
                }

                Err(_) => {
                    // TODO: Check specifically for timeout error
                    // Timeout is okay
                    // error!("Read failed"),
                    ()
                }
            }

            if let Ok(cmd) = cmd_receiver.try_recv() {
                match cmd {
                    Command::SetPos(x, y) => {
                        info!("Setting position");
                        let msg = json!({
                            "tag": "set_pos",
                            "x": x,
                            "y": y
                        })
                        .to_string();

                        // TODO: Remove unwrap //
                        port.write_all(msg.as_bytes()).unwrap();
                        port.write("\n".as_bytes()).unwrap();
                    }
                    Command::StartScan => {
                        info!("Starting scan");
                        let msg = json!({
                            "tag": "start_scan",
                        })
                        .to_string();

                        // TODO: Remove unwrap //
                        port.write_all(msg.as_bytes()).unwrap();
                        port.write("\n".as_bytes()).unwrap();
                    }

                    Command::StopScan => {
                        info!("Stopping scan");
                        let msg = json!({
                            "tag": "stop_scan",
                        })
                        .to_string();

                        // TODO: Remove unwrap //
                        port.write_all(msg.as_bytes()).unwrap();
                        port.write("\n".as_bytes()).unwrap();
                    }

                    Command::SetConf(step_size) => {
                        let conf = ScanConf::new(DEFAULT_AREA, step_size);
                        info!("Sending configuration");
                        info!("Conf: {:?}", conf);
                        let msg = json!({
                            "tag": "set_conf",
                            "scan_conf": conf
                        })
                        .to_string();

                        // TODO: Remove unwrap //
                        port.write_all(msg.as_bytes()).unwrap();
                        port.write("\n".as_bytes()).unwrap();
                    }
                }
            }

            thread::sleep(Duration::from_millis(1));
        });

        Ok(Self {
            last_pos: (0, 0),
            receiver: msg_receiver,
            sender: cmd_writer,
            callbacks,
            step_size: DEFAULT_STEP_SIZE,
        })
    }

    pub fn get_last_pos(&self) -> (i32, i32) {
        return self.last_pos;
    }

    pub fn set_pos(&self, x: i32, y: i32) {
        // TODO: Remove unwrap //
        self.sender.send(Command::SetPos(x, y)).unwrap();
    }

    pub fn start_scan(&self) {
        // TODO: Remove unwrap //
        self.sender.send(Command::StartScan).unwrap();
    }

    pub fn stop_scan(&self) {
        // TODO: Remove unwrap //
        self.sender.send(Command::StopScan).unwrap();
    }

    pub fn set_conf(&self) {
        // TODO: Remove unwrap //
        self.sender.send(Command::SetConf(self.step_size)).unwrap();
    }

    pub fn update<F>(&mut self, on_new_step_size: F)
    where
        F: FnOnce(f32) -> (),
    {
        match self.receiver.try_recv() {
            Ok(Msg::CurrentPos { x, y }) => {
                self.last_pos = (x, y);
            }

            Ok(Msg::ScanStep { x, y }) => {
                self.last_pos = (x, y);
                (self.callbacks.scan_step)(self.step_size, x, y);
            }

            Ok(Msg::CurrentConf { conf }) => {
                self.step_size = conf.step_size as f32;
                info!("New step size: {}", self.step_size);
                (on_new_step_size)(self.step_size);
            }

            Ok(Msg::ScanStarted) => (self.callbacks.scan_start)(),
            Err(TryRecvError::Empty) => (),
            Err(err) => error!("{}", err),
        }
    }

    pub fn adjust_step(&self, amount: f32) {
        let mut step_size = self.step_size + amount;

        if step_size < 0.1 {
            step_size = 0.1;
        }
        if step_size > 1. {
            step_size = 1.;
        }

        self.sender.send(Command::SetConf(step_size)).unwrap();
    }

    pub fn get_limits(&self) -> (i32, i32) {
        return (
            (DEFAULT_AREA.horiz_range as f32 / self.step_size).floor() as i32,
            (DEFAULT_AREA.vert_range as f32 / self.step_size).floor() as i32,
        );
    }
}

fn _request_pos(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    let get_pos_msg = json!({
        "tag": "get_pos",
    });

    port.write_all(get_pos_msg.to_string().as_bytes())?;
    port.write("\n".as_bytes())?;
    Ok(())
}
