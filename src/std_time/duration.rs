use std::ops::Deref;

use crate::{prelude::*, std_time::datetime::DateTime};
use mluau::prelude::*;
use jiff::{SignedDuration, Span, SpanArithmetic, SpanRelativeTo};

/// using SignedDuration instead of std::time::Duration
/// this is because we want to allow time.days(3) - time.days(5)
pub struct TimeDuration {
    pub inner: SignedDuration, 
}

impl TimeDuration {
    pub fn new(duration: SignedDuration) -> Self {
        Self { inner: duration }
    }

    pub fn days(days: f64) -> Self {
        let secs = (days * 86_400.0).clamp(-631_107_417_600.0, 631_107_417_600.0);
        Self::new(SignedDuration::from_secs_f64(secs))
    }

    pub fn hours(hours: f64) -> Self {
        let secs = (hours * 3_600.0).clamp(-631_107_417_600.0, 631_107_417_600.0);
        Self::new(SignedDuration::from_secs_f64(secs))
    }

    pub fn minutes(mins: f64) -> Self {
        let secs = (mins * 60.0).clamp(-631_107_417_600.0, 631_107_417_600.0);
        Self::new(SignedDuration::from_secs_f64(secs))
    }

    pub fn seconds(secs: f64) -> Self {
        let secs = secs.clamp(-631_107_417_600.0, 631_107_417_600.0);
        Self::new(SignedDuration::from_secs_f64(secs))
    }

    pub fn milliseconds(ms: f64) -> Self {
        let secs = (ms / 1_000.0).clamp(-631_107_417_600.0, 631_107_417_600.0);
        Self::new(SignedDuration::from_secs_f64(secs))
    }

    pub fn microseconds(us: f64) -> Self {
        let secs = (us / 1_000_000.0).clamp(-631_107_417_600.0, 631_107_417_600.0);
        Self::new(SignedDuration::from_secs_f64(secs))
    }

    pub fn nanoseconds(ns: f64) -> Self {
        let secs = (ns / 1_000_000_000.0).clamp(-631_107_417_600.0, 631_107_417_600.0);
        Self::new(SignedDuration::from_secs_f64(secs))
    }

    pub fn get_userdata(self, luau: &Lua) -> LuaValueResult {
        ok_userdata(self, luau)
    }
}


impl LuaUserData for TimeDuration {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_meta_field("__type", "Duration");

