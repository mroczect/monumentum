#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::restriction,
    clippy::arithmetic_side_effects
)]
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

mod date;
mod datetime;
mod julianday;
mod strftime;
mod time;
mod timediff;
mod unixepoch;

pub use self::date::DateFunction;
pub use self::datetime::DatetimeFunction;
pub use self::julianday::JuliandayFunction;
pub use self::strftime::StrftimeFunction;
pub use self::time::TimeFunction;
pub use self::timediff::TimediffFunction;
pub use self::unixepoch::UnixepochFunction;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TimeParts {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub fractional: f64,
}

pub(crate) fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub(crate) fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn to_jdn(year: i32, month: u32, day: u32) -> i64 {
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let a = y / 100;
    let b = 2 - a + a / 4;
    let jdn = (36_525_i64 * i64::from(y + 4_716) / 100)
        + (306_001_i64 * i64::from(m + 1) / 10_000)
        + i64::from(day)
        + i64::from(b)
        - 1_524;
    jdn
}
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn from_jdn(jdn: i64) -> (i32, u32, u32) {
    let l = jdn + 68_569;
    let n = 4 * l / 146_097;
    let l = l - (146_097 * n + 3) / 4;
    let i = 4_000 * (l + 1) / 1_461_001;
    let l = l - 1_461 * i / 4 + 31;
    let j = 80 * l / 2_447;
    let day = l - 2_447 * j / 80;
    let l = j / 11;
    let month = j + 2 - 12 * l;
    let year = 100 * (n - 49) + i + l;
    (year as i32, month as u32, day as u32)
}

pub(crate) fn to_julian_day(tp: &TimeParts) -> f64 {
    let jdn = to_jdn(tp.year, tp.month, tp.day) as f64;
    let seconds = f64::from(tp.hour * 3600 + tp.minute * 60 + tp.second) + tp.fractional;
    jdn + (seconds / 86_400.0) - 0.5
}

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn from_julian_day(jd: f64) -> TimeParts {
    let jdn = (jd + 0.5).floor() as i64;
    let frac = (jd + 0.5) - jdn as f64;
    let (year, month, day) = from_jdn(jdn);
    let total_seconds = (frac * 86_400.0).round();
    let second = (total_seconds % 60.0) as u32;
    let total_minutes = (total_seconds / 60.0) as u32;
    let minute = total_minutes % 60;
    let hour = total_minutes / 60;
    TimeParts {
        year,
        month,
        day,
        hour,
        minute,
        second,
        fractional: 0.0,
    }
}

pub(crate) fn parse_time_value(value: &Value, modifiers: &[String]) -> Result<TimeParts, DbError> {
    match value {
        Value::Text(s) => parse_text_time_value(s.as_str()),
        Value::Integer(i) => {
            let x = i.as_i64() as f64;
            if modifiers.first().map(String::as_str) == Some("unixepoch") {
                Ok(from_julian_day(unix_to_jd(x)?))
            } else {
                Ok(from_julian_day(x))
            }
        }
        Value::Float(f) => {
            let x = f.as_f64();
            if modifiers.first().map(String::as_str) == Some("unixepoch") {
                Ok(from_julian_day(unix_to_jd(x)?))
            } else {
                Ok(from_julian_day(x))
            }
        }
        _ => Err(DbError::type_mismatch("invalid time-value")),
    }
}

fn unix_to_jd(unix: f64) -> Result<f64, DbError> {
    let jd = unix / 86_400.0 + 2_440_587.5;
    if jd.is_finite() {
        Ok(jd)
    } else {
        Err(DbError::invalid_operation("unix timestamp out of range"))
    }
}

fn parse_text_time_value(s: &str) -> Result<TimeParts, DbError> {
    if s == "now" {
        return Ok(from_julian_day(2_451_545.0));
    }

    if let Ok(jd) = s.parse::<f64>() {
        return Ok(from_julian_day(jd));
    }

    if let Some(tp) = parse_iso_datetime(s) {
        return Ok(tp);
    }

    if let Some(tp) = parse_time_only(s) {
        return Ok(tp);
    }

    Err(DbError::type_mismatch("invalid time-value format"))
}

