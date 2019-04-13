use std::collections::HashMap;
use std::collections::hash_map::Entry;

use serde::{Serialize, Deserialize};

use crate::message::MessageInfo;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Statistics {
    pub number: u64,
    pub size: u64
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            number: 0,
            size: 0
        }
    }

    pub fn process_msg(&mut self, x: u64) {
        self.number += 1;
        self.size += x;
    }

}

#[derive(Serialize, Deserialize, Debug)]
pub struct Report {
    pub all: Statistics,
    pub min_time: u64,
    pub max_time: u64,
    pub duration: u64,
    pub avg_msg_number_per_sec: u64,
    pub avg_msg_size_per_sec: u64,
    pub stat_by_type: HashMap<String, Statistics>,
    pub stat_by_peer: HashMap<String, Statistics>
}

impl Report {
    pub fn new() -> Report {
        Report{
            all: Statistics::new(),
            min_time: std::u64::MAX,
            max_time: 0,
            duration: 0,
            avg_msg_number_per_sec: 0,
            avg_msg_size_per_sec: 0,
            stat_by_type: HashMap::new(),
            stat_by_peer: HashMap::new(),
        }
    }

    pub fn process_msg(&mut self, msg: &MessageInfo) {
        // msg_raw is hex encoded, so to obtain byte size we need to divide by 2
        let msg_size = (msg.msg_raw.len() as u64) / 2;
        self.all.process_msg(msg_size);

        if let Ok(t) = msg.time.parse::<u64>() {
            if t < self.min_time {
                self.min_time = t;
            }
            if t > self.max_time {
                self.max_time = t;
            }
        }

        Self::process_msg_for_map(&mut self.stat_by_peer, msg.peer_pubkey.clone(), msg_size);
        Self::process_msg_for_map(&mut self.stat_by_type, msg.type_.clone(), msg_size);
    }

    pub fn finalise(&mut self) {
        if self.min_time <= self.max_time {
            self.duration = self.max_time - self.min_time
        } else {
            self.duration = 0
        }
        if self.duration > 0 {
            self.avg_msg_number_per_sec = self.all.number / self.duration;
            self.avg_msg_size_per_sec = self.all.size / self.duration;
        }
    }

    pub fn process_msg_for_map(data: &mut HashMap<String, Statistics>, key: String, msg_size: u64) {
        let entry = data
            .entry(key)
            .or_insert(
                Statistics::new()
            );
        (*entry).process_msg(msg_size);
    }

    // TODO(mkl): add printing report in text format. Like ordered by size of messages
}