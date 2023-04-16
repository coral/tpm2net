use super::packet_type::PacketType;
use nom::{
    bytes::complete::{tag, take},
    number::streaming::be_u16,
};

#[allow(unused)]
#[derive(Debug)]
pub struct Packet {
    start_byte: u8,
    packet_type: PacketType,
    payload_size: u16,
    user_data: Vec<u8>,
    end_byte: u8,
}

#[allow(dead_code)]
impl Packet {
    pub fn new(packet_type: PacketType, user_data: Vec<u8>) -> Self {
        let payload_size = user_data.len() as u16;
        Self {
            start_byte: 0xC9,
            packet_type,
            payload_size,
            user_data,
            end_byte: 0x36,
        }
    }

    fn update_payload(&mut self, new_payload: Vec<u8>) {
        self.payload_size = new_payload.len() as u16;
        self.user_data = new_payload;
    }

    pub fn parse(input: &[u8]) -> nom::IResult<&[u8], Packet> {
        let (input, _) = tag(&[0xC9])(input)?;
        let (input, packet_type) = PacketType::parse(input)?;
        let (input, payload_size) = be_u16(input)?;
        let (input, user_data) = take(payload_size)(input)?;
        let (input, _) = tag(&[0x36])(input)?;
        Ok((
            input,
            Packet {
                start_byte: 0xC9,
                packet_type,
                payload_size,
                user_data: user_data.to_vec(),
                end_byte: 0x36,
            },
        ))
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.push(0xC9);
        bytes.push(match self.packet_type {
            PacketType::DataFrame => 0xDA,
            PacketType::Command => 0xC0,
            PacketType::RequestedResponse => 0xAA,
        });
        bytes.extend_from_slice(&(self.payload_size.to_be_bytes()));
        bytes.extend_from_slice(&self.user_data);
        bytes.push(0x36);
        bytes
    }
}

impl Into<Vec<u8>> for Packet {
    fn into(self) -> Vec<u8> {
        self.bytes()
    }
}

impl<'a> TryFrom<&'a [u8]> for Packet {
    type Error = nom::Err<nom::error::Error<&'a [u8]>>;
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Self::parse(value).map(|(_, p)| p)
    }
}

#[test]
fn create() {
    let p = Packet::new(PacketType::Command, vec![128, 0, 32, 64, 255]);
    let comp = hex::decode("C9C0000580002040FF36").unwrap();

    assert_eq!(p.bytes(), comp);
}
#[test]
fn parse() {
    // Even length
    let parsed = Packet::parse(&vec![0xC9, 0xC0, 0x00, 0x04, 128, 0, 32, 64, 0x36])
        .unwrap()
        .1;
    let constructed = Packet::new(PacketType::Command, vec![128, 0, 32, 64]);

    assert_eq!(parsed.bytes(), constructed.bytes());

    // Odd length
    let parsed = Packet::parse(&vec![0xC9, 0xC0, 0x00, 0x05, 128, 0, 32, 64, 255, 0x36])
        .unwrap()
        .1;
    let constructed = Packet::new(PacketType::Command, vec![128, 0, 32, 64, 255]);

    assert_eq!(parsed.bytes(), constructed.bytes());
}

#[test]
fn change() {
    let mut p = Packet::new(PacketType::Command, vec![0]);
    assert_eq!(p.bytes(), vec![0xC9, 0xC0, 0x00, 0x01, 0, 0x36]);

    p.update_payload(vec![0x12, 0x12]);
    assert_eq!(p.bytes(), vec![0xC9, 0xC0, 0x00, 0x02, 0x12, 0x12, 0x36]);
}
