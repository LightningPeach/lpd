fn main() {
    use binformat::BinarySD;
    use wire::Message;
    use std::{fs::File, io::{Cursor, Read}};
    use std::process::Command;

    let _ = Command::new("go")
        .current_dir("wire-compatibility")
        .arg("run").arg("lnwire_gen.go").output().unwrap();

    let data = {
        let mut file = File::open("target/messages").unwrap();
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
        assert_eq!(position + length, cursor.position() as u16);
    }
}
