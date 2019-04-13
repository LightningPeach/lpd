#![feature(custom_attribute)]
#![feature(plugin)]

extern crate clap;
extern crate serde;
extern crate serde_json;

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
         Command::Report => Box::new(ReportGenerator::new())
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
