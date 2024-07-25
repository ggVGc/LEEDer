use super::protocol::{Control, Message, Tag};

use log::{error, info};
use std::collections::VecDeque;
use std::fmt::Display;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime};

struct Ramp {
    last_time: SystemTime,
}

impl Ramp {
    pub fn new() -> Self {
        Self {
            last_time: SystemTime::now(),
        }
    }

    pub fn ready(&mut self) -> bool {
        let time_diff = SystemTime::now()
            .duration_since(self.last_time)
            .expect("Time went backwards");

        if time_diff > Duration::from_millis(1000) {
            self.last_time = SystemTime::now();
            true
        } else {
            false
        }
    }
}

enum ValueSetter {
    Direct,
    Ramped(Ramp),
}

pub enum Unit {
    Ampere,
    MicroAmpere,
    Volt,
    KiloVolt,
    ElectronVolt,
    Percentage,
}

impl Display for Unit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Unit::Ampere => "A",
                Unit::MicroAmpere => "uA",
                Unit::Volt => "V",
                Unit::KiloVolt => "kV",
                Unit::ElectronVolt => "eV",
                Unit::Percentage => "%",
            }
        )
    }
}

pub enum Range {
    Max(f32, Unit),
    MinMax(f32, f32, Unit),
}

pub struct ControlValue {
    pub name: String,
    pub current_value: i32,
    setter: ValueSetter,
    target_value: i32,
    default: i32,
    domain_max: i32,
    range: Range,
    control: Control,
}

#[derive(PartialEq)]
pub enum Adjustment {
    Up,
    Down,
}

fn send_control_message(
    message_tag: Tag,
    value: i32,
    sender: &mpsc::Sender<[u8; 6]>,
) -> Result<(), mpsc::SendError<[u8; 6]>> {
    let msg = Message {
        tag: message_tag,
        value: value as u32,
    };

    if let Some(bytes) = msg.to_bytes() {
        sender.send(bytes)
    } else {
        // TODO: Report error in suitable way
        error!("Message not sent. Serialization failed: {:?}", msg);
        Ok(())
    }
}

impl Display for ControlValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current_ratio = self.current_value as f32 / self.domain_max as f32;
        let target_ratio = self.target_value as f32 / self.domain_max as f32;
        // let value = ratio *
        let (cur, targ, unit) = match &self.range {
            Range::Max(max_value, unit) => {
                (current_ratio * max_value, target_ratio * max_value, unit)
            }
            Range::MinMax(min_value, max_value, unit) => (
                min_value + current_ratio * (max_value - min_value),
                min_value + target_ratio * (max_value - min_value),
                unit,
            ),
        };

        write!(
            formatter,
            "{} [{}] {}  ({} / {})",
            cur, targ, unit, self.current_value, self.domain_max
        )
    }
}

impl ControlValue {
    fn new(
        name: &str,
        setter: ValueSetter,
        default: i32,
        control: Control,
        domain_max: i32,
        range: Range,
    ) -> Self {
        Self {
            current_value: 0,
            setter,
            target_value: default,
            default,
            control,
            domain_max,
            name: name.to_string(),
            range,
        }
    }

    fn update(&mut self, sender: &mpsc::Sender<[u8; 6]>) -> Result<(), mpsc::SendError<[u8; 6]>> {
        match &mut self.setter {
            ValueSetter::Direct => {
                if self.target_value != self.current_value {
                    send_control_message(Tag::Control(self.control), self.target_value, sender)
                } else {
                    Ok(())
                }
            }
            ValueSetter::Ramped(ramp) => {
                if ramp.ready() {
                    let step = (self.domain_max as f32 / 500.0) as i32;
                    if (self.target_value - self.current_value).abs() < step {
                        Ok(())
                    } else {
                        let dir = if self.target_value < self.current_value {
                            Adjustment::Down
                        } else {
                            Adjustment::Up
                        };
                        let value = self.next(self.current_value, dir);
                        info!("Ramp {}: {}", self.name, value);
                        send_control_message(Tag::Control(self.control), value, sender)
                    }
                } else {
                    Ok(())
                }
            }
        }
    }