fn parse_iso_datetime(s: &str) -> Option<TimeParts> {
    let bytes = s.as_bytes();
    let len = bytes.len();

    if len < 10 {
        return None;
    }

    let year = parse_digits(&s[0..4])? as i32;
    if bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return None;
    }
    let month = parse_digits(&s[5..7])?;
    let day = parse_digits(&s[8..10])?;

    let mut hour = 0;
    let mut minute = 0;
    let mut second = 0;
    let mut fractional = 0.0;

    if len > 10 {
        let sep = *bytes.get(10)?;
        match sep {
            b'T' | b' ' => {
                if len < 16 {
                    return None;
                }
                hour = parse_digits(&s[11..13])?;
                if bytes.get(13) != Some(&b':') {
                    return None;
                }
                minute = parse_digits(&s[14..16])?;
                if len > 16 {
                    if bytes.get(16) != Some(&b':') {
                        return None;
                    }
                    second = parse_digits(&s[17..19])?;
                    if len > 19 && bytes.get(19) == Some(&b'.') {
                        let frac_str = &s[20..];
                        let frac_end = frac_str
                            .char_indices()
                            .find(|(_, c)| !c.is_ascii_digit())
                            .map(|(i, _)| i)
                            .unwrap_or(frac_str.len());
                        let frac_digits = &frac_str[..frac_end];
                        let frac_val = frac_digits.parse::<f64>().ok()?;
                        fractional = frac_val / 10_f64.powi(frac_digits.len() as i32);
                    }
                }
            }
            _ => return None,
        }
    }

    Some(TimeParts {
        year,
        month,
        day,
        hour,
        minute,
        second,
        fractional,
    })
}

fn parse_time_only(s: &str) -> Option<TimeParts> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len < 5 || bytes.get(2) != Some(&b':') {
        return None;
    }
    let hour = parse_digits(&s[0..2])?;
    let minute = parse_digits(&s[3..5])?;
    let mut second = 0;
    let mut fractional = 0.0;
    if len > 5 {
        if bytes.get(5) != Some(&b':') {
            return None;
        }
        second = parse_digits(&s[6..8])?;
        if len > 8 && bytes.get(8) == Some(&b'.') {
            let frac_str = &s[9..];
            let frac_end = frac_str
                .char_indices()
                .find(|(_, c)| !c.is_ascii_digit())
                .map(|(i, _)| i)
                .unwrap_or(frac_str.len());
            let frac_digits = &frac_str[..frac_end];
            let frac_val = frac_digits.parse::<f64>().ok()?;
            fractional = frac_val / 10_f64.powi(frac_digits.len() as i32);
        }
    }
    Some(TimeParts {
        year: 2000,
        month: 1,
        day: 1,
        hour,
        minute,
        second,
        fractional,
    })
}

fn parse_digits(s: &str) -> Option<u32> {
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    s.parse::<u32>().ok()
}

pub(crate) fn apply_modifiers(
    mut tp: TimeParts,
    modifiers: &[String],
) -> Result<TimeParts, DbError> {
    for modifier in modifiers {
        match modifier.as_str() {
            "subsec" | "subsecond" => {}
            "ceiling" | "floor" => {}
            "start of month" => {
                tp.day = 1;
                tp.hour = 0;
                tp.minute = 0;
                tp.second = 0;
                tp.fractional = 0.0;
            }
            "start of year" => {
                tp.month = 1;
                tp.day = 1;
                tp.hour = 0;
                tp.minute = 0;
                tp.second = 0;
                tp.fractional = 0.0;
            }
            "start of day" => {
                tp.hour = 0;
                tp.minute = 0;
                tp.second = 0;
                tp.fractional = 0.0;
            }
            "utc" | "localtime" => {}
            "unixepoch" | "julianday" | "auto" => {}
            _ => {
                if let Some(new_tp) = apply_time_shift(&tp, modifier)? {
                    tp = new_tp;
                } else if let Some(new_tp) = apply_weekday(&tp, modifier)? {
                    tp = new_tp;
                } else if let Some(new_tp) = apply_named_shift(&tp, modifier)? {
                    tp = new_tp;
                } else {
                    return Err(DbError::invalid_operation(format!(
                        "unknown date/time modifier: {modifier}"
                    )));
                }
            }
        }
    }
    Ok(tp)
}

