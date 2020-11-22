use std::convert::TryFrom;

pub enum Month{
    Jan,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl TryFrom<u32> for Month {
    type Error = ();

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == Month::Jan as u32 => Ok(Month::Jan),
            x if x == Month::Feb as u32 => Ok(Month::Feb),
            x if x == Month::Mar as u32 => Ok(Month::Mar),
            x if x == Month::Apr as u32 => Ok(Month::Apr),
            x if x == Month::May as u32 => Ok(Month::May),
            x if x == Month::Jun as u32 => Ok(Month::Jun),
            x if x == Month::Jul as u32 => Ok(Month::Jul),
            x if x == Month::Aug as u32 => Ok(Month::Aug),
            x if x == Month::Sep as u32 => Ok(Month::Sep),
            x if x == Month::Oct as u32 => Ok(Month::Oct),
            x if x == Month::Nov as u32 => Ok(Month::Nov),
            x if x == Month::Dec as u32 => Ok(Month::Dec),
            _ => Err(()),
        }
    }
}


pub fn month(month: Month) -> String {
    match month {
        Month::Jan => "Jan",
        Month::Feb => "Feb",
        Month::Mar => "Mar",
        Month::Apr => "Apr",
        Month::May => "May",
        Month::Jun=> "Jun",
        Month::Jul=> "Jul",
        Month::Aug=> "Aug",
        Month::Sep => "Sep",
        Month::Oct=> "Oct",
        Month::Nov=> "Nov",
        Month::Dec=> "Dec",
    }.to_string()
}