    fn next(&self, start_value: i32, dir: Adjustment) -> i32 {
        let step = (self.domain_max as f32 / 500.0) as i32;

        let mut res = match dir {
            Adjustment::Up => start_value + step,
            Adjustment::Down => start_value - step,
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

    pub fn adjust(&mut self, adjustment: Adjustment) {
        self.target_value = self.next(self.target_value, adjustment)
    }

    pub fn send_default(
        &self,
        sender: &mpsc::Sender<[u8; 6]>,
    ) -> Result<(), mpsc::SendError<[u8; 6]>> {
        send_control_message(Tag::Control(self.control), self.default, sender)
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
    fn update(&mut self, sender: &mpsc::Sender<[u8; 6]>) {
        let controls = vec![
            &mut self.beam_energy,
            &mut self.wehnheit,
            &mut self.emission,
            &mut self.filament,
            &mut self.screen,
            &mut self.lens1_3,
            &mut self.lens2,
            &mut self.suppressor,
        ];

        for control in controls {
            if control.update(sender).is_err() {
                error!("Failed updating control: {}", control.name);
            }
        }
    }
}

impl Controls {
    fn new() -> Self {
        Self {
            filament: ControlValue::new(
                "Filament",
                ValueSetter::Ramped(Ramp::new()),
                0,
                Control::IFIL_SET1,
                63999,
                Range::Max(2.7, Unit::Ampere),
            ),
            beam_energy: ControlValue::new(
                "Beam energy",
                ValueSetter::Direct,
                3500,
                Control::BEAM_SET_INT,
                63999,
                Range::Max(1000.0, Unit::ElectronVolt),
            ),
            wehnheit: ControlValue::new(
                "Wehnheit",
                ValueSetter::Direct,
                0,
                Control::WEH_SET,
                63999,
                Range::Max(100.0, Unit::Volt),
            ),
            emission: ControlValue::new(
                "Emission",
                ValueSetter::Direct,
                16959,
                Control::EMI_SET,
                16959,
                Range::Max(50.0, Unit::MicroAmpere),
            ),
            screen: ControlValue::new(
                "Screen",
                ValueSetter::Direct,
                63999,
                Control::SCR_SET,
                63999,
                Range::Max(7.0, Unit::KiloVolt),
            ),
            // Lenses:
            // Offset: -20 - 100V
            // L2 Gain: 0 - 1.0
            // L13 Gain: 0 - 2.5
            // Output value: gain * 1000 + offset
            lens2: ControlValue::new(
                "Lens 2",
                ValueSetter::Direct,
                20000,
                Control::L2_SET,
                23734,
                Range::MinMax(-20.0, 1100.0, Unit::Volt),
            ),
            lens1_3: ControlValue::new(
                "Lens 1/3",
                ValueSetter::Direct,
                50000,
                Control::L13_SET,
                55522,
                Range::MinMax(-20.0, 2500.0, Unit::Volt),
            ),
            suppressor: ControlValue::new(
                "Suppressor",
                ValueSetter::Direct,
                26000,
                Control::RET_SET_INT,
                35199,
                Range::MinMax(10.0, 110.0, Unit::Percentage),
            ),
        }
    }
}

pub struct Controller {
    pub current: Current,
    pub controls: Controls,
    last_current_update: Instant,
    adc_counter: u8,
    defaults_counter: u8,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            current: Current::new(),
            controls: Controls::new(),
            last_current_update: Instant::now(),
            adc_counter: 0,
            defaults_counter: 0,
        }
    }

    pub fn update(&mut self, leed_sender: &mpsc::Sender<[u8; 6]>) {
        let now = Instant::now();
        let time_diff = now.duration_since(self.last_current_update);

        if time_diff > Duration::from_secs(1) {
            self.last_current_update = now;
            // TODO: Send defaults in a better way
            match self.defaults_counter {
                0 => {
                    // self.controls.beam_energy.send_default(leed_sender);
                    self.defaults_counter += 1;
                }
                1 => {
                    // self.controls.emission.send_default(leed_sender);
                    self.defaults_counter += 1;
                }
                2 => {
                    // self.controls.suppressor.send_default(leed_sender);
                    self.defaults_counter += 1;
                }
                3 => {
                    // self.controls.screen.send_default(leed_sender);
                    self.defaults_counter += 1;
                }
                4 => {
                    // self.controls.lens2.send_default(leed_sender);
                    self.defaults_counter += 1;
                }
                5 => {
                    // self.controls.lens1_3.send_default(leed_sender);
                    self.defaults_counter += 1;
                }
                _ => {
                    self.update_currents(leed_sender);
                }
            }
        }

        self.controls.update(leed_sender);
    }

    fn update_currents(&mut self, sender: &mpsc::Sender<[u8; 6]>) {
        let tag = match self.adc_counter {
            0 => Tag::ADC1,
            1 => Tag::ADC2,
            _ => Tag::ADC3,
        };

        match send_control_message(tag, 0, sender) {
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
            Tag::ADC1 => self.current.emission = v,
            Tag::ADC2 => self.current.beam = v,
            Tag::ADC3 => self.current.filament = v,
            Tag::Control(ctrl) => match ctrl {
                Control::L2_SET => self.controls.lens2.current_value = v,
                Control::L13_SET => self.controls.lens1_3.current_value = v,
                Control::WEH_SET => self.controls.wehnheit.current_value = v,
                Control::SCR_SET => self.controls.screen.current_value = v,
                Control::RET_SET_INT => self.controls.suppressor.current_value = v,
                Control::BEAM_SET_INT => self.controls.beam_energy.current_value = v,
                Control::EMI_SET => self.controls.emission.current_value = v,
                Control::IFIL_SET1 => self.controls.filament.current_value = v,
                Control::EMI_MAX => {
                    log_messages.push_front(format!("Unhandled LEED message: {:?}", msg.tag))
                }
            },

            _ => log_messages.push_front(format!("Unhandled LEED message: {:?}", msg.tag)),
        }
    }
}