fn apply_time_shift(tp: &TimeParts, modifier: &str) -> Result<Option<TimeParts>, DbError> {
    let parts: Vec<&str> = modifier.split_whitespace().collect();
    if parts.len() < 1 {
        return Ok(None);
    }

    if parts.len() == 2 {
        let num_str = parts[0];
        let unit = parts[1];
        let num = match parse_float(num_str) {
            Some(n) => n,
            None => return Ok(None),
        };
        let new_tp = match unit.trim_end_matches('s') {
            "day" => add_days(tp, num),
            "hour" => add_hours(tp, num),
            "minute" => add_minutes(tp, num),
            "second" => add_seconds(tp, num),
            "month" => add_months(tp, num, false)?,
            "year" => add_years(tp, num, false)?,
            _ => return Ok(None),
        };
        return Ok(Some(new_tp));
    }

    let shift = modifier;
    if shift.starts_with('+') || shift.starts_with('-') {
        let sign = if shift.starts_with('+') { 1.0 } else { -1.0 };
        let body = &shift[1..];
        if body.contains(':') {
            let components: Vec<&str> = body.split(':').collect();
            let hour = parse_float(components.get(0).copied().unwrap_or("0")).unwrap_or(0.0) * sign;
            let minute =
                parse_float(components.get(1).copied().unwrap_or("0")).unwrap_or(0.0) * sign;
            let second = if components.len() > 2 {
                parse_float(components[2]).unwrap_or(0.0) * sign
            } else {
                0.0
            };
            let total_seconds = hour * 3600.0 + minute * 60.0 + second;
            return Ok(Some(add_seconds(tp, total_seconds)));
        } else if body.contains('-') {
            let parts: Vec<&str> = body.split('-').collect();
            if parts.len() != 3 {
                return Ok(None);
            }
            let year = parse_float(parts[0]).unwrap_or(0.0) * sign;
            let month = parse_float(parts[1]).unwrap_or(0.0) * sign;
            let day = parse_float(parts[2]).unwrap_or(0.0) * sign;
            let mut new_tp = *tp;
            if year != 0.0 {
                new_tp = add_years(&new_tp, year, false)?;
            }
            if month != 0.0 {
                new_tp = add_months(&new_tp, month, false)?;
            }
            if day != 0.0 {
                new_tp = add_days(&new_tp, day);
            }
            return Ok(Some(new_tp));
        }
    }

    Ok(None)
}

fn apply_weekday(tp: &TimeParts, modifier: &str) -> Result<Option<TimeParts>, DbError> {
    let parts: Vec<&str> = modifier.split_whitespace().collect();
    if parts.len() == 2 && parts[0] == "weekday" {
        let target = parse_digits(parts[1])
            .ok_or_else(|| DbError::invalid_operation("weekday modifier requires a number"))?;
        if target > 6 {
            return Err(DbError::invalid_operation("weekday number must be 0-6"));
        }
        let jdn = to_jdn(tp.year, tp.month, tp.day);
        let current_weekday = ((jdn + 1) % 7 + 7) % 7;
        let diff = (i64::from(target) - current_weekday + 7) % 7;
        if diff != 0 {
            let new_tp = add_days(tp, diff as f64);
            return Ok(Some(new_tp));
        }
        return Ok(Some(*tp));
    }
    Ok(None)
}

