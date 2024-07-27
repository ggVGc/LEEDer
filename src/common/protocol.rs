use log::{error, warn};

#[derive(Debug, Clone, Copy)]
pub enum Tag {
    ADC1,
    ADC2,
    ADC3,
    Control(Control),
    // Status(Status),
    DigOut, // DAC(u8)
}

#[derive(Debug, Clone, Copy)]
pub struct Message {
    pub tag: Tag,
    pub value: u32,
}

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Control {
    L2_SET,
    WEH_SET,
    L13_SET,
    SCR_SET,
    RET_SET_INT,
    BEAM_SET_INT,
    IFIL_SET1,
    EMI_SET,
    EMI_MAX,
}

impl Message {
    fn from_raw(m: RawMessage) -> Option<Message> {
        let tag = match m.id {
            0x21 => Some(Tag::DigOut),

            0x31 => Some(Tag::Control(Control::L2_SET)),
            0x32 => Some(Tag::Control(Control::WEH_SET)),
            0x33 => Some(Tag::Control(Control::L13_SET)),
            0x34 => Some(Tag::Control(Control::SCR_SET)),
            0x35 => Some(Tag::Control(Control::RET_SET_INT)),
            0x36 => Some(Tag::Control(Control::BEAM_SET_INT)),
            0x37 => Some(Tag::Control(Control::IFIL_SET1)),
            0x38 => Some(Tag::Control(Control::EMI_SET)),
            0x39 => Some(Tag::Control(Control::EMI_MAX)),

            0x42 => Some(Tag::ADC1),
            0x45 => Some(Tag::ADC2),
            0x48 => Some(Tag::ADC3),

            _ => {
                warn!("Unhandled message: {:?}", m);
                None
            }
        }?;

        Some(Message {
            tag,
            value: ((m.msb as u32) << 8) + (m.lsb as u32),
        })
    }

    fn to_raw(&self) -> Option<RawMessage> {
        let id = match self.tag {
            Tag::Control(Control::L2_SET) => Some(0x31),
            Tag::Control(Control::WEH_SET) => Some(0x32),
            Tag::Control(Control::L13_SET) => Some(0x33),
            Tag::Control(Control::SCR_SET) => Some(0x34),
            Tag::Control(Control::RET_SET_INT) => Some(0x35),
            Tag::Control(Control::BEAM_SET_INT) => Some(0x36),
            Tag::Control(Control::IFIL_SET1) => Some(0x37),
            Tag::Control(Control::EMI_SET) => Some(0x38),
            Tag::Control(Control::EMI_MAX) => Some(0x39),
            Tag::ADC1 => Some(0x42),
            Tag::ADC2 => Some(0x45),
            Tag::ADC3 => Some(0x48),
            msg => {
                error!("Unimplemented: {:?}", msg);
                None
            }
        }?;

        Some(RawMessage {
            id,
            msb: (self.value >> 8) as u8,
            lsb: (self.value & 0xFF) as u8,
        })
    }

    pub fn from_bytes(bytes: &[u8; 6]) -> Option<Message> {
        let raw = RawMessage::parse(bytes)?;
        Message::from_raw(raw)
    }

    pub fn to_bytes(&self) -> Option<[u8; 6]> {
        let raw = self.to_raw()?;
        Some(raw.to_bytes())
    }
}

#[derive(Debug)]
pub struct RawMessage {
    id: u8,
    msb: u8,
    lsb: u8,
}

impl RawMessage {
    pub fn checksum(&self) -> u8 {
        0x2 ^ self.id ^ self.msb ^ self.lsb
    }

    fn parse(bytes: &[u8; 6]) -> Option<RawMessage> {
        match *bytes {
            [0x2, id, msb, lsb, bcc, 0x3] => {
                let raw_msg = RawMessage { id, msb, lsb };
                let check = raw_msg.checksum();
                if check == bcc {
                    Some(raw_msg)
                } else {
                    println!(
                        "Invalid checksum for message: {:02X?}. {}:{}",
                        bytes, check, bcc
                    );
                    None
                }
            }
            _ => {
                error!("Received unparseable message");
                None
            }
        }
    }

    pub fn to_bytes(&self) -> [u8; 6] {
        let bcc = self.checksum();
        [0x2, self.id, self.msb, self.lsb, bcc, 0x3]
    }
}

// enum ADC {
//     A1,
//     A2,
//     A3,
//     A4,
//     A5,
//     A6,
//     A7,
//     A8,
//     A9,
// }

// impl ADC {
//     fn parse(byte: u8) -> Option<ADC> {
//         match byte {
//             0x41 => Some(ADC::A1),
//             0x42 => Some(ADC::A2),
//             0x43 => Some(ADC::A3),
//             0x44 => Some(ADC::A4),
//             0x45 => Some(ADC::A5),
//             0x46 => Some(ADC::A6),
//             0x47 => Some(ADC::A7),
//             0x48 => Some(ADC::A8),
//             0x49 => Some(ADC::A9),

//             _ => None,
//         }
//     }
// }

// #[derive(Debug)]
// enum DAC {
//     D1,
//     D2,
//     D3,
//     D4,
//     D5,
//     D6,
//     D7,
//     D8,
//     D9,
// }

// impl DAC {
//     fn parse(byte: u8) -> Option<DAC> {
//         match byte {
//             0x31 => Some(DAC::D1),
//             0x32 => Some(DAC::D2),
//             0x33 => Some(DAC::D3),
//             0x34 => Some(DAC::D4),
//             0x35 => Some(DAC::D5),
//             0x36 => Some(DAC::D6),
//             0x37 => Some(DAC::D7),
//             0x38 => Some(DAC::D8),
//             0x39 => Some(DAC::D9),

//             _ => None,
//         }
//     }
// }

// #[derive(Debug)]
// #[allow(non_camel_case_types)]
// enum DigOutBits {
//     LEED_AUGER,
//     BEAM_INT_EXT,
// }

// #[derive(Debug)]
// #[allow(non_camel_case_types)]
// enum Status {
//     MON,
//     SHUTDOWN,
//     ENABLE,
//     OK_15V,
//     OK_15VHV,
//     SAFETY_SWITCH,
// }
