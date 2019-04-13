#![feature(custom_attribute)]
#![feature(plugin)]

extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate reqwest;

mod message;
use message::MessageInfo;

mod statistics;
use statistics::{Statistics, Report};

mod filter;
use filter::Filter;

mod config;
use config::{Config};

use std::fs;

use clap::{App, Arg};
use std::error::Error;
use std::io::Read;

use serde::{Serialize, Deserialize};
use serde_json::Deserializer;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use crate::config::Command;

// TODO(mkl): print messages in JSON format subcommand
// TODO(mkl): plot sequence of messages
// TODO(mkl): add comments
// TODO(mkl): add tests
// TODO(mkl): clippy
// TODO(mkl): rustfmt
// TODO(mkl): logging
// TODO(mkl): different output formats

trait MessageProcessor {
    fn init(&mut self);
    fn process_msg(&mut self, msg: &MessageInfo);
    fn finalize(&mut self);
}

#[derive(Debug, Clone)]
struct ReportGenerator {
    report: Report
}

impl ReportGenerator {
    fn new() -> ReportGenerator {
        ReportGenerator {
            report: Report::new()
        }
    }
}

impl MessageProcessor for ReportGenerator {
    fn init(&mut self) {
        self.report = Report::new();
    }

    fn process_msg(&mut self, msg: &MessageInfo) {
        self.report.process_msg(msg)
    }

    fn finalize(&mut self) {
        self.report.finalise();
        println!("{}", serde_json::to_string_pretty(&self.report).unwrap());
    }
}

// Creates diagram using websequencediagram website
#[derive(Debug, Clone)]
pub struct DiagramGenerator {
    spec: String
}

impl DiagramGenerator {
    pub fn new() -> DiagramGenerator {
        DiagramGenerator {
            spec: String::new()
        }
    }
}

// Represent response from websequence diagram website
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebSequenceDiagramResponse {
    img: String,
    errors: Vec<String>

    // TODO(mkl): add aditional fields
}

impl MessageProcessor for DiagramGenerator {
    fn init(&mut self) {
        self.spec = String::new()
    }
    fn process_msg(&mut self, msg: &MessageInfo) {
        use std::fmt::Write;
        let (src, dst) = match msg.direction.as_str() {
            "sent" => ("self".to_owned(), msg.peer_pubkey.clone()),
            "received" => (msg.peer_pubkey.clone(), "self".to_owned()),
            default=> {
                println!("Unknown direction: {}", msg.direction);
                return;
            }
        };
        write!(self.spec, "{}->{}: {}\n", src, dst, msg.type_);
    }
    fn finalize(&mut self) {
        println!("New diagram generated");
        println!("{}", self.spec);

        let resp = reqwest::Client::new()
            .post("http://www.websequencediagrams.com/index.php")
            .form(&[
                ("message", self.spec.as_ref()),
                ("style", "default"),
                ("apiVersion", "1")
            ])
            .send();
        let wr: WebSequenceDiagramResponse = match resp {
            Ok(mut r) => {
                 match serde_json::from_reader(r) {
                    Ok(r) => r,
                    Err(err) => {
                        println!("Error deserializing websequencegiagram response: {:?}", err);
                        return;
                    }
                }
            },
            Err(err) =>  {
                println!("ERROR: {}", err);
                return;
            }
        };
        println!("wr={:?}", wr);

        let mut resp2 = reqwest::Client::new()
            .get(("http://www.websequencediagrams.com/index.php".to_owned() + &wr.img).as_str())
            .send().unwrap();

        let mut f = fs::File::create("out.png").unwrap();
        // copy the response body directly to stdout
        std::io::copy(&mut resp2, &mut f);

    }
}


fn main() -> Result<(), Box<Error>> {
    let config = Config::from_command_line();
    println!("Using input file: {}", &config.input_file_name);
    let mut file = fs::File::open(&config.input_file_name)
        .map_err(|err| format!("error opening file: {} {:?}", &config.input_file_name, err) )?;
//    let mut content = String::new();
//    file.read_to_string(&mut content)
//        .map(|x| println!("read bytes: {}", x))
//        .map_err(|err| format!("cannot read from: {} {:?}", input_file_name, err))?;

//    println!("{}", content);
    let stream = Deserializer::from_reader(file)
        .into_iter::<MessageInfo>();

    let filter = Filter::new(
        config.filter_types,
        config.filter_directions,
        config.filter_peers
    );

    let mut processor: Box<MessageProcessor> = match config.command {
         Command::Report => Box::new(ReportGenerator::new()),
        Command::Diagram => Box::new(DiagramGenerator::new())
    };

    processor.init();

    for value in stream {
        match value {
            Ok(v) => {
                // TODO(mkl): add normalisation of messages, like lowercase/uppercase, types
                if !filter.pass(&v) {
                    continue;
                }
                println!("{:?}", &v);
                processor.process_msg(&v)
            },
            Err(e) => println!("ERROR: {:?}", e)
        }
    }
    processor.finalize();
    Ok(())
}