fn apply_named_shift(_tp: &TimeParts, _modifier: &str) -> Result<Option<TimeParts>, DbError> {
    Ok(None)
}

fn add_seconds(tp: &TimeParts, seconds: f64) -> TimeParts {
    let total = to_julian_day(tp) + seconds / 86_400.0;
    from_julian_day(total)
}

fn add_minutes(tp: &TimeParts, minutes: f64) -> TimeParts {
    add_seconds(tp, minutes * 60.0)
}

fn add_hours(tp: &TimeParts, hours: f64) -> TimeParts {
    add_minutes(tp, hours * 60.0)
}

fn add_days(tp: &TimeParts, days: f64) -> TimeParts {
    add_seconds(tp, days * 86_400.0)
}

#[allow(clippy::cast_possible_truncation)]
fn add_months(tp: &TimeParts, months: f64, _use_floor: bool) -> Result<TimeParts, DbError> {
    let total_months = months.round() as i32;
    if total_months.abs() > 1_000_000 {
        return Err(DbError::invalid_operation("month shift too large"));
    }
    let total = i32::try_from(tp.year).unwrap_or(0) * 12 + i32::try_from(tp.month).unwrap_or(0) - 1
        + total_months;
    let new_year = total.div_euclid(12);
    let new_month = total.rem_euclid(12) + 1;
    let max_day = days_in_month(new_year, new_month as u32);
    let new_day = tp.day.min(max_day);
    Ok(TimeParts {
        year: new_year,
        month: new_month as u32,
        day: new_day,
        hour: tp.hour,
        minute: tp.minute,
        second: tp.second,
        fractional: tp.fractional,
    })
}

#[allow(clippy::cast_possible_truncation)]
fn add_years(tp: &TimeParts, years: f64, _use_floor: bool) -> Result<TimeParts, DbError> {
    let total_years = years.round() as i32;
    if total_years.abs() > 1_000_000 {
        return Err(DbError::invalid_operation("year shift too large"));
    }
    let new_year = tp.year + total_years;
    let max_day = days_in_month(new_year, tp.month);
    let new_day = tp.day.min(max_day);
    Ok(TimeParts {
        year: new_year,
        month: tp.month,
        day: new_day,
        hour: tp.hour,
        minute: tp.minute,
        second: tp.second,
        fractional: tp.fractional,
    })
}

fn parse_float(s: &str) -> Option<f64> {
    s.parse::<f64>().ok()
}

pub(crate) fn format_date(tp: &TimeParts) -> String {
    let mut out = String::new();
    let _ = write!(out, "{:04}-{:02}-{:02}", tp.year, tp.month, tp.day);
    out
}

pub(crate) fn format_time(tp: &TimeParts, subsec: bool) -> String {
    let mut out = String::new();
    if subsec {
        let _ = write!(
            out,
            "{:02}:{:02}:{:06.3}",
            tp.hour,
            tp.minute,
            f64::from(tp.second) + tp.fractional
        );
    } else {
        let _ = write!(out, "{:02}:{:02}:{:02}", tp.hour, tp.minute, tp.second);
    }
    out
}

pub(crate) fn format_datetime(tp: &TimeParts, subsec: bool) -> String {
    let date = format_date(tp);
    let time = format_time(tp, subsec);
    let mut out = String::with_capacity(date.len() + 1 + time.len());
    out.push_str(&date);
    out.push(' ');
    out.push_str(&time);
    out
}

pub(crate) fn format_julianday(tp: &TimeParts) -> f64 {
    to_julian_day(tp)
}

pub(crate) fn format_unixepoch(tp: &TimeParts, subsec: bool) -> f64 {
    let jd = to_julian_day(tp);
    let unix = (jd - 2_440_587.5) * 86_400.0;
    if subsec { unix } else { unix.floor() }
}

