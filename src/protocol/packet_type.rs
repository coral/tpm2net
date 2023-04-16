#[allow(unused)]
#[derive(Debug,PartialEq)]
#[repr(u8)]
pub enum PacketType {
    DataFrame = 0xDA,
    Command = 0xC0,
    RequestedResponse = 0xAA,
}