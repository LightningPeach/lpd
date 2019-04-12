#![feature(custom_attribute)]
#![feature(plugin)]

extern crate clap;
extern crate serde;
extern crate serde_json;

use std::fs;

use clap::{App, Arg};
use std::error::Error;
use std::io::Read;

use serde::{Serialize, Deserialize};
use serde_json::Deserializer;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Serialize, Deserialize, Debug)]
struct MessageInfo {
    // hex-encoded message
    msg_raw: String,

    // hex-encoded pubkey of peer (in compressed format)
    peer_pubkey: String,

    // sent or received
    direction: String,

    #[serde(rename = "type")]
    type_ : String,

    // Unix timestamp
    time: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Statistics {
    number: u64,
    size: u64
}

impl Statistics {
    fn new() -> Statistics {
        Statistics {
            number: 0,
            size: 0
        }
    }

    fn process_msg(&mut self, x: u64) {
        self.number += 1;
        self.size += x;
    }

}

#[derive(Serialize, Deserialize, Debug)]
struct Report {
    all: Statistics,
    min_time: u64,
    max_time: u64,
    duration: u64,
    avg_msg_number_per_sec: u64,
    avg_msg_size_per_sec: u64,
    stat_by_type: HashMap<String, Statistics>,
    stat_by_peer: HashMap<String, Statistics>
}

impl Report {
    fn new() -> Report {
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

    fn process_msg(&mut self, msg: &MessageInfo) {
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

    fn finalise(&mut self) {
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

    fn process_msg_for_map(data: &mut HashMap<String, Statistics>, key: String, msg_size: u64) {
        let entry = data
            .entry(key)
            .or_insert(
                Statistics::new()
            );
        (*entry).process_msg(msg_size);
    }
}

fn main() -> Result<(), Box<Error>> {
    let matches = App::new("dump-reader")
        .version("0.0.0")
        .author("Mykola Sakhno <mykola.sakhno@bitfury.com>")
        .about("dump-reader is a tool for reading Lightning message dump files and extracting useful information")
        .arg(
            Arg::with_name("INPUT")
                .help("set the input file to use")
                .required(true)
                .index(1)
        )
        .get_matches();

    println!("Using input file: {}", matches.value_of("INPUT").unwrap());
    let input_file_name = matches.value_of("INPUT").unwrap();
    let mut file = fs::File::open(input_file_name)
        .map_err(|err| format!("error opening file: {} {:?}", input_file_name, err) )?;
//    let mut content = String::new();
//    file.read_to_string(&mut content)
//        .map(|x| println!("read bytes: {}", x))
//        .map_err(|err| format!("cannot read from: {} {:?}", input_file_name, err))?;

//    println!("{}", content);
    let stream = Deserializer::from_reader(file).into_iter::<MessageInfo>();

    let mut report = Report::new();
    for value in stream {
        match value {
            Ok(v) => {
                report.process_msg(&v);
            },
            Err(e) => println!("ERROR: {:?}", e)
        }
    }
    report.finalise();
    println!("Statistics: {:?}", report);
    Ok(())
}
