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

use chrono::datetime::DateTime;
use chrono::offset::local::Local;
use chrono::Datelike;
use chrono::Timelike;
use chrono::Weekday;
use directory::FileMeta;
use hyper::method::Method;
use month::{month, Month};
use mustache::{self, MapBuilder, VecBuilder};
use std::convert::TryFrom;
use std::string::String;

fn size(bytes: u64) -> String {
    return match bytes {
        b if b < 1000 => {
            format!("{} b", b)
        }
        b @ 1000..=999999 => {
            let kb = b / 1000;
            let rm = (b % 1000) / 10;
            format!("{}.{} Kb", kb, rm)
        }
        b @ 1000000..=999999999 => {
            let mb = b / 1000000;
            let rm = (b % 1000000) / 10000;
            format!("{}.{} Mb", mb, rm)
        }
        b if b > 999999999 => {
            let gb = b / 1000000000;
            let rm = (b % 1000000000) / 10000000;
            format!("{}.{} Gb", gb, rm)
        }
        _ => {
            format!("n/a")
        }
    };
}

pub fn weekday(weekday: Weekday) -> String {
    match weekday {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
    .to_string()
}

pub fn date(date: &DateTime<Local>) -> String {
    format!(
        "{}, {:02} {:02} {}  {:02}:{:02}:{:02}",
        weekday(date.weekday()),
        month(Month::try_from(date.month()).expect("Could not parse date.month u32 into Month?")),
        date.day(),
        date.year(),
        date.hour(),
        date.minute(),
        date.second()
    )
}

pub fn html(
    template: &str,
    res: &Vec<FileMeta>,
    del: bool,
    show: bool,
    no_upload: bool,
    dir: String,
) -> String {
    let root = MapBuilder::new()
        .insert_vec("files", |_| {
            let mut data = VecBuilder::new();
            for name in res {
                data = data.push_map(|builder| {
                    builder
                        .insert_str("url", format!("/files/{}", name.name))
                        .insert_str("name", name.name.clone())
                        .insert_str("size", size(name.size))
                        .insert_bool("delete", del)
                        .insert_str("dir", "bla")
                        .insert_str("size-bytes", format!("{}", name.size))
                        .insert_str("time", format!("{}", name.modified_raw))
                        .insert_str("modified", name.modified.clone())
                });
            }
            data
        })
        .insert_map("header", |_| {
            MapBuilder::new()
                .insert_bool("showdir", show)
                .insert_bool("delete", del)
                .insert_str("dir", dir.clone())
        })
        .insert_bool("upload", !no_upload)
        .build();

    let mut buff: Vec<u8> = Vec::new();
    let template = mustache::compile_str(template).expect("could not compile str");
    template.render_data(&mut buff, &root);

    String::from_utf8(buff).unwrap()
}

pub fn file_list(res: &Vec<FileMeta>) -> String {
    let mut response = String::new();
    for meta in res {
        response.push_str(&format!("/files/{}\n", meta.name.clone()));
    }
    response
}
