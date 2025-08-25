use mluau::prelude::*;
use crate::prelude::*;
use crate::std_time::{datetime::DateTime, duration::TimeDuration};
use jiff::{Span, SpanArithmetic, SpanRelativeTo};
use std::ops::Deref;

pub struct TimeSpan {
    pub inner: Span,
    pub relative_to: Option<DateTime>,
}

impl TimeSpan {
    pub fn new(span: Span) -> Self {
        Self {
            inner: span,
            relative_to: None,
        }
    }
    pub fn relative_to(span: Span, relative_to: DateTime) -> Self {
        Self {
            inner: span,
            relative_to: Some(relative_to),
        }
    }
    pub fn years(years: i64, relative_to: Option<DateTime>) -> Self {
        let clamped = years.clamp(-19_998, 19_998);
        if let Some(relative) = relative_to {
            Self::relative_to(
                Span::new().years(clamped), 
                relative
            )
        } else {
            Self::new(Span::new().years(clamped))
        }
    }
    pub fn months(months: i64, relative_to: Option<DateTime>) -> Self {
        let clamped = months.clamp(-239_976, 239_976);
        if let Some(relative) = relative_to {
            Self::relative_to(
                Span::new().months(clamped), 
                relative
            )
        } else {
            Self::new(Span::new().months(clamped))
        }
    }
    pub fn days(days: i64) -> Self {
        // Self::new(Span::new().days(days))
        Self::new(Span::new().days(days.clamp(-7_304_484, 7_304_484)))
    }
    pub fn hours(hours: i64) -> Self {
        Self::new(Span::new().hours(hours.clamp(-175_307_616, 175_307_616)))
    }
    pub fn minutes(mins: i64) -> Self {
        Self::new(Span::new().minutes(mins.clamp(-10_518_456_960, 10_518_456_960)))
    }
    pub fn seconds(secs: i64) -> Self {
        Self::new(Span::new().seconds(secs.clamp(-631_107_417_600, 631_107_417_600)))
    }
    pub fn get_userdata(self, luau: &Lua) -> LuaValueResult {
        ok_userdata(self, luau)
    }
}

impl LuaUserData for TimeSpan {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("relative_to", |luau: &Lua, this: &TimeSpan| {
            if let Some(relative_to) = &this.relative_to {
                relative_to.clone().get_userdata(luau)
            } else {
                Ok(LuaNil)
            }
        });
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, | luau: &Lua, this: &TimeSpan, _: LuaValue| -> LuaValueResult {
            ok_string(format!("TimeSpan<{:#}>", this.inner), luau)
        });
        // we actually don't want to make these as easy to access as with DateTime & TimeDuration
        // because they're the internal representations of the units and probably not what users want
        methods.add_method("units", |luau: &Lua, this: &TimeSpan, _: LuaValue| {
            TableBuilder::create(luau)?
                .with_value("years", this.inner.get_years())?
                .with_value("months", this.inner.get_months())?
                .with_value("days", this.inner.get_days())?
                .with_value("hours", this.inner.get_hours())?
                .with_value("minutes", this.inner.get_minutes())?
                .with_value("seconds", this.inner.get_seconds())?
                .with_value("milliseconds", this.inner.get_milliseconds())?
                .with_value("microseconds", this.inner.get_microseconds())?
                .build_readonly()
        });
        methods.add_method("duration", |luau: &Lua, this: &TimeSpan, _: LuaValue| {
            let function_name = "TimeSpan:duration()";
            let total = match if let Some(relative) = &this.relative_to {
                this.inner.to_duration(relative.date())
            } else {
                this.inner.to_duration(SpanRelativeTo::days_are_24_hours())
            } {
                Ok(total) => total,
                Err(err) => {
                    return wrap_err!("{} unable to convert to duration: {}", function_name, err);
                }
            };
            ok_userdata(TimeDuration::new(total), luau)
        });

        /// we want to not error.. so we check if either TimeSpan is relative_to a DateTime for SpanArithmetic
        fn which_relative<'a>(this: &'a TimeSpan, other: &'a TimeSpan) -> Option<&'a DateTime> {
            if let Some(this_relative) = &this.relative_to {
                Some(this_relative)
            } else if let Some(other_relative) = &other.relative_to {
                Some(other_relative)
            } else {
                None
            }
        }

        methods.add_meta_method(LuaMetaMethod::Add, | luau: &Lua, this: &TimeSpan, other: LuaValue | {
            let function_name = "TimeSpan.__add(self, other: TimeSpan)";
            let other = match other {
                LuaValue::UserData(ud) => {
                    if ud.is::<TimeSpan>() {
                        ud.borrow::<TimeSpan>().expect("must be TimeSpan here")
                    } else {
                        return wrap_err!("{}: other must be another TimeSpan", function_name);
                    }
                },
                other => {
                    return wrap_err!("{} expected other to be another TimeSpan, got: {:?}", function_name, other);
                }
            };

            let relative_to = which_relative(this, &other);

            let subbed = match if let Some(relative) = relative_to {
                this.inner.checked_add((other.deref().inner, relative.date()))
            } else {
                this.inner.checked_add(SpanArithmetic::from(other.deref().inner).days_are_24_hours())
            } {
                Ok(span) => span,
                Err(err) => {
                    return wrap_err!("{} error subtracting timespans {} + {}; err: {}", function_name, this.inner, other.inner, err);
                }
            };
            if let Some(relative) = relative_to {
                TimeSpan::relative_to(subbed, relative.clone()).get_userdata(luau)
            } else {
                TimeSpan::new(subbed).get_userdata(luau)
            }
        });

        methods.add_meta_method(LuaMetaMethod::Sub, | luau: &Lua, this: &TimeSpan, other: LuaValue | {
            let function_name = "TimeSpan.__sub(self, other: TimeSpan)";
            let other = match other {
                LuaValue::UserData(ud) => {
                    if ud.is::<TimeSpan>() {
                        ud.borrow::<TimeSpan>().expect("must be TimeSpan here")
                    } else {
                        return wrap_err!("{}: other must be another TimeSpan", function_name);
                    }
                },
                other => {
                    return wrap_err!("{} expected other to be another TimeSpan, got: {:?}", function_name, other);
                }
            };

            let relative_to = which_relative(this, &other);

            let subbed = match if let Some(relative) = relative_to {
                this.inner.checked_sub((other.deref().inner, relative.date()))
            } else {
                this.inner.checked_sub(SpanArithmetic::from(other.deref().inner).days_are_24_hours())
            } {
                Ok(span) => span,
                Err(err) => {
                    return wrap_err!("{} error subtracting timespans {} + {}; err: {}", function_name, this.inner, other.inner, err);
                }
            };
            if let Some(relative) = relative_to {
                TimeSpan::relative_to(subbed, relative.clone()).get_userdata(luau)
            } else {
                TimeSpan::new(subbed).get_userdata(luau)
            }
        });
    }
}