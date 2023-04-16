use nom::{
    bytes::complete::{tag, take},
    error::{make_error, ErrorKind},
};

#[allow(unused)]
#[derive(Debug)]
#[repr(u8)]
pub enum PacketType {
    DataFrame,
    Command,
    RequestedResponse,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Tpm2Packet {
    start_byte: u8,
    packet_type: PacketType,
    payload_size: u16,
    user_data: Vec<u8>,
    end_byte: u8,
}

impl Tpm2Packet {
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
    
    pub fn parse_packet(input: &[u8]) -> nom::IResult<&[u8], Tpm2Packet> {
        let (input, _) = tag(&[0xC9])(input)?;
        let (input, packet_type_byte) = take(1usize)(input)?;
        let packet_type = match packet_type_byte[0] {
            0xDA => PacketType::DataFrame,
            0xC0 => PacketType::Command,
            0xAA => PacketType::RequestedResponse,
            _ => return Err(nom::Err::Failure(make_error(input, ErrorKind::Alt))),
        };
        let (input, payload_size_bytes) = take(2usize)(input)?;
        let payload_size = ((payload_size_bytes[0] as u16) << 8) | (payload_size_bytes[1] as u16);
        if payload_size < 1 || payload_size > 65535 {
            return Err(nom::Err::Failure(make_error(input, ErrorKind::Verify)));
        }
        let (input, user_data) = take(payload_size)(input)?;
        let (input, _) = tag(&[0x36])(input)?;
        Ok((input, Tpm2Packet {
            start_byte: 0xC9,
            packet_type,
            payload_size,
            user_data: user_data.to_vec(),
            end_byte: 0x36,
        }))
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
