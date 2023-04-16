use tpm2net::{Packet, PacketType};

fn main() {
    let p = Packet::new(PacketType::DataFrame, vec![0,0,0,0]);

    dbg!(&p);

    dbg!(&p.bytes());

    let parsed = Packet::parse(&vec![0xC9, 0xC0, 0x00, 0x05, 128, 0, 32, 64, 255, 0x36]).unwrap().1;
}