        fields.add_field_method_get("days", |_, this| Ok(this.inner.as_secs_f64() / 86_400.0));
        fields.add_field_method_get("hours", |_, this| Ok(this.inner.as_secs_f64() / 3_600.0));
        fields.add_field_method_get("minutes", |_, this| Ok(this.inner.as_secs_f64() / 60.0));
        fields.add_field_method_get("seconds", |_, this| Ok(this.inner.as_secs_f64()));
        fields.add_field_method_get("milliseconds", |_, this| Ok(this.inner.as_secs_f64() * 1_000.0));
        fields.add_field_method_get("microseconds", |_, this| Ok(this.inner.as_secs_f64() * 1_000_000.0));
        fields.add_field_method_get("nanoseconds", |_, this| Ok(this.inner.as_secs_f64() * 1_000_000_000.0));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |luau, this: &TimeDuration, _: LuaValue| {
            ok_string(format!("Duration<{}s>", this.inner.as_secs_f64()), luau)
        });

        methods.add_method("abs", |luau: &Lua, this: &TimeDuration, _: LuaValue| {
            let signed = this.inner.abs();
            ok_userdata(TimeDuration::new(signed), luau)
        });

        methods.add_method("display", |luau: &Lua, this: &TimeDuration, _: LuaValue| {
            let secs = this.inner.as_secs_f64();
            let (value, unit) = if secs.abs() >= 86_400.0 {
                (secs / 86_400.0, "days")
            } else if secs.abs() >= 3_600.0 {
                (secs / 3_600.0, "hours")
            } else if secs.abs() >= 60.0 {
                (secs / 60.0, "minutes")
            } else if secs.abs() >= 1.0 {
                (secs, "seconds")
            } else if secs.abs() >= 0.001 {
                (secs * 1_000.0, "milliseconds")
            } else if secs.abs() >= 0.000_001 {
                (secs * 1_000_000.0, "microseconds")
            } else {
                (secs * 1_000_000_000.0, "nanoseconds")
            };

            let formatted = if value.fract() == 0.0 {
                // is a whole number (show 30 days not 30.00 days)
                format!("{} {}", value as i64, unit)
            } else { // keep fractional representation
                format!("{:.2} {}", value, unit)
            };

            ok_string(formatted, luau)
        });

        methods.add_meta_method(LuaMetaMethod::Add, |luau, this, other| {
            let function_name = "TimeDuration.__add(self, other: TimeDuration)";
            let result = match other {
                LuaValue::UserData(ud) => match ud.borrow::<TimeDuration>() {
                    Ok(other) => this.inner.checked_add(other.inner),
                    Err(err) => return wrap_err!("{}: other must be TimeDuration; err: {}", function_name, err),
                },
                other => return wrap_err!("{} expected TimeDuration, got {:?}", function_name, other),
            };

            match result {
                Some(sum) => ok_userdata(TimeDuration::new(sum), luau),
                None => wrap_err!("{} overflow when adding durations", function_name),
            }
        });

        methods.add_meta_method(LuaMetaMethod::Sub, |luau, this, other| {
            let function_name = "TimeDuration.__sub(self, other: TimeDuration)";
            let result = match other {
                LuaValue::UserData(ud) => match ud.borrow::<TimeDuration>() {
                    Ok(other) => this.inner.checked_sub(other.inner),
                    Err(err) => return wrap_err!("{}: other must be TimeDuration; err: {}", function_name, err),
                },
                other => return wrap_err!("{} expected TimeDuration, got {:?}", function_name, other),
            };

            match result {
                Some(diff) => ok_userdata(TimeDuration::new(diff), luau),
                None => wrap_err!("{}: underflow when subtracting durations", function_name),
            }
        });
        
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, other| {
            let function_name = "TimeDuration.__eq(self, other: TimeDuration)";
            match other {
                LuaValue::UserData(ud) => match ud.borrow::<TimeDuration>() {
                    Ok(other) => Ok(this.inner == other.inner),
                    Err(err) => wrap_err!("{}: other must be TimeDuration; err: {}", function_name, err),
                },
                other => wrap_err!("{} expected TimeDuration, got {:?}", function_name, other),
            }
        });

        methods.add_meta_method(LuaMetaMethod::Lt, |_, this, other| {
            let function_name = "TimeDuration.__lt(self, other: TimeDuration)";
            match other {
                LuaValue::UserData(ud) => match ud.borrow::<TimeDuration>() {
                    Ok(other) => Ok(this.inner < other.inner),
                    Err(err) => wrap_err!("{}: other must be TimeDuration; err: {}", function_name, err),
                },
                other => wrap_err!("{} expected TimeDuration, got {:?}", function_name, other),
            }
        });

        methods.add_meta_method(LuaMetaMethod::Le, |_, this, other| {
            let function_name = "TimeDuration.__le(self, other: TimeDuration)";
            match other {
                LuaValue::UserData(ud) => match ud.borrow::<TimeDuration>() {
                    Ok(other) => Ok(this.inner <= other.inner),
                    Err(err) => wrap_err!("{}: other must be TimeDuration; err: {}", function_name, err),
                },
                other => wrap_err!("{} expected TimeDuration, got {:?}", function_name, other),
            }
        });
        
    }
}

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
        fields.add_field_method_get("years", |_: &Lua, this: &TimeSpan| Ok(this.inner.get_years()));
        fields.add_field_method_get("months", |_: &Lua, this: &TimeSpan| Ok(this.inner.get_months()));
        fields.add_field_method_get("days", |_: &Lua, this: &TimeSpan| Ok(this.inner.get_days()));
        fields.add_field_method_get("hours", |_: &Lua, this: &TimeSpan| Ok(this.inner.get_hours()));
        fields.add_field_method_get("minutes", |_: &Lua, this: &TimeSpan| Ok(this.inner.get_minutes()));
        fields.add_field_method_get("seconds", |_: &Lua, this: &TimeSpan| Ok(this.inner.get_seconds()));
        fields.add_field_method_get("milliseconds", |_: &Lua, this: &TimeSpan| Ok(this.inner.get_milliseconds()));
        fields.add_field_method_get("microseconds", |_: &Lua, this: &TimeSpan| Ok(this.inner.get_microseconds()));
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, | luau: &Lua, this: &TimeSpan, _: LuaValue| -> LuaValueResult {
            ok_string(format!("TimeSpan<{:#}>", this.inner), luau)
        });

        methods.add_method("duration", |luau: &Lua, this: &TimeSpan, _: LuaValue| {
            let function_name = "TimeSpan:duration()";
            let total = match this.inner.to_duration(SpanRelativeTo::days_are_24_hours()) {
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
            let added = match other {
                LuaValue::UserData(ud) => match ud.borrow::<TimeSpan>() {
                    Ok(other) => {
                        let relative_to = which_relative(this, &other);
                        match if let Some(relative) = relative_to {
                            this.inner.checked_add((other.deref().inner, relative.date()))
                        } else {
                            this.inner.checked_add(SpanArithmetic::from(other.deref().inner).days_are_24_hours())
                        } {
                            Ok(span) => span,
                            Err(err) => {
                                return wrap_err!("{} error adding timespans {} + {}; err: {}", function_name, this.inner, other.inner, err);
                            }
                        }
                    },
                    Err(err) => {
                        return wrap_err!("{}: other must be another TimeSpan; err: {:?}", function_name, err);
                    }
                },
                other => {
                    return wrap_err!("{} expected other to be another TimeSpan, got: {:?}", function_name, other);
                }
            };
            ok_userdata(TimeSpan::new(added), luau)
        });

        methods.add_meta_method(LuaMetaMethod::Sub, | luau: &Lua, this: &TimeSpan, other: LuaValue | {
            let function_name = "TimeSpan.__sub(self, other: TimeSpan)";
            let subbed = match other {
                LuaValue::UserData(ud) => match ud.borrow::<TimeSpan>() {
                    Ok(other) => {
                        let relative_to = which_relative(this, &other);
                        match if let Some(relative) = relative_to {
                            this.inner.checked_sub((other.deref().inner, relative.date()))
                        } else {
                            this.inner.checked_sub(SpanArithmetic::from(other.deref().inner).days_are_24_hours())
                        } {
                            Ok(span) => span,
                            Err(err) => {
                                return wrap_err!("{} error subtracting timespans {} + {}; err: {}", function_name, this.inner, other.inner, err);
                            }
                        }
                    },
                    Err(err) => {
                        return wrap_err!("{}: other must be another TimeSpan; err: {:?}", function_name, err);
                    }
                },
                other => {
                    return wrap_err!("{} expected other to be another TimeSpan, got: {:?}", function_name, other);
                }
            };
            ok_userdata(TimeSpan::new(subbed), luau)
        });
    }
}

