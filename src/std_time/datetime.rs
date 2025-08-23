use std::ops::Deref;

use crate::{prelude::*, std_time::duration::TimeDuration};
use mluau::prelude::*;
use jiff::Zoned;

#[derive(Clone)]
pub struct DateTime {
    inner: Zoned,
}

use super::TimeSpan;

impl DateTime {
    pub fn from(zoned: Zoned) -> Self {
        Self {
            inner: zoned,
        }
    }
    pub fn now() -> Self {
        Self {
            inner: Zoned::now(),
        }
    }
    fn get_format_string(format_string: &str) -> &str {
        match format_string {
            "ISO_8601" => "%Y-%m-%d %H:%M",
            "RFC_2822" => "%a, %d %b %Y %H:%M:%S %z",
            "RFC_3339" => "%Y-%m-%dT%H:%M:%S%:z",
            "SHORT_DATE" => "%Y-%m-%d",
            "SHORT_TIME" => "%H:%M",
            "FULL_DATE_TIME" => "%A, %B %d, %Y %H:%M:%S",
            "LOGGING_12_HR" => "%a %b %e %I:%M:%S %p %Z %Y",
            "LOGGING_24_HR" => "%a %b %e %H:%M:%S %Z %Y",
            "MM/DD/YYYY" => "%m/%d/%Y",
            "MM/DD/YYYY HH:MM (AM/PM)" => "%m/%d/%Y %I:%M %p",
            "MM/DD/YY" => "%m/%d/%y",
            "HH:MM (AM/PM)" => "%I:%M %p",
            "AMERICAN_FULL_DATE_TIME" => "%A, %B %d, %Y %I:%M:%S %p",
            other => other,
        }
    }
    pub fn parse(source: &mut String, format_string: &str, iana_timezone: &str, function_name: &'static str) -> LuaResult<Self> {
        let mut format_string = Self::get_format_string(format_string).to_string();

        if !format_string.contains("%Q") {
            format_string.push_str(" %Q");
            source.push(' ');
            source.push_str(iana_timezone);
        }

        match Zoned::strptime(&format_string, &source) {
            Ok(zoned) => Ok(Self { inner: zoned }),
            Err(err) => wrap_err!(
                "{}: failed to parse source '{}' with format '{}' due to err: {}",
                function_name,
                source,
                format_string,
                err
            ),
        }
    }
    pub fn format(&self, format_string: &str, function_name: &'static str) -> LuaResult<String> {
        let format_string = Self::get_format_string(format_string);
        match jiff::fmt::strtime::format(format_string, &self.inner) {
            Ok(s) => Ok(s),
            Err(err) => {
                wrap_err!("{} unable to format DateTime because {}", function_name, err)
            }
        }
    }
    pub fn get_userdata(self, luau: &Lua) -> LuaValueResult {
        ok_userdata(self, luau)
    }
    pub fn date(&self) -> jiff::civil::Date {
        self.inner.date()
    }
}

fn datetime_now(luau: &Lua, _: ()) -> LuaValueResult {
    ok_userdata(DateTime::now(), luau)
}

fn datetime_parse(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "datetime.parse(source: string, format: string)";
    let mut source = match multivalue.pop_front() {
        Some(LuaValue::String(s)) => match s.to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => {
                return wrap_err!("{}: source string was unexpectedly invalid utf-8", function_name);
            }
        },
        Some(LuaNil) | None => {
            return wrap_err!("{} expected source to be a datetime-formattable string, but was incorrectly called with zero arguments or nil", function_name);
        },
        Some(other) => {
            return wrap_err!("{} expected source to be a datetime-formattable string, got: {:?}", function_name, other);
        }
    };
    let format_string = match multivalue.pop_front() {
        Some(LuaValue::String(s)) => s.to_string_lossy(),
        Some(LuaNil) | None => {
            return wrap_err!("{} expected format to be a common datetime format or valid datetime formatting string, but was incorrectly called with zero arguments or nil", function_name);
        },
        Some(other) => {
            return wrap_err!("{} expected format to be a common datetime format or valid datetime formatting string, got: {:?}", function_name, other);
        }
    };
    let iana_timezone = match multivalue.pop_front() {
        Some(LuaValue::String(s)) => s.to_string_lossy(),
        Some(LuaNil) | None => String::from("UTC"),
        Some(other) => {
            return wrap_err!("{} expected the timezone to be one of the 500+ IANA timezones or nil (defaults to UTC), got: {:?}", function_name, other);
        }
    };

    ok_userdata(DateTime::parse(&mut source, &format_string,  &iana_timezone, function_name)?, luau)
}

