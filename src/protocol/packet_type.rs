use nom::{number::streaming::be_u8, IResult};
#[allow(unused)]
#[derive(Debug, PartialEq)]
#[repr(u8)]

pub enum PacketType {
    DataFrame = 0xDA,
    Command = 0xC0,
    RequestedResponse = 0xAA,
}

impl PacketType {
    pub fn parse(i: &[u8]) -> IResult<&[u8], PacketType> {
        let (i, packet_type) = be_u8(i)?;
        match packet_type {
            0xDA => Ok((i, PacketType::DataFrame)),
            0xC0 => Ok((i, PacketType::Command)),
            0xAA => Ok((i, PacketType::RequestedResponse)),
            _ => Err(nom::Err::Failure(nom::error::make_error(
                i,
                nom::error::ErrorKind::Verify,
            ))),
        }
    }
}
