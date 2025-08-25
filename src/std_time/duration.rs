use crate::prelude::*;
use mluau::prelude::*;
use jiff::SignedDuration;

/// using SignedDuration instead of std::time::Duration
/// this is because we want to allow time.days(3) - time.days(5)
pub struct TimeDuration {
    pub inner: SignedDuration, 
}

const SECONDS_IN_A_YEAR: f64 = 31_536_000.0;
const SECONDS_IN_A_DAY: f64 = 86_400.0;
const SECONDS_IN_AN_HOUR: f64 = 3_600.0;
const SECONDS_IN_A_MINUTE: f64 = 60.0;

impl TimeDuration {
    pub fn new(duration: SignedDuration) -> Self {
        Self { inner: duration }
    }

    pub fn years(years: f64) -> LuaResult<Self> {
        let secs = years * SECONDS_IN_A_YEAR;
        let signed = match SignedDuration::try_from_secs_f64(secs) {
            Ok(signed) => signed,
            Err(err) => {
                return wrap_err!("years overflowed bounds, err: {}", err);
            }
        };
        Ok(Self::new(signed))
    }

    pub fn months(months: f64) -> LuaResult<Self> {
        let secs = months * (SECONDS_IN_A_YEAR / 12.0);
        let signed = match SignedDuration::try_from_secs_f64(secs) {
            Ok(signed) => signed,
            Err(err) => {
                return wrap_err!("months overflowed bounds, err: {}", err);
            }
        };
        Ok(Self::new(signed))
    }

    pub fn days(days: f64) -> LuaResult<Self> {
        let secs = days * SECONDS_IN_A_DAY;
        let signed = match SignedDuration::try_from_secs_f64(secs) {
            Ok(signed) => signed,
            Err(err) => {
                return wrap_err!("days overflowed bounds, err: {}", err);
            }
        };
        Ok(Self::new(signed))
    }

    pub fn hours(hours: f64) -> LuaResult<Self> {
        let secs = hours * SECONDS_IN_AN_HOUR;
        let signed = match SignedDuration::try_from_secs_f64(secs) {
            Ok(signed) => signed,
            Err(err) => {
                return wrap_err!("hours overflowed bounds, err: {}", err);
            }
        };
        Ok(Self::new(signed))
    }

    pub fn minutes(mins: f64) -> LuaResult<Self> {
        let secs = mins * SECONDS_IN_A_MINUTE;
        let signed = match SignedDuration::try_from_secs_f64(secs) {
            Ok(signed) => signed,
            Err(err) => {
                return wrap_err!("minutes overflowed bounds, err: {}", err);
            }
        };
        Ok(Self::new(signed))
    }

    pub fn seconds(secs: f64) -> LuaResult<Self> {
        let signed = match SignedDuration::try_from_secs_f64(secs) {
            Ok(signed) => signed,
            Err(err) => {
                return wrap_err!("seconds overflowed bounds, err: {}", err);
            }
        };
        Ok(Self::new(signed))
    }

    pub fn milliseconds(ms: f64) -> LuaResult<Self> {
        let secs = ms / 1_000.0;
        let signed = match SignedDuration::try_from_secs_f64(secs) {
            Ok(signed) => signed,
            Err(err) => {
                return wrap_err!("milliseconds overflowed bounds, err: {}", err);
            }
        };
        Ok(Self::new(signed))
    }

    pub fn microseconds(us: f64) -> LuaResult<Self> {
        let secs = us / 1_000_000.0;
        let signed = match SignedDuration::try_from_secs_f64(secs) {
            Ok(signed) => signed,
            Err(err) => {
                return wrap_err!("microseconds overflowed bounds, err: {}", err);
            }
        };
        Ok(Self::new(signed))
    }

    pub fn nanoseconds(ns: f64) -> LuaResult<Self> {
        let secs = ns / 1_000_000_000.0;
        let signed = match SignedDuration::try_from_secs_f64(secs) {
            Ok(signed) => signed,
            Err(err) => {
                return wrap_err!("nanoseconds overflowed bounds, err: {}", err);
            }
        };
        Ok(Self::new(signed))
    }

    pub fn get_userdata(self, luau: &Lua) -> LuaValueResult {
        ok_userdata(self, luau)
    }
}


impl LuaUserData for TimeDuration {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_meta_field("__type", "Duration");

        fn round_4_decimals(val: f64) -> f64 {
            (val * 10_000.0).round() / 10_000.0
        }

        fields.add_field_method_get("years", |_, this| {
            let years = this.inner.as_secs_f64() / 31_536_000.0;
            Ok(round_4_decimals(years))
        });

        fields.add_field_method_get("months", |_, this| {
            // 1 month = 1/12 of a year = 2_628_000 seconds
            let months = this.inner.as_secs_f64() / (31_536_000.0 / 12.0);
            Ok(round_4_decimals(months))
        });

        fields.add_field_method_get("days", |_, this| {
            Ok(round_4_decimals(this.inner.as_secs_f64() / 86_400.0))
        });
        fields.add_field_method_get("hours", |_, this| {
            Ok(round_4_decimals(this.inner.as_secs_f64() / 3_600.0))
        });
        fields.add_field_method_get("minutes", |_, this| {
            Ok(round_4_decimals(this.inner.as_secs_f64() / 60.0))
        });
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


