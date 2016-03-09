
use std::fs;

pub struct Directory {
    root: String,
}


impl Directory {

    pub fn new(root: String) -> Directory {
        Directory {
            root: root,
        }
    }

    pub fn list_available_resources(&self) -> Vec<String> {
        let mut files: Vec<String> = Vec::new();
        let paths = fs::read_dir(&*(self.root)).unwrap();

        for p in paths {
            let pu = p.unwrap();
            if pu.file_type().unwrap().is_file() {
                files.push(format!("/{}", pu.file_name().into_string().unwrap()));
            }
        }
        files
    }

    pub fn full_path(&self, name: String) -> String {
        self.root.clone() + "/" + &name
    }
}
