use super::packet_type::PacketType;
use super::tpm2::Packet;
use std::net::UdpSocket;
use std::convert::TryInto;

const NET_PACKET_START_BYTE: u8 = 0x9C;
const NET_PACKET_END_BYTE: u8 = 0x36;

#[derive(Debug, PartialEq)]
pub struct NetPacket {
    pub packet_type: PacketType,
    pub frame_size: u16,
    pub packet_number: u8,
    pub packet_count: u8,
    pub payload: Vec<u8>,
}

impl<'a> NetPacket {
    fn parse_packet(input: &'a [u8]) -> nom::IResult<&'a [u8], NetPacket> {
        let (input, _) = nom::bytes::complete::tag(&[NET_PACKET_START_BYTE])(input)?;
        let (input, packet_type) = PacketType::parse(input)?;
        let (input, frame_size) = nom::number::complete::be_u16(input)?;
        let (input, packet_number) = nom::number::complete::be_u8(input)?;
        let (input, packet_count) = nom::number::complete::be_u8(input)?;
        let (input, payload) = nom::bytes::complete::take(frame_size - 4)(input)?;
        let (input, _) = nom::bytes::complete::tag(&[NET_PACKET_END_BYTE])(input)?;

        Ok((
            input,
            NetPacket {
                packet_type,
                frame_size,
                packet_number,
                packet_count,
                payload: payload.to_vec(),
            },
        ))
    }

    pub fn parse(
        input: &'a [u8],
    ) -> Result<(&'a [u8], NetPacket), nom::Err<nom::error::Error<&'a [u8]>>> {
        NetPacket::parse_packet(input).map_err(|e| match e {
            nom::Err::Incomplete(_) => nom::Err::Incomplete(nom::Needed::Unknown),
            nom::Err::Error(err) | nom::Err::Failure(err) => nom::Err::Error(err),
        })
    }

    fn send_tpm2_packet(packet: &Packet, socket: &UdpSocket) -> std::io::Result<()> {
        // Determine the maximum payload size for a Tpm2Net packet
        let max_payload_size = 1490; // 1490 bytes is the maximum allowed user data for a Tpm2Net packet
        let tpm2_packet_size = packet.bytes().len();
        let max_payloads_per_packet = max_payload_size / tpm2_packet_size;
        let tpm2_packet_count = ((tpm2_packet_size as f32) / max_payload_size as f32).ceil() as usize;
        
        // Split the Tpm2Packet into Tpm2Net packets
        for packet_number in 1..=tpm2_packet_count {
            let start_byte = (packet_number == 1) as u8 * 0x9C + (packet_number != 1) as u8 * 0xDC;
            let end_byte = (packet_number == tpm2_packet_count) as u8 * 0x36 + (packet_number != tpm2_packet_count) as u8 * 0xDC;
            let packet_type = PacketType::DataFrame; // Assuming all Tpm2Packets are data frames for simplicity
            
            let payload_start = (packet_number - 1) * max_payload_size;
            let payload_end = std::cmp::min(packet_number * max_payload_size, tpm2_packet_size);
            let payload = &packet.bytes()[payload_start..payload_end];
            
            let net_packet = Self {
                packet_type,
                frame_size: (payload_end - payload_start + 6) as u16,
                packet_number: packet_number as u8,
                packet_count: tpm2_packet_count as u8,
                payload: payload.try_into().unwrap(),
            };
            
            // Send the Tpm2Net packet over UDP
            let net_packet_bytes = net_packet.bytes();
            let dest_addr = "127.0.0.1:1234"; // Replace with the actual destination address
            socket.send_to(&net_packet_bytes, dest_addr)?;
        }
        
        Ok(())
    }

    fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.payload.len() + 7);
        bytes.push(NET_PACKET_START_BYTE);
        bytes.push(self.packet_type.clone() as u8);
        bytes.extend_from_slice(&(self.frame_size.to_be_bytes()));
        bytes.push(self.packet_number);
        bytes.push(self.packet_count);
        bytes.extend_from_slice(&self.payload);
        bytes.push(NET_PACKET_END_BYTE);
        bytes
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

    fn assemble_packet(&mut self, packet: NetPacket) -> Option<&[u8]> {
        let packet_num = packet.packet_number;
        let packet_count = packet.packet_count;

        // check if we already received all the packets for this message
        if self.buffer.contains_key(&packet_num) {
            return None;
        }

        // store the packet in the buffer
        self.buffer.insert(packet_num, packet.payload.to_vec());

        // if we received all the packets, re-assemble the message
        if self.buffer.len() == packet_count as usize {
            let mut payload = Vec::new();

            // concatenate all the packets' payload
            for i in 1..=packet_count {
                payload
                    .extend_from_slice(&self.buffer.remove(&i).expect("packet buffer corrupted"));
            }

            // create the Tpm2Packet from the re-assembled payload
            let packet = Packet::new(packet.packet_type, payload);

            // check if the packet is too large
            if packet.bytes().len() > self.max_packet_size {
                return None;
            }

            // store the packet in the assembled_packets buffer and return it
            self.assembled_packets.push(packet);
            return None
        } else {
            None
        }
    }

 
}
