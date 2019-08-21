use dependencies::clap;

use wire::{Message, MessageExt};
use binformat::WireError;

fn de(data: Vec<u8>) -> Result<Vec<MessageExt>, WireError> {
    use binformat::BinarySD;
    use std::io::{Cursor, Seek, SeekFrom};

    let mut messages = Vec::new();

    let mut cursor = Cursor::new(data);
    loop {
        if cursor.get_ref().len() == cursor.position() as usize {
            break;
        }
        let length = BinarySD::deserialize::<u16, _>(&mut cursor)?;
        let position = cursor.position() as u16;
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor)?;

        let extra_length = position + length - (cursor.position() as u16);
        let mut extra_data = Vec::new();
        if extra_length > 0 {
            extra_data.extend_from_slice(&cursor.get_ref()[(cursor.position() as usize)..((position + length) as usize)]);
            cursor.seek(SeekFrom::Current(extra_length as i64)).unwrap();
        }
        let message_ext = MessageExt {
            message: msg,
            extra_data: extra_data,
        };
        messages.push(message_ext);
    }

    Ok(messages)
}

fn ser(messages: &Vec<MessageExt>) -> Result<Vec<u8>, WireError> {
    use binformat::BinarySD;

    messages.iter()
        .try_fold(Vec::new(), |mut x, message_ext| {
            let length_position = x.len();
            BinarySD::serialize(&mut x, &0u16)?;
            let position = x.len();
            BinarySD::serialize(&mut x, &message_ext.message)?;
            x.extend_from_slice(message_ext.extra_data.as_ref());
            let length = (x.len() - position) as u16;
            let mut y = Vec::with_capacity(2);
            BinarySD::serialize(&mut y, &length)?;
            x[length_position..(length_position + 2)].copy_from_slice(y.as_ref());
            Ok(x)
        })
}

fn main() {
    use clap::{App, Arg};
    use std::{fs::File, io::{Read, Write}};

    let matches = App::new("wire-compatibility")
        .arg(
            Arg::with_name("input-binary")
                .long("input-binary")
                .short("i")
                .value_name("INPUT_BINARY")
                .help("path to binary file containing input data")
                .takes_value(true)
                .default_value("/tmp/messages")
        )
        .arg(
            Arg::with_name("output-binary")
                .long("output-binary")
                .short("b")
                .value_name("INPUT_BINARY")
                .help("path where to store binary file containing output data")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("output-text")
                .long("output-text")
                .short("t")
                .value_name("INPUT_TEXT")
                .help("path where to store text file containing output data")
                .takes_value(true)
                .default_value("/dev/stdout")
        )
        .get_matches();

    let data = {
        let mut file = File::open(matches.value_of("input-binary").unwrap())
            .expect("missing input file");
        let mut v = Vec::new();
        file.read_to_end(&mut v).unwrap();
        v
    };

    let messages = de(data).unwrap();
    let _ = matches.value_of("output-binary")
        .map(|output_binary_path| {
            let mut file = File::create(output_binary_path).unwrap();
            let data = ser(&messages).unwrap();
            file.write_all(data.as_ref()).unwrap();
        });
    let _ = matches.value_of("output-text")
        .map(|output_text_path| {
            let mut file = File::create(output_text_path).unwrap();
            // TODO(vlad): print is not pretty enough, use RON (https://github.com/ron-rs/ron)
            file.write_fmt(format_args!("{:#?}", messages)).unwrap();
        });

    // assert_eq!(de(ser(&messages).unwrap()).unwrap(), messages);
}
