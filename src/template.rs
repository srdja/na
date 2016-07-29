/*
 * na
 *
 * Copyright (C) 2016 Srđan Panić <i@srdja.me>
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


use std::collections::HashMap;
use std::string::String;
use mustache::{self, VecBuilder, MapBuilder};
use directory::FileMeta;
use chrono::Datelike;
use chrono::Timelike;
use chrono::datetime::DateTime;
use chrono::offset::local::Local;
use chrono::Weekday;


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


fn weekday_to_string(wd: Weekday) -> String {
    match wd {
        Weekday::Mon => "Mon".to_string(),
        Weekday::Tue => "Tue".to_string(),
        Weekday::Wed => "Wed".to_string(),
        Weekday::Thu => "Thu".to_string(),
        Weekday::Fri => "Fri".to_string(),
        Weekday::Sat => "Sat".to_string(),
        Weekday::Sun => "Sun".to_string()
    }
}


fn month_to_string(month: u32) -> String {
    match month {
        1 => "Jan".to_string(),
        2 => "Feb".to_string(),
        3 => "Mar".to_string(),
        4 => "Apr".to_string(),
        5 => "May".to_string(),
        6 => "Jun".to_string(),
        7 => "Jul".to_string(),
        8 => "Aug".to_string(),
        9 => "Sep".to_string(),
        10 => "Oct".to_string(),
        11 => "Nov".to_string(),
        12 => "Dec".to_string(),
        _ => "--".to_string()
    }
}


fn date_format(date: &DateTime<Local>) -> String {
    let hour = if date.hour()   > 9 {format!("{}", date.hour())}   else {format!("0{}", date.hour())};
    let minute = if date.minute() > 9 {format!("{}", date.minute())} else {format!("0{}", date.minute())};
    let second = if date.second() > 9 {format!("{}", date.second())} else {format!("0{}", date.second())};
    let day = if date.day() > 9 {format!("{}", date.day())} else {format!("0{}", date.day())};
    let wd = weekday_to_string(date.weekday());
    let month = month_to_string(date.month());
    format!("{}, {} {} {}  {}:{}:{}", wd, month, day, date.year(), hour, minute, second)
}



pub fn render_html(template: String, res: &HashMap<String, FileMeta>) -> String {
    let root = MapBuilder::new().insert_vec("files", |_| {
        let mut data = VecBuilder::new();
        for (uri, name) in res {
            data = data.push_map(|builder| {
                builder
                    .insert_str("url", format!("/files/{}", uri))
                    .insert_str("name".to_string(), name.name.clone())
                    .insert_str("size".to_string(), format_size(name.size))
                    .insert_str("size-bytes".to_string(), format!("{}", name.size))
                    .insert_str("modified".to_string(),
                                match name.modified {
                                    Some(d) => date_format(&d),
                                    None => "n/a".to_string()
                                })
            });
        }
        data
    }).build();

    let mut buff: Vec<u8> = Vec::new();
    let template = mustache::compile_str(template.as_str());
    template.render_data(&mut buff, &root);

    String::from_utf8(buff).unwrap()
}
