use clap::{App, Arg, AppSettings, SubCommand};

fn split_list(s: Option<&str>) -> Vec<String> {
    match s {
        Some(s) => s.split(",").map(|x| x.to_owned()).collect(),
        None => vec![]
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Report
}

#[derive(Debug, Clone)]
pub struct Config {
    pub input_file_name: String,

    pub filter_types: Vec<String>,
    pub filter_directions: Vec<String>,
    pub filter_peers: Vec<String>,

    pub command: Command
}

impl Config {
    pub fn from_command_line() -> Config {
        let matches = App::new("dump-reader")
            .version("0.0.0")
            .author("Mykola Sakhno <mykola.sakhno@bitfury.com>")
            .about("dump-reader is a tool for reading Lightning message dump files and extracting useful information")
            .setting(AppSettings::SubcommandRequired)
            .arg(
                Arg::with_name("INPUT")
                    .help("set the input file to use")
                    .required(true)
                    .index(1)
            )
            .arg(
                Arg::with_name("filter-type")
                    .long("filter-type")
                    .help("comma delimited list of required message types")
                    .takes_value(true)
            )
            .arg(
                Arg::with_name("filter-direction")
                    .long("filter-direction")
                    .help("comma delimited list of required directions")
                    .takes_value(true)
            )
            .arg(
                Arg::with_name("filter-peer")
                    .long("filter-peer")
                    .help("comma delimited list of required peer")
                    .takes_value(true)
            )
            .subcommand(
            SubCommand::with_name("report")
                    .about("Generate report about size and number of messages")
                    .version("0.0.0")
            )
            .get_matches();

        let filter_types = split_list(matches.value_of("filter-type"));
        let filter_directions = split_list(matches.value_of("filter-direction"));
        let filter_peers = split_list(matches.value_of("filter-peer"));
        let input_file_name = matches.value_of("INPUT").unwrap().to_owned();

        let mut command = Command::Report;
        if matches.is_present("report") {
            command = Command::Report;
        }

        Config {
            input_file_name,
            filter_types,
            filter_directions,
            filter_peers,
            command
        }
    }
}