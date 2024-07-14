mod common;
use colored::Colorize;
use common::protocol::Message;
use common::sniffer::monitor;
use std::sync::mpsc;

fn main() {
    if let Some(ports) = get_ports() {
        sniff(ports);
        loop {}
    }

    println!("Exiting.");
}

fn show_message(caption: &str, bytes: &[u8; 6]) {
    if let Some(msg) = Message::from_bytes(&bytes) {
        println!("{}:\t{:?}", caption, msg);
    } else {
        println!("Unhandled message: {:02X?}", bytes);
    }
}

pub fn sniff((soft_port_name, leed_port_name): (String, String)) {
    let (soft_in, soft_out) = mpsc::channel();
    let (leed_in, leed_out) = mpsc::channel();

    println!("{}", "Started channels");

    monitor(soft_port_name, leed_in, soft_out, |buf| {
        show_message("Soft".red().to_string().as_str(), buf);
    });

    monitor(leed_port_name, soft_in, leed_out, |buf| {
        show_message("LEED".green().to_string().as_str(), buf);
    });
}

pub fn get_ports() -> Option<(String, String)> {
    Some(("/dev/ttyUSB1".to_string(), "/dev/ttyUSB0".to_string()))
    // let ports = serialport::available_ports().expect("Cannot enumerate available ports.");
    // let selected_port = ports.first();

    // if selected_port.is_none() {
    //     println!("No ports available.");
    // }

    // Some(selected_port?.port_name.clone())
}
