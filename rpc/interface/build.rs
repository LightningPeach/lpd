extern crate protoc_rust_grpc;

use protoc_rust_grpc::{Args, Error};

fn main() -> Result<(), Error> {
    use std::fs;

    let inputs = vec!["common", "payment", "routing", "channel"];
    let inputs = inputs.into_iter().filter(|&name| {
        let input = format!("{}.proto", name);
        let output = format!("src/{}.rs", name);

        match fs::metadata(output) {
            Ok(metadata) => match metadata.modified() {
                Ok(output_time) => {
                    let input_time = fs::metadata(input).unwrap().modified().unwrap();
                    input_time > output_time
                },
                Err(_) => true,
            },
            Err(_) => true,
        }
    }).collect::<Vec<_>>();

    if !inputs.is_empty() {
        protoc_rust_grpc::run(Args {
            out_dir: "src",
            includes: &["."],
            input: inputs.as_slice(),
            rust_protobuf: true,
        })
    } else {
        Ok(())
    }
}
