fn main() {
    use binformat::BinarySD;
    use wire::Message;
    use std::{fs::File, io::{Cursor, Read, Seek, SeekFrom}};
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

    let mut cursor = Cursor::new(data);
    loop {
        if cursor.get_ref().len() == cursor.position() as _ {
            break;
        }
        let length = BinarySD::deserialize::<u16, _>(&mut cursor).unwrap();
        let position = cursor.position() as u16;
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        println!("{:?}", msg);

        let extra_length = position + length - (cursor.position() as u16);
        if extra_length > 0 {
            println!("WARNING: extra length: {}", extra_length);
        }
        cursor.seek(SeekFrom::Start((position + length) as u64)).unwrap();
    }
}
