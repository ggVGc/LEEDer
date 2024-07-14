use super::protocol::{Control, Message, MessageTag};

use std::collections::VecDeque;
use std::sync::mpsc;

pub enum RangeUnit {
    Ampere(f32),
    MicroAmpere(f32),
    Volt(f32),
    Percentage(f32),
    Dummy,
}

pub struct ControlValue {
    pub value: i32,
    domain_max: i32,
    pub name: String,
    range_max: RangeUnit,
}

#[derive(PartialEq)]
pub enum Adjustment {
    Up,
    Down,
}

fn send_control_change(
    message_tag: MessageTag,
    value: i32,
    sender: &mpsc::Sender<[u8; 6]>,
) -> Result<(), mpsc::SendError<[u8; 6]>> {
    let msg = Message {
        tag: message_tag,
        value: value as u32,
    };
    let bytes = msg.to_bytes();
    sender.send(bytes)
}

impl ControlValue {
    pub fn to_string(&self) -> String {
        return format!("{}", self.value as f32 / self.domain_max as f32).to_string();
    }

    fn next(&self, dir: Adjustment) -> i32 {
        let step = (self.domain_max as f32 / 100.0) as i32;

        let mut res = match dir {
            Adjustment::Up => self.value + step,
            Adjustment::Down => self.value - step,
        };

        if res < 0 {
            res = 0;
        }

        if res > self.domain_max {
            res = self.domain_max;
        }

        res
        // info!("Increased value: {}", self.value);
    }

    pub fn adjust(
        &self,
        adjustment: Adjustment,
        sender: &mpsc::Sender<[u8; 6]>,
    ) -> Result<(), mpsc::SendError<[u8; 6]>> {
        let value = self.next(adjustment);
        send_control_change(MessageTag::Control(Control::BEAM_SET_INT), value, sender)
    }
}

impl ControlValue {
    // pub fn new(domain_max: i32, range_max: RangeUnit ) -> Self {
    pub fn new(name: &str, domain_max: i32) -> Self {
        Self {
            value: 0,
            domain_max,
            name: name.to_string(),
            range_max: RangeUnit::Dummy,
        }
    }
}

pub struct Current {
    pub beam: i32,
    pub emission: i32,
    pub filament: i32,
}

impl Current {
    fn new() -> Self {
        Self {
            beam: 0,
            emission: 0,
            filament: 0,
        }
    }
}

pub struct Controls {
    pub beam_energy: ControlValue,
    pub wehnheit: ControlValue,
    pub emission: ControlValue,
    pub filament: ControlValue,
    pub screen: ControlValue,
    pub lens1_3: ControlValue,
    pub lens2: ControlValue,
    pub suppressor: ControlValue,
}

impl Controls {
    fn new() -> Self {
        Self {
            beam_energy: ControlValue::new("Beam energy", 63999),
            wehnheit: ControlValue::new("Wehnheit", 63999),
            emission: ControlValue::new("Emission", 16959),
            filament: ControlValue::new("Filament", 63999),
            screen: ControlValue::new("Screen", 63999),
            lens1_3: ControlValue::new("Lens 1/3", 55522),
            lens2: ControlValue::new("Lens 2", 23734),
            // suppressor: ControlValue::new(35199, RangeUnit::Percentage(110.0)),
            suppressor: ControlValue::new("Suppressor", 35199),
        }
    }
}

pub struct Controller {
    pub current: Current,
    pub controls: Controls,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            current: Current::new(),
            controls: Controls::new(),
        }
    }

    pub fn update_from_message(&mut self, msg: Message, log_messages: &mut VecDeque<String>) {
        let v = msg.value as i32;
        match &msg.tag {
            // MessageTag::Arbitrary(_) => todo!(),
            MessageTag::ADC(1) => self.current.emission = v,
            MessageTag::ADC(2) => self.current.beam = v,
            MessageTag::ADC(3) => self.current.filament = v,
            MessageTag::Control(ctrl) => match ctrl {
                Control::L2_SET => self.controls.lens2.value = v,
                Control::L13_SET => self.controls.lens1_3.value = v,
                Control::WEH_SET => self.controls.wehnheit.value = v,
                Control::SCR_SET => self.controls.screen.value = v,
                Control::RET_SET_INT => self.controls.suppressor.value = v,
                Control::BEAM_SET_INT => self.controls.beam_energy.value = v,
                Control::EMI_SET => self.controls.emission.value = v,
                Control::IFIL_SET1 => self.controls.filament.value = v,
                Control::EMI_MAX => {
                    log_messages.push_front(format!("Unhandled LEED message: {:?}", msg.tag))
                }
            },
            // MessageTag::DigOut => {}

            // MessageTag::Arbitrary(_) => {
            _ => log_messages.push_front(format!("Unhandled LEED message: {:?}", msg.tag)),
        }
    }
}
