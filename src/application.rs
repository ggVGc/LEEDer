// use camera::{init_camera, start_camera};
// use common::{controller::Controller, scanner::Scanner};

use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    common::{controller::Controller, protocol::Message},
    scanner::Scanner,
};

pub struct Application {
    pub leed_controller: Controller,
    pub scanner: Option<Scanner>,
}

impl Application {
    pub fn new(motors_port_name: Option<&str>) -> Self {
        let scanner: Option<Scanner> = motors_port_name.and_then(Scanner::new);
        Self {
            leed_controller: Controller::new(),
            scanner,
        }
    }

    pub fn update<F>(
        &mut self,
        leed_send: &Sender<[u8; 6]>,
        leed_responses: &Receiver<[u8; 6]>,
        on_message: F,
    ) where
        F: FnMut(Message),
    {
        self.leed_controller.update(leed_send);
        handle_leed_messages(leed_responses, &mut self.leed_controller, on_message);
        if let Some(scanner) = &mut self.scanner {
            scanner.update();
        }
    }
}

fn handle_leed_messages<F>(
    receiver: &Receiver<[u8; 6]>,
    controller: &mut Controller,
    mut on_message: F, // ui: &mut UIState,
) where
    F: FnMut(Message),
{
    while let Ok(buf) = receiver.try_recv() {
        if let Some(msg) = Message::from_bytes(&buf) {
            let mut logs = VecDeque::new();
            controller.update_from_message(msg, &mut logs);
            on_message(msg);
        }
    }
}
