
const BAUD_RATE: u32 = 38400;
use std::time::Duration;

use leed_controller::common::protocol::Message;

fn main() {
    if let Some(port_name) = get_port() {
        sniff(&port_name).unwrap();
    }

    println!("Exiting.");
}

fn get_port() -> Option<String> {
    let ports = serialport::available_ports().expect("Cannot enumerate available ports.");
    let selected_port = ports.first();

    if selected_port.is_none() {
        println!("No ports available.");
    }

    Some(selected_port?.port_name.clone())
}

fn sniff(port_name: &str) -> Result<(), serialport::Error> {
    let mut port = serialport::new(port_name, BAUD_RATE).open()?;
    port.set_timeout(Duration::from_secs(60))?;

    let mut buf: [u8; 6] = [0; 6];
    loop {
        port.read_exact(&mut buf)?;
        port.write_all(&buf)?;
        // println!("Message: {:02X?}", buf);
        if let Some(msg) = Message::from_bytes(&buf) {
            println!("Message: {:?}", msg);
        } else {
            println!("Unhandled message: {:02X?}", buf);
        }
    }
}
