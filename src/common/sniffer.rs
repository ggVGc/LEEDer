use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const BAUD_RATE: u32 = 38400;

pub fn monitor(
    port_name: &str,
    senders: Vec<mpsc::Sender<[u8; 6]>>,
    receiver: mpsc::Receiver<[u8; 6]>,
) -> serialport::Result<JoinHandle<()>> {
    let timeout = Duration::from_millis(10);

    let mut port = serialport::new(port_name.to_string(), BAUD_RATE)
        .timeout(timeout)
        .open()?;

    let handle = thread::spawn(move || loop {
        let mut buf: [u8; 6] = [0; 6];
        if port.read_exact(&mut buf).is_ok() {
            for sender in &senders {
                sender.send(buf).expect("Failed storing message.");
            }
        }

        if let Ok(data) = receiver.try_recv() {
            port.write_all(&data).expect("Failed sending data");
        }
    });
    Ok(handle)
}
