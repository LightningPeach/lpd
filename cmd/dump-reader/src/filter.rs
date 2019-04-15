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

    #[allow(dead_code)]
    pub fn empty() -> Filter {
        Filter::new(vec![], vec![], vec![])
    }


    pub fn pass(&self, msg: &MessageInfo) -> bool {
        (self.types.is_empty() || self.types.contains(&msg.type_))
            && (self.directions.is_empty() || self.directions.contains(&msg.direction))
            && (self.peers.is_empty() || self.peers.contains(&msg.peer_pubkey))
    }
}