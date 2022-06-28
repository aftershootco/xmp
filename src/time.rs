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

pub fn timestamp_offset(datestr: impl AsRef<str>) -> Option<(i64, Option<i64>)> {
    let datetime = iso8601::datetime(datestr.as_ref()).ok()?;
    let offset_seconds =
        datetime.time.tz_offset_hours * 3600 + datetime.time.tz_offset_minutes * 60;

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
    )? - chrono::Duration::seconds(offset_seconds.into());
    let offset = chrono::FixedOffset::east_opt(offset_seconds)?;

    let finaltime: chrono::DateTime<chrono::FixedOffset> =
        chrono::DateTime::<chrono::Utc>::from_utc(ndt, chrono::Utc).with_timezone(&offset);

    if offset_seconds != 0 {
        Some((finaltime.timestamp(), Some(offset_seconds.into())))
    } else {
        Some((finaltime.timestamp(), None))
    }
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

#[test]
pub fn rfc3999_localtime() {
    let t = chrono::Local::now();
    let t_string = t.to_rfc3339();
    let timestamp = timestamp(t_string).unwrap();
    assert_eq!(t.timestamp(), timestamp);
}

#[test]
pub fn rfc3999_utctime() {
    let t = chrono::Utc::now();
    let t_string = t.to_rfc3339();
    let timestamp = timestamp(t_string).unwrap();
    assert_eq!(t.timestamp(), timestamp);
}

#[test]
pub fn timezone_plus() {
    let sometime = "2022-04-30T11:37:36.93+04:00";
    let t1 = chrono::DateTime::parse_from_rfc3339(sometime)
        .unwrap()
        .timestamp();
    let t2 = timestamp(sometime).unwrap();
    assert_eq!(t1, t2)
}

#[test]
pub fn timezone_minus() {
    let sometime = "2022-04-30T11:37:36.93-04:00";
    let t1 = chrono::DateTime::parse_from_rfc3339(sometime)
        .unwrap()
        .timestamp();
    let t2 = timestamp(sometime).unwrap();
    assert_eq!(t1, t2)
}

#[test]
pub fn timezone() {
    let sometime = "2020-05-18T14:57:37.87-04:00";
    let t1 = chrono::DateTime::parse_from_rfc3339(sometime)
        .unwrap()
        .timestamp();
    let t2 = timestamp_offset(sometime).unwrap();
    assert_eq!(t1, t2.0);
    assert_eq!(t2.1.unwrap(), -4 * 3600);

    let sometime = "2020-05-18T14:57:37.87-05:15";
    let t1 = chrono::DateTime::parse_from_rfc3339(sometime)
        .unwrap()
        .timestamp();
    let t2 = timestamp_offset(sometime).unwrap();
    assert_eq!(t1, t2.0);
    assert_eq!(t2.1.unwrap(), -(5 * 3600 + 15 * 60));

    let sometime = "2020-05-18T14:57:37.87+05:30";
    let t1 = chrono::DateTime::parse_from_rfc3339(sometime)
        .unwrap()
        .timestamp();
    let t2 = timestamp_offset(sometime).unwrap();
    assert_eq!(t1, t2.0);
    assert_eq!(t2.1.unwrap(), 5 * 3600 + 30 * 60);
}
