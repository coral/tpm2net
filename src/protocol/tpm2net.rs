use super::packet_type::PacketType;
use super::tpm2::Packet;

#[derive(Debug, PartialEq)]
pub struct NetPacket<'a> {
    pub packet_type: PacketType,
    pub frame_size: u16,
    pub packet_number: u8,
    pub num_packets: u8,
    pub user_data: &'a [u8],
}

impl<'a> NetPacket<'a> {
    fn parse_packet(input: &'a [u8]) -> nom::IResult<&'a [u8], NetPacket<'a>> {
        let (input, _) = nom::bytes::complete::tag(&[0x9C])(input)?;
        let (input, packet_type) = PacketType::parse(input)?;
        let (input, frame_size) = nom::number::complete::be_u16(input)?;
        let (input, packet_number) = nom::number::complete::be_u8(input)?;
        let (input, num_packets) = nom::number::complete::be_u8(input)?;
        let (input, user_data) = nom::bytes::complete::take(frame_size - 4)(input)?;
        let (input, _) = nom::bytes::complete::tag(&[0x36])(input)?;

        Ok((
            input,
            NetPacket {
                packet_type,
                frame_size,
                packet_number,
                num_packets,
                user_data,
            },
        ))
    }

    pub fn parse(
        input: &'a [u8],
    ) -> Result<(&'a [u8], NetPacket<'a>), nom::Err<nom::error::Error<&'a [u8]>>> {
        NetPacket::parse_packet(input).map_err(|e| match e {
            nom::Err::Incomplete(_) => nom::Err::Incomplete(nom::Needed::Unknown),
            nom::Err::Error(err) | nom::Err::Failure(err) => nom::Err::Error(err),
        })
    }
}

use std::collections::HashMap;

#[derive(Default)]
struct PacketAssembler<'a> {
    buffer: HashMap<u8, Vec<u8>>,
    max_packet_size: usize,
    assembled_packets: Vec<Packet>,
    lifetime: std::marker::PhantomData<&'a u8>,
}

impl<'a> PacketAssembler<'a> {
    fn new(max_packet_size: usize) -> Self {
        Self {
            buffer: HashMap::new(),
            max_packet_size,
            assembled_packets: Vec::new(),
            lifetime: std::marker::PhantomData,
        }
    }

    fn assemble_packet(&mut self, packet: NetPacket<'a>) -> Option<&[u8]> {
        let packet_num = packet.packet_num;
        let num_packets = packet.num_packets;

        // check if we already received all the packets for this message
        if self.buffer.contains_key(&packet_num) {
            return None;
        }

        // store the packet in the buffer
        self.buffer.insert(packet_num, packet.payload.to_vec());

        // if we received all the packets, re-assemble the message
        if self.buffer.len() == num_packets as usize {
            let mut payload = Vec::new();

            // concatenate all the packets' payload
            for i in 1..=num_packets {
                payload
                    .extend_from_slice(&self.buffer.remove(&i).expect("packet buffer corrupted"));
            }

            // create the Tpm2Packet from the re-assembled payload
            let packet = Tpm2Packet::new(packet.packet_type, &payload);

            // check if the packet is too large
            if packet.bytes().len() > self.max_packet_size {
                return None;
            }

            // store the packet in the assembled_packets buffer and return it
            self.assembled_packets.push(packet);
            Some(self.assembled_packets.last().unwrap().bytes())
        } else {
            None
        }
    }
}
