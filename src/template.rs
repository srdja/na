use std::collections::HashMap;
use std::string::String;
use mustache::{self, VecBuilder, MapBuilder};
use directory::FileMeta;


fn format_size(bytes: u64) -> String {
    match bytes {
        b if b < 1000 => {
            return format!("{} b", b);
        },
        b @ 1000 ... 999999 => {
            let kb = b / 1000;
            let rm = (b % 1000) / 10;
            return format!("{}.{} Kb", kb, rm);
        },
        b @ 1000000 ... 999999999 => {
            let mb = b / 1000000;
            let rm = (b % 1000000) / 10000;
            return format!("{}.{} Mb", mb, rm);
        },
        b if b > 999999999 => {
            let gb = b / 1000000000;
            let rm = (b % 1000000000) / 10000000;
            return format!("{}.{} Gb", gb, rm);
        },
        _ => {
            return format!("n/a");
        }
    }
}


pub fn render(template: String, res: &HashMap<String, FileMeta>) -> String {
    let root = MapBuilder::new().insert_vec("files", |hash_build| {
        let mut data = VecBuilder::new();
        for (uri, name) in res {
            data = data.push_map(|builder| {
                builder
                    .insert_str("url".to_string(), uri)
                    .insert_str("name".to_string(), name.name.clone())
                    .insert_str("size".to_string(), format_size(name.size))
            });
        }
        data
    }).build();

    let mut buff: Vec<u8> = Vec::new();
    let template = mustache::compile_str(template.as_str());
    template.render_data(&mut buff, &root);

    String::from_utf8(buff).unwrap()
}
