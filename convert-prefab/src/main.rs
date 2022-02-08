use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use serde_json::{Map, Value};
use toml::Serializer;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 1 {
        return eprintln!("USAGE: convert-prefab <FILE>");
    }
    let path = PathBuf::from(args.first().unwrap());
    let file = File::open(&path);
    if let Err(_) = file {
        return eprintln!("Error: that file does not exist.");
    }

    let mut file = file.unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Is your file system broken?");

    let data = if let Ok(decoded_bytes) = base64::decode(&contents) {
        eprintln!("Found base64, continueing");
        String::from_utf8_lossy(&decoded_bytes).to_string()
    } else {
        contents
    };

    let deserialized: Map<String, Value> =
        serde_json::from_str(&data).expect("This isn't valid JSON.");
    let deserialized: Map<String, Value> = deserialized
        .into_iter()
        .filter(|x| !x.1.is_null())
        .collect();
    let mut toml = String::new();
    // Otherwise it has a bug:
    // https://github.com/alexcrichton/toml-rs/issues/142#issuecomment-278970591
    toml::ser::tables_last(
        &deserialized,
        Serializer::new(&mut toml)
            .pretty_array(true)
            .pretty_string(true)
            .pretty_array_indent(4)
            .pretty_string_literal(true)
            .pretty_array_trailing_comma(true),
    )
    .expect("Could not serialize to toml");

    let mut move_target = path.clone();
    move_target.set_extension("toml");
    if let Err(_) = fs::remove_file(path) {
        eprintln!("WARNING: could not remove original json file");
    }
    if move_target.exists() {
        let mut bak_target = move_target.clone();
        bak_target.set_extension("bak");
        let _ = fs::copy(&move_target, bak_target);
    }
    let mut new_file = File::create(move_target).expect("Could not create the new toml file");
    new_file
        .write(toml.as_bytes())
        .expect("Could not write to the new toml file");
}