pub(crate) fn strftime(format: &str, tp: &TimeParts) -> Option<String> {
    let mut out = String::new();
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let next = chars.next()?;
        match next {
            '%' => out.push('%'),
            'd' => {
                let _ = write!(out, "{:02}", tp.day);
            }
            'e' => {
                let _ = write!(out, "{}", tp.day);
            }
            'f' => {
                let _ = write!(out, "{:06.3}", f64::from(tp.second) + tp.fractional);
            }
            'F' => out.push_str(&format_date(tp)),
            'G' | 'g' => {
                if next == 'G' {
                    let _ = write!(out, "{:04}", tp.year);
                } else {
                    let _ = write!(out, "{:02}", tp.year % 100);
                }
            }
            'H' => {
                let _ = write!(out, "{:02}", tp.hour);
            }
            'I' => {
                let h = if tp.hour % 12 == 0 { 12 } else { tp.hour % 12 };
                let _ = write!(out, "{h:02}");
            }
            'j' => {
                let mut day_of_year = 0;
                for m in 1..tp.month {
                    day_of_year += days_in_month(tp.year, m);
                }
                day_of_year += tp.day;
                let _ = write!(out, "{day_of_year:03}");
            }
            'J' => {
                let _ = write!(out, "{}", to_julian_day(tp));
            }
            'k' => {
                let _ = write!(out, "{}", tp.hour);
            }
            'l' => {
                let h = if tp.hour % 12 == 0 { 12 } else { tp.hour % 12 };
                let _ = write!(out, "{h}");
            }
            'm' => {
                let _ = write!(out, "{:02}", tp.month);
            }
            'M' => {
                let _ = write!(out, "{:02}", tp.minute);
            }
            'p' => {
                let ampm = if tp.hour < 12 { "AM" } else { "PM" };
                out.push_str(ampm);
            }
            'P' => {
                let ampm = if tp.hour < 12 { "am" } else { "pm" };
                out.push_str(ampm);
            }
            'R' => {
                let _ = write!(out, "{:02}:{:02}", tp.hour, tp.minute);
            }
            's' => {
                let _ = write!(out, "{}", format_unixepoch(tp, false));
            }
            'S' => {
                let _ = write!(out, "{:02}", tp.second);
            }
            'T' => out.push_str(&format_time(tp, false)),
            'U' | 'W' => {
                let jdn = to_jdn(tp.year, tp.month, tp.day);
                let jan1_jdn = to_jdn(tp.year, 1, 1);
                let days = jdn - jan1_jdn;
                let week = (days + if next == 'U' { 6 } else { 0 }) / 7;
                let _ = write!(out, "{week:02}");
            }
            'u' => {
                let jdn = to_jdn(tp.year, tp.month, tp.day);
                let weekday = ((jdn + 1) % 7 + 7) % 7;
                let iso_weekday = if weekday == 0 { 7 } else { weekday };
                let _ = write!(out, "{iso_weekday}");
            }
            'V' => {
                let jdn = to_jdn(tp.year, tp.month, tp.day);
                let jan1_jdn = to_jdn(tp.year, 1, 1);
                let days = jdn - jan1_jdn;
                let week = (days + 3) / 7;
                let _ = write!(out, "{week:02}");
            }
            'w' => {
                let jdn = to_jdn(tp.year, tp.month, tp.day);
                let weekday = ((jdn + 1) % 7 + 7) % 7;
                let _ = write!(out, "{weekday}");
            }
            'Y' => {
                let _ = write!(out, "{:04}", tp.year);
            }
            _ => return None,
        }
    }
    Some(out)
}

pub(crate) fn split_args(args: &[Value]) -> Result<(Value, Vec<String>), DbError> {
    let value = if args.is_empty() {
        Value::try_from("now")?
    } else {
        args[0].clone()
    };
    let mut modifiers = Vec::new();
    for arg in args.iter().skip(1) {
        if let Value::Text(s) = arg {
            modifiers.push(s.as_str().to_string());
        } else {
            return Err(DbError::type_mismatch("modifier must be text"));
        }
    }
    Ok((value, modifiers))
}
