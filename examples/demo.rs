use tpm2net::{TPM2Packet, PacketType};

fn main() {
    let p = TPM2Packet::new(PacketType::DataFrame, vec![0,0,0,0]);

    dbg!(&p);

    dbg!(&p.bytes());

    let parsed = TPM2Packet::parse(&vec![0xC9, 0xC0, 0x00, 0x05, 128, 0, 32, 64, 255, 0x36]).unwrap().1;
}