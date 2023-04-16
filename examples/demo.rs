use tpm2net::{Packet, PacketType};

fn main() {
    let p = Packet::new(PacketType::DataFrame, vec![0,0,0,0]);

    dbg!(&p);

    dbg!(&p.bytes());
}