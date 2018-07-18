
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Ping {
    pong_length: u16,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Pong {
    data: Vec<u8>,
}
