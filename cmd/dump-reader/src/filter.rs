use crate::message::MessageInfo;

pub struct Filter {
    pub types: Vec<String>,
    pub directions: Vec<String>,
    pub peers: Vec<String>
}

impl Filter {
    // TODO(mkl): add case-insensitive filters
    pub fn new(types: Vec<String>, directions: Vec<String>, peers: Vec<String>) -> Filter {
        Filter {
            types,
            directions,
            peers
        }
    }
    pub fn empty() -> Filter {
        Filter::new(vec![], vec![], vec![])
    }
    pub fn pass(&self, msg: &MessageInfo) -> bool {
        (self.types.len() == 0 || self.types.contains(&msg.type_))
            && (self.directions.len() == 0 || self.directions.contains(&msg.direction))
            && (self.peers.len() == 0 || self.peers.contains(&msg.peer_pubkey))
    }
}