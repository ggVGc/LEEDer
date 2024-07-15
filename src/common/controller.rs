use super::protocol::{Control, Message, MessageTag};

use log::error;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

pub enum RangeUnit {
    Ampere,
    MicroAmpere,
    Volt,
    KiloVolt,
    ElectronVolt,
    Percentage,
}

impl RangeUnit {
    pub fn to_string(&self) -> String {
        match self {
            RangeUnit::Ampere => "A",
            RangeUnit::MicroAmpere => "uA",
            RangeUnit::Volt => "V",
            RangeUnit::KiloVolt => "kV",
            RangeUnit::ElectronVolt => "eV",
            RangeUnit::Percentage => "%",
        }
        .to_string()
    }
}

pub enum RangeValue {
    Value(f32, RangeUnit),
    MinMaxValue(f32, f32, RangeUnit),
}

pub struct ControlValue {
    pub value: i32,
    default: i32,
    domain_max: i32,
    pub name: String,
    range_max: RangeValue,
    control: Control,
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
        let ratio = self.value as f32 / self.domain_max as f32;
        // let value = ratio *
        match &self.range_max {
            RangeValue::Value(max_value, unit) => {
                let value = ratio * max_value;
                return format!("{} {}", value, unit.to_string()).to_string();
            }
            RangeValue::MinMaxValue(min_value, max_value, unit) => {
                let value = min_value + ratio * (max_value - min_value);
                return format!("{} {}", value, unit.to_string()).to_string();
            }
        }
    }

    fn next(&self, dir: Adjustment) -> i32 {
        let step = (self.domain_max as f32 / 500.0) as i32;

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
        send_control_change(MessageTag::Control(self.control), value, sender)
    }

    pub fn send_default(
        &self,
        sender: &mpsc::Sender<[u8; 6]>,
    ) -> Result<(), mpsc::SendError<[u8; 6]>> {
        send_control_change(MessageTag::Control(self.control), self.default, sender)
    }
}

impl ControlValue {
    pub fn new(
        name: &str,
        default: i32,
        control: Control,
        domain_max: i32,
        range_max: RangeValue,
    ) -> Self {
        Self {
            value: 0,
            default,
            control,
            domain_max,
            name: name.to_string(),
            range_max,
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
            beam_energy: ControlValue::new(
                "Beam energy",
                3500,
                Control::BEAM_SET_INT,
                63999,
                RangeValue::Value(1000.0, RangeUnit::ElectronVolt),
            ),
            wehnheit: ControlValue::new(
                "Wehnheit",
                0,
                Control::WEH_SET,
                63999,
                RangeValue::Value(100.0, RangeUnit::Volt),
            ),
            emission: ControlValue::new(
                "Emission",
                16959,
                Control::EMI_SET,
                16959,
                RangeValue::Value(50.0, RangeUnit::MicroAmpere),
            ),
            filament: ControlValue::new(
                "Filament",
                0,
                Control::IFIL_SET1,
                63999,
                RangeValue::Value(2.7, RangeUnit::Ampere),
            ),
            screen: ControlValue::new(
                "Screen",
                0,
                Control::SCR_SET,
                63999,
                RangeValue::Value(7.0, RangeUnit::KiloVolt),
            ),
            // Lenses:
            // Offset: -20 - 100V
            // L2 Gain: 0 - 1.0
            // L13 Gain: 0 - 2.5
            // Output value: gain * 1000 + offset
            lens2: ControlValue::new(
                "Lens 2",
                0,
                Control::L2_SET,
                23734,
                RangeValue::MinMaxValue(-20.0, 1100.0, RangeUnit::Volt),
            ),
            lens1_3: ControlValue::new(
                "Lens 1/3",
                0,
                Control::L13_SET,
                55522,
                RangeValue::MinMaxValue(-20.0, 2500.0, RangeUnit::Volt),
            ),
            suppressor: ControlValue::new(
                "Suppressor",
                26000,
                Control::RET_SET_INT,
                35199,
                RangeValue::Value(110.0, RangeUnit::Percentage),
            ),
        }
    }
}

pub struct Controller {
    pub current: Current,
    pub controls: Controls,
    last_current_update: SystemTime,
    adc_counter: u8,
    defaults_counter: u8,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            current: Current::new(),
            controls: Controls::new(),
            last_current_update: SystemTime::now(),
            adc_counter: 0,
            defaults_counter: 0,
        }
    }

    pub fn update(&mut self, leed_sender: &mpsc::Sender<[u8; 6]>) {
        let time_diff = SystemTime::now()
            .duration_since(self.last_current_update)
            .expect("Time went backwards");
        if time_diff > Duration::from_secs(1) {
            self.last_current_update = SystemTime::now();
            // TODO: Send defaults in a better way
            match self.defaults_counter {
                0 => {
                    self.controls.beam_energy.send_default(leed_sender);
                    self.defaults_counter += 1;
                }
                1 => {
                    self.controls.emission.send_default(leed_sender);
                    self.defaults_counter += 1;
                }
                2 => {
                    self.controls.suppressor.send_default(leed_sender);
                    self.defaults_counter += 1;
                }
                _ => self.update_currents(&leed_sender),
            }
        }
    }

    fn update_currents(&mut self, sender: &mpsc::Sender<[u8; 6]>) {
        match send_control_change(MessageTag::ADC(self.adc_counter + 1), 0, sender) {
            Ok(_) => {
                self.adc_counter = (self.adc_counter + 1) % 3;
            }
            Err(err) => {
                error!("Query of current failed: {:?}", err);
            }
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
