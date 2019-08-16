#![feature(custom_attribute)]
#![feature(plugin)]

use dependencies::clap;

mod message;
use message::MessageInfo;

mod statistics;
use statistics::{Report};

mod filter;
use filter::Filter;

mod config;
use config::{Config};

mod wsdclient;

use std::fs;

use std::error::Error;
use std::io::{Write};

use serde_json::Deserializer;

use crate::config::Command;
use crate::wsdclient::{PlotParameters};

// TODO(mkl): print messages in JSON format subcommand
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
    report: Report,
}

impl ReportGenerator {
    fn new() -> ReportGenerator {
        ReportGenerator {
            report: Report::new(),
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
    spec: String,
    plot_parameters: PlotParameters,
    output_file: String,
    spec_output_file: Option<String>,
}

// TODO(mkl): add error processing
// TODO(mkl): add checking for premium features. Like image format
impl DiagramGenerator {
    pub fn new(
        plot_parameters: PlotParameters,
        output_file: String,
        spec_output_file: Option<String>,
    ) -> DiagramGenerator {
        DiagramGenerator {
            spec: String::new(),
            plot_parameters,
            output_file,
            spec_output_file,
        }
    }
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
            _ => {
                println!("Unknown direction: {}", msg.direction);
                return;
            }
        };
        writeln!(self.spec, "{}->{}: {}", src, dst, msg.type_).unwrap();
    }
    fn finalize(&mut self) {
        if let Some(ref spec_output_file) = self.spec_output_file {
            let mut spec_f = fs::File::create(spec_output_file).unwrap();
            spec_f.write_all(self.spec.as_bytes()).unwrap();
        }

        let diag = wsdclient::get_diagram(&self.spec, &self.plot_parameters).unwrap();

        let mut f = fs::File::create(&self.output_file).unwrap();
        // copy the response body directly to stdout
        f.write_all(&diag[..]).unwrap();
    }
}

fn main() -> Result<(), Box<Error>> {
    let config = Config::from_command_line();
    println!("Using input file: {}", &config.input_file_name);
    let file = fs::File::open(&config.input_file_name)
        .map_err(|err| format!("error opening file: {} {:?}", &config.input_file_name, err))?;
    //    let mut content = String::new();
    //    file.read_to_string(&mut content)
    //        .map(|x| println!("read bytes: {}", x))
    //        .map_err(|err| format!("cannot read from: {} {:?}", input_file_name, err))?;

    //    println!("{}", content);
    let stream = Deserializer::from_reader(file).into_iter::<MessageInfo>();

    let filter = Filter::new(
        config.filter_types,
        config.filter_directions,
        config.filter_peers,
    );

    let mut processor: Box<MessageProcessor> = match config.command {
        Command::Report => Box::new(ReportGenerator::new()),
        Command::Diagram {
            output_file,
            plot_parameters,
            spec_output_file,
        } => Box::new(DiagramGenerator::new(
            plot_parameters,
            output_file,
            spec_output_file,
        )),
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
            }
            Err(e) => println!("ERROR: {:?}", e),
        }
    }
    processor.finalize();
    Ok(())
}