impl LuaUserData for DateTime {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("year", |_: &Lua, this: &DateTime| Ok(this.inner.date().year()));
        fields.add_field_method_get("month", |_: &Lua, this: &DateTime| Ok(this.inner.date().month()));
        fields.add_field_method_get("day", |_: &Lua, this: &DateTime| Ok(this.inner.date().day()));
        fields.add_field_method_get("hour", |_: &Lua, this: &DateTime| Ok(this.inner.time().hour()));
        fields.add_field_method_get("minute", |_: &Lua, this: &DateTime| Ok(this.inner.time().minute()));
        fields.add_field_method_get("second", |_: &Lua, this: &DateTime| Ok(this.inner.time().second()));
        fields.add_field_method_get("millisecond", |_: &Lua, this: &DateTime| Ok(this.inner.time().millisecond()));
        fields.add_field_method_get("weekday", |_: &Lua, this: &DateTime| {
            Ok(this.inner.strftime("%A").to_string()) // like 'Monday'
        });
        fields.add_field_method_get("unix_timestamp", |_: &Lua, this: &DateTime| {
            Ok(this.inner.timestamp().as_second())
        });
        fields.add_field_method_get("timezone", |_: &Lua, this: &DateTime| {
            let timezone = match this.inner.time_zone().iana_name() {
                Some(tz) => tz.as_bytes().to_owned(),
                None => Vec::<u8>::default(),
            };
            Ok(timezone)
        });
        fields.add_field_method_get("iso", |_: &Lua, this: &DateTime| {
            Ok(this.inner.to_string())
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, | luau: &Lua, this: &DateTime, _: LuaValue| -> LuaValueResult {
            ok_string(format!("DateTime<{}>", this.inner.strftime("%Y-%m-%d %H:%M:%S (%I:%M %p) [%Z]")), luau)
        });
        methods.add_method("display", | luau: &Lua, this: &DateTime, _value: ()| {
            // :display() shows format like 2025-08-18 18:20:44 (6:20 PM) [CDT]
            ok_string(format!("{}", this.inner.strftime("%Y-%m-%d %H:%M:%S (%I:%M %p) [%Z]")), luau)
        });
        methods.add_method("__dp", | luau: &Lua, this: &DateTime, _value: ()| {
            // shows the entire roundtrippable DateTime<2025-08-18T20:23:29.85205845-05:00[America/Chicago]>
            ok_string(format!("DateTime<{:?}>", this.inner), luau)
        });

        // methods.add_method("add_months", |luau: &Lua, this: &DateTime, value: LuaValue| -> LuaValueResult {
        //     let months = match value {
        //         LuaValue::Integer(i) => i,
        //         LuaValue::Number(n) => n as i64,
        //         other => todo!()
        //     };
        //     let new = this.inner.saturating_add(months.months());
        //     DateTime::from(new).get_userdata(luau)
        // });

        methods.add_method("format", |luau: &Lua, this: &DateTime, value: LuaValue| -> LuaValueResult { 
            let function_name = "DateTime:format(format: string)";
            let format_string = match value {
                LuaValue::String(s) => s.to_string_lossy(),
                other => {
                    return wrap_err!("{} expected format to be a string, got: {:?}", function_name, other);
                }
            };
            ok_string(this.format(&format_string, function_name)?, luau)
        });


        methods.add_method("timespan", |luau: &Lua, this: &DateTime, other: LuaValue| -> LuaValueResult {
            let function_name = "DateTime:timespan(other: DateTime)";
            match other {
                LuaValue::UserData(ud) => {
                    if ud.is::<DateTime>() {
                        let other_dt = ud.borrow::<DateTime>().expect("no way not DateTime");
                        let span = match this.inner.until(&other_dt.deref().inner) {
                            Ok(span) => span,
                            Err(err) => {
                                return wrap_err!("{} unable to compute timespan due to err: {}", function_name, err);
                            }
                        };
                        TimeSpan::new(span).get_userdata(luau)
                    } else {
                        wrap_err!("{} expected other to be a DateTime, got: {:?}", function_name, ud.type_name()?)
                    }
                },
                other => {
                    wrap_err!("{} expected other to be a DateTime, got: {:?}", function_name, other)
                }
            }
        });

        methods.add_method("since", |luau: &Lua, this: &DateTime, other: LuaValue| -> LuaValueResult {
            let function_name = "DateTime:since(other: DateTime)";
            match other {
                LuaValue::UserData(ud) => {
                    if ud.is::<DateTime>() {
                        let other_dt = ud.borrow::<DateTime>().expect("no way not DateTime");
                        let span = match this.inner.since(&other_dt.deref().inner) {
                            Ok(span) => span,
                            Err(err) => {
                                return wrap_err!("{} unable to compute timespan due to err: {}", function_name, err);
                            }
                        };
                        TimeSpan::new(span).get_userdata(luau)
                    } else {
                        wrap_err!("{} expected other to be a DateTime, got: {:?}", function_name, ud.type_name()?)
                    }
                },
                other => {
                    wrap_err!("{} expected other to be a DateTime, got: {:?}", function_name, other)
                }
            }
        });

        // DateTime + TimeSpan -> DateTime
        methods.add_meta_method(LuaMetaMethod::Add, |luau: &Lua, this: &DateTime, other: LuaValue| -> LuaValueResult {
            let function_name = "DateTime.__add(self, other: TimeSpan)";
            match other {
                LuaValue::UserData(ud) => {
                    if ud.is::<TimeSpan>() {
                        let other_timespan = ud.borrow::<TimeSpan>().expect("impossible not TimeSpan");
                        let new_dt = &this.inner + other_timespan.deref().inner;
                        DateTime::from(new_dt).get_userdata(luau)
                    } else if ud.is::<DateTime>() {
                        // let other_dt = ud.borrow::<DateTime>().expect("impossible not DateTime");
                        wrap_err!("{}: adding DateTime + DateTime makes no sense and is ambiguous, did you mean to add a TimeSpan?", function_name)
                    } else if ud.is::<TimeDuration>() {
                        wrap_err!("{}: unfortunately we can't add DateTime + Duration directly, you need to use a TimeSpan (time.datetime.days(n))")
                    } else {
                        wrap_err!("{}: other must be a TimeSpan", function_name)
                    }
                },
                other => {
                    wrap_err!("{} expected other to be a TimeSpan, got: {:?}", function_name, other)
                }
            }
        });

        // DateTime - TimeSpan -> DateTime;
        methods.add_meta_method(LuaMetaMethod::Sub, |luau: &Lua, this: &DateTime, other: LuaValue| -> LuaValueResult {
            let function_name = "DateTime.__sub(self, other: TimeSpan)";
            match other {
                LuaValue::UserData(ud) => {
                    if ud.is::<TimeSpan>() {
                        let other_timespan = ud.borrow::<TimeSpan>().expect("impossible not TimeSpan");
                        let new_dt = &this.inner - other_timespan.deref().inner;
                        DateTime::from(new_dt).get_userdata(luau)
                    } else if ud.is::<DateTime>() {
                        // let other_dt = ud.borrow::<DateTime>().expect("impossible not DateTime");
                        wrap_err!("{}: adding DateTime - DateTime is ambiguous; if you need a TimeSpan between two DateTimes, use DateTime:since(dt: DateTime)", function_name)
                    } else if ud.is::<TimeDuration>() {
                        wrap_err!("{}: unfortunately we can't add DateTime + Duration directly, you need to use a TimeSpan (time.datetime.days(n))")
                    } else {
                        wrap_err!("{}: other must be a TimeSpan", function_name)
                    }
                },
                other => {
                    wrap_err!("{} expected other to be a TimeSpan, got: {:?}", function_name, other)
                }
            }
        });

        methods.add_meta_method(LuaMetaMethod::Eq, |_luau: &Lua, this: &DateTime, other: LuaValue| -> LuaValueResult {
            let function_name = "DateTime.__eq(self, other: DateTime)";
            match other {
                LuaValue::UserData(ud) => {
                    if ud.is::<DateTime>() {
                        let other_dt = ud.borrow::<DateTime>().expect("impossible not DateTime");
                        Ok(LuaValue::Boolean(this.inner == other_dt.deref().inner))
                    } else if ud.is::<TimeSpan>() {
                        wrap_err!("{}: DateTime == TimeSpan makes no sense to me lol", function_name)
                    } else {
                        wrap_err!("{}: other must be a DateTime", function_name)
                    }
                },
                other => {
                    wrap_err!("{} expected other to be a DateTime, got: {:?}", function_name, other)
                }
            }
        });

        methods.add_meta_method(LuaMetaMethod::Lt, |_luau: &Lua, this: &DateTime, other: LuaValue| -> LuaValueResult {
            let function_name = "DateTime.__lt(self, other: DateTime)";
            match other {
                LuaValue::UserData(ud) => {
                    if ud.is::<DateTime>() {
                        let other_dt = ud.borrow::<DateTime>().expect("impossible not DateTime");
                        Ok(LuaValue::Boolean(this.inner < other_dt.deref().inner))
                    } else if ud.is::<TimeSpan>() {
                        wrap_err!("{}: DateTime < TimeSpan makes no sense to me lol", function_name)
                    } else {
                        wrap_err!("{}: other must be a DateTime", function_name)
                    }
                },
                other => {
                    wrap_err!("{} expected other to be a DateTime, got: {:?}", function_name, other)
                }
            }
        });

        methods.add_meta_method(LuaMetaMethod::Le, |_luau: &Lua, this: &DateTime, other: LuaValue| -> LuaValueResult {
            let function_name = "DateTime.__le(self, other: DateTime)";
            match other {
                LuaValue::UserData(ud) => {
                    if ud.is::<DateTime>() {
                        let other_dt = ud.borrow::<DateTime>().expect("impossible not DateTime");
                        Ok(LuaValue::Boolean(this.inner <= other_dt.deref().inner))
                    } else if ud.is::<TimeSpan>() {
                        wrap_err!("{}: DateTime <= TimeSpan makes no sense to me lol", function_name)
                    } else {
                        wrap_err!("{}: other must be a DateTime", function_name)
                    }
                },
                other => {
                    wrap_err!("{} expected other to be a DateTime, got: {:?}", function_name, other)
                }
            }
        });

        // Mon, 18 Aug 2025 20:54:00 -0500
        methods.add_method("rfc_2822", |luau: &Lua, this: &DateTime, _value: ()| {
            ok_string(this.inner.strftime("%a, %d %b %Y %H:%M:%S %z").to_string(), luau)
        });

        // 2025-08-18T20:54:00-05:00
        methods.add_method("rfc_3339", |luau: &Lua, this: &DateTime, _value: ()| {
            ok_string(this.inner.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string(), luau)
        });

        // 2025-08-18T20:54:00.000-05:00
        methods.add_method("rfc_3339_opts", |luau: &Lua, this: &DateTime, _value: ()| {
            ok_string(this.inner.strftime("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string(), luau)
        });

        // 2025-08-18T20:54:00-05:00
        methods.add_method("iso_8601", |luau: &Lua, this: &DateTime, _value: ()| {
            ok_string(this.inner.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string(), luau)
        });

        // Mon, 18 Aug 2025 20:54:00 GMT
        methods.add_method("http_date", |luau: &Lua, this: &DateTime, _value: ()| {
            ok_string(this.inner.strftime("%a, %d %b %Y %H:%M:%S GMT").to_string(), luau)
        });

    }
}


pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("now", datetime_now)?
        .with_function("parse", datetime_parse)?
        .with_value("COMMON_FORMATS", TableBuilder::create(luau)?
            .with_value("ISO_8601", "%Y-%m-%d %H:%M")?
            .with_value("RFC_2822", "%a, %d %b %Y %H:%M:%S %z")?
            .with_value("RFC_3339", "%Y-%m-%dT%H:%M:%S%:z")?
            .with_value("SHORT_DATE", "%Y-%m-%d")?
            .with_value("SHORT_TIME", "%H:%M")?
            .with_value("FULL_DATE_TIME", "%A, %B %d, %Y %H:%M:%S")?
            .with_value("LOGGING_12_HR", "%a %b %e %I:%M:%S %p %Z %Y")?
            .with_value("LOGGING_24_HR", "%a %b %e %H:%M:%S %Z %Y")?
            // Common American formats
            .with_value("MM/DD/YYYY", "%m/%d/%Y")?
            .with_value("MM/DD/YYYY HH:MM (AM/PM)", "%m/%d/%Y %I:%M %p")?
            .with_value("MM/DD/YY", "%m/%d/%y")?
            .with_value("HH:MM (AM/PM)", "%I:%M %p")?
            .with_value("AMERICAN_FULL_DATE_TIME", "%A, %B %d, %Y %I:%M:%S %p")?
            .build_readonly()?
        )?
        .with_function("months", | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
            let function_name = "time.months(months: number)";
            let months = match multivalue.pop_front() {
                Some(LuaValue::Number(f)) => f as i64,
                Some(LuaValue::Integer(i)) => i,
                other => {
                    return wrap_err!("{} expected days to be an integer number, got: {:?}", function_name, other);
                }
            };
            let relative_to = match multivalue.pop_front() {
                Some(LuaValue::UserData(ud)) => {
                    if ud.is::<DateTime>() {
                        let dt = ud.borrow::<DateTime>().expect("impossible not DateTime");
                        Some(dt.deref().clone())
                    } else {
                        let type_name = ud.type_name()?.unwrap_or_default();
                        return wrap_err!("{} expected relative_to to be a DateTime or nil/unspecified, got a userdata of type: {}", function_name, type_name);
                    }
                },
                Some(LuaNil) | None => None,
                Some(other) => {
                    return wrap_err!("{} expected relative_to to be DateTime or nil/unspecified, got: {:?}", function_name, other);
                }
            };
            TimeSpan::months(months, relative_to).get_userdata(luau)
        })?
        .with_function("days", | luau: &Lua, value: LuaValue | -> LuaValueResult {
            let function_name = "time.days(d: number)";
            let days = match value {
                LuaValue::Number(f) => f as i64,
                LuaValue::Integer(i) => i,
                other => {
                    return wrap_err!("{} expected days to be an integer number, got: {:?}", function_name, other);
                }
            };
            TimeSpan::days(days).get_userdata(luau)
        })?
        .with_function("hours", | luau: &Lua, value: LuaValue | -> LuaValueResult {
            let function_name = "time.hours(hours: number)";
            let days = match value {
                LuaValue::Number(f) => f as i64,
                LuaValue::Integer(i) => i,
                other => {
                    return wrap_err!("{} expected hours to be an integer number, got: {:?}", function_name, other);
                }
            };
            TimeSpan::hours(days).get_userdata(luau)
        })?
        .with_function("minutes", | luau: &Lua, value: LuaValue | -> LuaValueResult {
            let function_name = "time.minutes(minutes: number)";
            let days = match value {
                LuaValue::Number(f) => f as i64,
                LuaValue::Integer(i) => i,
                other => {
                    return wrap_err!("{} expected minutes to be an integer number, got: {:?}", function_name, other);
                }
            };
            TimeSpan::minutes(days).get_userdata(luau)
        })?
        .with_function("seconds", | luau: &Lua, value: LuaValue | -> LuaValueResult {
            let function_name = "time.seconds(seconds: number)";
            let days = match value {
                LuaValue::Number(f) => f as i64,
                LuaValue::Integer(i) => i,
                other => {
                    return wrap_err!("{} expected seconds to be an integer number, got: {:?}", function_name, other);
                }
            };
            TimeSpan::seconds(days).get_userdata(luau)
        })?
        .build_readonly()
}