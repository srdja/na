/*
 * na
 *
 * Copyright (C) 2016 Srđan Panić <sp@srdja.me>
 *
 * na is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * na is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with na.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;
use chrono::naive::datetime::NaiveDateTime;
use chrono::datetime::DateTime;
use chrono::offset::TimeZone;
use chrono::offset::local::Local;
use chrono::offset::LocalResult;
use std::time::Duration;

use format;

use chrono::Datelike;
use chrono::Timelike;
use chrono::Weekday;

pub struct Directory {
    pub root: PathBuf,
}


#[derive(RustcDecodable, RustcEncodable)]
pub struct FileMeta {
    pub name: String,
    pub size: u64,
    pub modified: String,
    pub modified_raw: u64
}


impl Directory {

    pub fn new(root: PathBuf) -> Directory {
        Directory {
            root: root,
        }
    }

    /// Get a table of uri => filename from the root directory. Files are
    /// not listed recursively, only the base level files are listed.
    ///  Directories are ommited as well.
    pub fn list_available_resources(&self) -> Vec<FileMeta> {
        let mut files: Vec<FileMeta> = Vec::new();
        let paths = fs::read_dir(&(self.root)).unwrap();

        for p in paths {
            let pu = p.unwrap();
            if pu.file_type().unwrap().is_file() {
                let date = match pu.metadata().unwrap().modified() {
                    Ok(systime) => {
                        match systime.duration_since(UNIX_EPOCH) {
                            Ok(since_unix) => {
                                let ndt = NaiveDateTime::from_timestamp(
                                    since_unix.as_secs() as i64,
                                    since_unix.subsec_nanos());
                                 match Local.offset_from_local_datetime(&ndt) {
                                    LocalResult::Single(t) => Some(DateTime::from_utc(ndt, t)),
                                    _ => None
                                 }
                            },
                            Err(_) => None
                        }
                    },
                    Err(_) => None
                };
                files.push(
                    FileMeta {
                        name: pu.file_name().into_string().unwrap(),
                        size: pu.metadata().unwrap().len(),
                        modified: match date {
                            Some(d) => format::date_format(&d),
                            None => "n/a".to_string()
                        },
                        modified_raw: pu.metadata().unwrap()
                            .modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs()
                    });
            }
        }
        files
    }

    /// Returns the full path of a file with the name "name". The file
    /// need not be an already existing file.
    pub fn full_path(&self, name: String) -> PathBuf {
        let mut path = PathBuf::new();
        path.push(self.root.to_str().unwrap());
        path.push(name);
        path
    }


    pub fn get_available_name(&self, name: String) -> String {
        let files = self.list_available_resources();
        let mut aname = name.clone();
        let mut num   = 1;
        while self.name_exists(&aname, &files) {
            aname = format!("{} ({})", name, num);
            num += 1;
        }
        return aname;
    }


    pub fn get_resource<'a>(&self, name: &str, files: &'a Vec<FileMeta>)
                        -> Option<&'a FileMeta> {
        for file in files {
            if file.name == name {
                return Some(&file)
            }
        }
        None
    }


    fn name_exists(&self, name: &String, files: &Vec<FileMeta>) -> bool {
        for v in files {
            if v.name == *name {
                return true;
            }
        }
        return false;
    }
}
