use chrono::NaiveDate;
use iso8601::Date;
pub fn timestamp(datestr: impl AsRef<str>) -> Option<i64> {
    let datetime = iso8601::datetime(datestr.as_ref()).ok()?;
    let ndt = match datetime.date {
        Date::YMD { year, month, day } => NaiveDate::from_ymd_opt(year, month, day)?,
        Date::Week { year, ww, d } => NaiveDate::from_isoywd_opt(year, ww, isoweekday(d)?)?,
        Date::Ordinal { year, ddd } => NaiveDate::from_yo_opt(year, ddd)?,
    }
    .and_hms_milli_opt(
        datetime.time.hour,
        datetime.time.minute,
        datetime.time.second,
        datetime.time.millisecond,
    )? - chrono::Duration::seconds(
        (datetime.time.tz_offset_hours * 3600 + datetime.time.tz_offset_minutes * 60).into(),
    );
    let offset = chrono::FixedOffset::east_opt(
        datetime.time.tz_offset_hours * 3600 + datetime.time.tz_offset_minutes * 60,
    )?;

    let datetime: chrono::DateTime<chrono::FixedOffset> =
        chrono::DateTime::<chrono::Utc>::from_utc(ndt, chrono::Utc).with_timezone(&offset);

    Some(datetime.timestamp())
}

pub fn isoweekday(d: u32) -> Option<chrono::Weekday> {
    use chrono::Weekday::*;
    Some(match d {
        1 => Mon,
        2 => Tue,
        3 => Wed,
        4 => Thu,
        5 => Fri,
        6 => Sat,
        7 => Sun,
        _ => None?,
    })
}
