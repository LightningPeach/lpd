extern crate protoc_rust_grpc;

use protoc_rust_grpc::{Args, Error};

fn main() -> Result<(), Error> {
    protoc_rust_grpc::run(Args {
        out_dir: "src",
        includes: &[],
        input: &["src/channel.proto"],
        rust_protobuf: true,
    })
}
