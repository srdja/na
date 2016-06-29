use std::collections::HashMap;
use std::string::String;
use mustache::{self, VecBuilder, MapBuilder};


pub fn render(template: String, res: &HashMap<String, String>) -> String {
    let root = MapBuilder::new().insert_vec("files", |hash_build| {
        let mut data = VecBuilder::new();
        for (uri, name) in res {
            data = data.push_map(|builder| {
                builder
                    .insert_str("url".to_string(), uri)
                    .insert_str("name".to_string(), name)
            });
        }
        data
    }).build();

    let mut buff: Vec<u8> = Vec::new();
    let template = mustache::compile_str(template.as_str());
    template.render_data(&mut buff, &root);

    String::from_utf8(buff).unwrap()
}
