use super::packet_type::PacketType;

#[derive(Debug, PartialEq)]
pub struct Tpm2NetPacket<'a> {
    pub packet_type: PacketType,
    pub frame_size: u16,
    pub packet_number: u8,
    pub num_packets: u8,
    pub user_data: &'a [u8],
}

impl<'a> Tpm2NetPacket<'a> {
    fn parse_packet(input: &'a [u8]) -> nom::IResult<&'a [u8], Tpm2NetPacket<'a>> {
        let (input, _) = nom::bytes::complete::tag(&[0x9C])(input)?;
        //let (input, packet_type) = PacketType::parse(input)?;
        let (input, frame_size) = nom::number::complete::be_u16(input)?;
        let (input, packet_number) = nom::number::complete::be_u8(input)?;
        let (input, num_packets) = nom::number::complete::be_u8(input)?;
        let (input, user_data) = nom::bytes::complete::take(frame_size - 4)(input)?;
        let (input, _) = nom::bytes::complete::tag(&[0x36])(input)?;

        Ok((
            input,
            Tpm2NetPacket {
                packet_type,
                frame_size,
                packet_number,
                num_packets,
                user_data,
            },
        ))
    }

    pub fn parse(input: &'a [u8]) -> Result<(&'a [u8], Tpm2NetPacket<'a>), nom::Err<nom::error::Error<&'a [u8]>>> {
        Tpm2NetPacket::parse_packet(input)
            .map_err(|e| match e {
                nom::Err::Incomplete(_) => nom::Err::Incomplete(nom::Needed::Unknown),
                nom::Err::Error(err) | nom::Err::Failure(err) => nom::Err::Error(err),
            })
    }
}