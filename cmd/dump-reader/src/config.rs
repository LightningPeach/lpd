use clap::{App, Arg, AppSettings, SubCommand};

use crate::wsdclient;
use crate::wsdclient::{WSDEnum, Format, Style, PaperSize, PaperOrientation};

fn split_list(s: Option<&str>) -> Vec<String> {
    match s {
        Some(s) => s.split(",").map(|x| x.to_owned()).collect(),
        None => vec![]
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Report,
    Diagram {
        output_file: String,
        plot_parameters: wsdclient::PlotParameters,
        spec_output_file: Option<String>
    }
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
            .subcommand(
                SubCommand::with_name("diagram")
                    .about("Generate diagram")
                    .version("0.0.0")
                    .arg(
                        Arg::with_name("output-file")
                            .help("Output file for diagram. By default out.<format> is used. E.g. out.png")
                            .long("output")
                            .short("o")
                            .takes_value(true)
                    )
                    .arg(
                        Arg::with_name("api-key")
                            .help("websequencediagram api key. For security reason it is better to use environmental variable WEBSEQUENCEDIAGRAM_API_KEY. Command line option has higher precedence over environment variable. Api key can be obtained by going to http://www.websequencediagrams.com/users/getapikey while logged in.")
                            .long("api-key")
                            .takes_value(true)
                    )
                    .arg(
                        Arg::with_name("format")
                            .help(&format!("Format of the output file. Some formats are premium. Possible values: {}. Default value is png", wsdclient::Format::help_str()))
                            .long("format")
                            .takes_value(true)
                    )
                    .arg(
                        Arg::with_name("style")
                            .help(&format!("Style to use. Possible styles are {}. Default value: {}", Style::help_str(), Style::Default.human_readable_value()))
                            .long("style")
                            .takes_value(true)
                    )
                    .arg(
                        Arg::with_name("spec-output-file")
                            .help("File to write diagram in text format. This text is used to create image by websequencediagram API")
                            .long("spec")
                            .takes_value(true)
                    )
                    .arg(
                        Arg::with_name("paper-size")
                            .help(&format!("Paper size to use. Useful only for pdf output format. Possible values: {}. By default it is not included into request.", PaperSize::help_str()))
                            .long("paper-size")
                            .takes_value(true)
                    )
                    .arg(
                        Arg::with_name("paper-orientation")
                            .help(&format!("Paper orientation to use. Useful only for pdf output format. Possible values: {}. By default it is not included into request.", PaperOrientation::help_str()))
                            .long("paper-orientation")
                            .takes_value(true)
                    )
                    .arg(
                        Arg::with_name("scale")
                            .help("Scale. Default value is 100. High res is 200. It seems it only useful for png format. By default it is not included into request.")
                            .long("scale")
                            .takes_value(true)
                    )
            )
            .get_matches();

        let filter_types = split_list(matches.value_of("filter-type"));
        let filter_directions = split_list(matches.value_of("filter-direction"));
        let filter_peers = split_list(matches.value_of("filter-peer"));
        let input_file_name = matches.value_of("INPUT").unwrap().to_owned();

        let mut command = Command::Report;
        if matches.is_present("report") {
            command = Command::Report;
        } else if matches.is_present("diagram") {
            let sub_matches= matches.subcommand_matches("diagram").unwrap();

            let mut api_key: Option<String> = None;
            if let Some(api_key_arg) = sub_matches.value_of("api-key") {
                api_key = Some(api_key_arg.to_owned())
            } else {
                if let Ok(api_key_env) = std::env::var("WEBSEQUENCEDIAGRAM_API_KEY") {
                    api_key = Some(api_key_env);
                }
            }

            let mut format = Format::Png;
            if let Some(format_arg_str) = sub_matches.value_of("format") {
                if let Some(format_arg) = Format::from_str(format_arg_str) {
                    format = format_arg;
                } else {
                    println!("ERROR: incorrect format value. Possible values are: {}. Got: {}", Format::help_str(), format_arg_str);
                    std::process::exit(1);
                }
            }

            let mut style = Style::Default;
            if let Some(style_arg_str) = sub_matches.value_of("style") {
                if let Some(style_arg) = Style::from_str(style_arg_str) {
                    style = style_arg;
                } else {
                    println!("ERROR: incorrect style value. Possible values are: {}. Got: {}", Style::help_str(), style_arg_str);
                    std::process::exit(1);
                }
            }

            let mut paper_size: Option<PaperSize> = None;
            if let Some(paper_size_arg_str) = sub_matches.value_of("paper-size") {
                if let Some(paper_size_arg) = PaperSize::from_str(paper_size_arg_str) {
                    paper_size = Some(paper_size_arg)
                } else {
                    println!("ERROR: incorrect paper-size value. Possible values are: {}. Got: {}", PaperSize::help_str(), paper_size_arg_str);
                    std::process::exit(1);
                }
            }

            let mut paper_orientation: Option<PaperOrientation> = None;
            if let Some(paper_orientation_arg_str) = sub_matches.value_of("paper-orientation") {
                if let Some(paper_orientation_arg) = PaperOrientation::from_str(paper_orientation_arg_str) {
                    paper_orientation = Some(paper_orientation_arg)
                } else {
                    println!("ERROR: incorrect paper-orientation value. Possible values are: {}. Got: {}", PaperOrientation::help_str(), paper_orientation_arg_str);
                    std::process::exit(1);
                }
            }

            let mut scale: Option<u32> = None;
            if let Some(scale_arg_str) = sub_matches.value_of("scale") {
                use std::str::FromStr;
                if let Ok(scale_arg) = u32::from_str(scale_arg_str) {
                    scale = Some(scale_arg)
                } else {
                    println!("ERROR: incorrect scale value. It shoulf be positive integer. Got: {}", scale_arg_str);
                    std::process::exit(1);
                }
            }

            let output_file: String = if let Some(output_file_arg) = sub_matches.value_of("output-file") {
                output_file_arg.to_owned()
            } else {
                format!("out.{}", format.wsd_value())
            };

            let spec_output_file: Option<String> = if let Some(spec_output_file_arg) = sub_matches.value_of("spec-output-file") {
                Some(spec_output_file_arg.to_owned())
            } else {
                None
            };

            let plot_parameters = wsdclient::PlotParameters {
                style: style,
                format: format,
                paper_size: paper_size,
                paper_orientation: paper_orientation,
                scale: scale,
                api_key: api_key
            };
            command = Command::Diagram {
                output_file: output_file,
                plot_parameters: plot_parameters,
                spec_output_file: spec_output_file
            }
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