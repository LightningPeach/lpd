use wire::MessageExt;
use binformat::WireError;

fn de(data: Vec<u8>) -> Result<Vec<MessageExt>, WireError> {
    use binformat::BinarySD;
    use wire::{Message, MessageExt};
    use std::io::{Cursor, Seek, SeekFrom};

    let mut messages = Vec::new();

    let mut cursor = Cursor::new(data);
    loop {
        if cursor.get_ref().len() == cursor.position() as _ {
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
    use std::{fs::File, io::Read};
    use std::process::Command;

    let _ = Command::new("go")
        .current_dir("wire-compatibility")
        .arg("run").arg("lnwire_gen.go").output().unwrap();

    let data = {
        let mut file = File::open("/tmp/messages").unwrap();
        let mut v = Vec::new();
        file.read_to_end(&mut v).unwrap();
        v
    };

    let messages = de(data).unwrap();
    for message in &messages {
        println!("{:?}", message);
    }
    assert_eq!(de(ser(&messages).unwrap()).unwrap(), messages);
}
