use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const BAUD_RATE: u32 = 38400;

pub fn monitor(
    port_name: &str,
    sender: mpsc::Sender<[u8; 6]>,
    receiver: mpsc::Receiver<[u8; 6]>,
) -> bool {
    let timeout = Duration::from_millis(10);

    let port = serialport::new(port_name.to_string(), BAUD_RATE)
        .timeout(timeout)
        .open();

    if let Ok(mut port) = port {
        thread::spawn(move || loop {
            let mut buf: [u8; 6] = [0; 6];
            if let Ok(_) = port.read_exact(&mut buf) {
                sender.send(buf).expect("Failed storing message.");
            }

            if let Ok(data) = receiver.try_recv() {
                port.write_all(&data).expect("Failed sending data");
            }
        });
        return true;
    } else {
        return false;
    }
}

pub fn monitor2(
    port_name: String,
    sender: mpsc::Sender<[u8; 6]>,
    sender2: mpsc::Sender<[u8; 6]>,
    receiver: mpsc::Receiver<[u8; 6]>,
) -> std::thread::JoinHandle<()> {
    let timeout = Duration::from_millis(10);

    thread::spawn(move || {
        let port_res = serialport::new(port_name, BAUD_RATE)
            .open()
            .and_then(|mut port| {
                port.set_timeout(timeout)?;
                Ok(port)
            });

        if let Ok(mut port) = port_res {
            loop {
                let mut buf: [u8; 6] = [0; 6];
                if let Ok(_) = port.read_exact(&mut buf) {
                    sender.send(buf).expect("Failed storing message.");
                    sender2.send(buf).expect("Failed storing message.");
                }

                if let Ok(data) = receiver.try_recv() {
                    port.write_all(&data).expect("Failed sending data");
                }
            }
        }
    })
}