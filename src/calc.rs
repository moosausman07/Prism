//! Natural-language calculator: arithmetic, percentages, unit and currency
//! conversion, and date/time math. Pure logic — no I/O. Currency uses a
//! USD-based rate table supplied by the caller (fetched once by the backend).

use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub enum CalcKind {
    Math,
    Unit,
    Currency,
    Time,
}

impl CalcKind {
    pub fn label(&self) -> &'static str {
        match self {
            CalcKind::Math => "CALCULATOR",
            CalcKind::Unit => "CONVERSION",
            CalcKind::Currency => "CURRENCY",
            CalcKind::Time => "TIME",
        }
    }
    pub fn icon(&self) -> &'static str {
        match self {
            CalcKind::Math => "calc",
            CalcKind::Unit => "ruler",
            CalcKind::Currency => "coins",
            CalcKind::Time => "clock",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct CalcHit {
    /// Primary result — shown large and copied to the clipboard on Enter.
    pub value: String,
    /// Secondary line echoing the interpreted input.
    pub label: String,
    pub kind: CalcKind,
}

/// Try every interpreter, most specific first. Returns `None` when the query
/// is not a calculation so the normal app/command results show instead.
pub fn evaluate(input: &str, rates: &HashMap<String, f64>) -> Option<CalcHit> {
    let q = input.trim();
    if q.len() < 2 {
        return None;
    }
    time_calc(q)
        .or_else(|| currency_calc(q, rates))
        .or_else(|| unit_calc(q))
        .or_else(|| math_calc(q))
}

// ===========================================================================
// Number formatting
// ===========================================================================

fn fmt_num(x: f64) -> String {
    if !x.is_finite() {
        return "—".into();
    }
    let ax = x.abs();
    if ax != 0.0 && (ax >= 1e15 || ax < 1e-6) {
        // Scientific for extreme magnitudes.
        let s = format!("{:e}", x);
        return s;
    }
    // Round to 6 decimals, strip trailing zeros.
    let mut s = format!("{:.6}", x);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    // Thousands separators on the integer part.
    let neg = s.starts_with('-');
    let body = if neg { &s[1..] } else { &s[..] };
    let (int_part, frac) = match body.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (body, None),
    };
    let mut grouped = String::new();
    let bytes = int_part.as_bytes();
    for (i, c) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(*c as char);
    }
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    out.push_str(&grouped);
    if let Some(f) = frac {
        out.push('.');
        out.push_str(f);
    }
    out
}

/// Parse a leading number (optional sign, commas as thousands, decimal,
/// scientific). Returns (value, rest-after-number).
fn parse_leading_number(s: &str) -> Option<(f64, &str)> {
    let s = s.trim_start();
    let bytes = s.as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    let start_digits = i;
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b',') {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i == start_digits {
        return None;
    }
    // Optional exponent.
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        let mut j = i + 1;
        if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
            j += 1;
        }
        let ds = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j > ds {
            i = j;
        }
    }
    let num: String = s[..i].chars().filter(|c| *c != ',').collect();
    num.parse::<f64>().ok().map(|v| (v, &s[i..]))
}

// ===========================================================================
// Arithmetic expression evaluator (recursive descent)
// ===========================================================================

fn math_calc(q: &str) -> Option<CalcHit> {
    let lc = q.to_lowercase();

    // Percentage phrases handled before the generic parser.
    if let Some(v) = percent_phrase(&lc) {
        return Some(CalcHit {
            value: fmt_num(v),
            label: q.to_string(),
            kind: CalcKind::Math,
        });
    }

    // Only treat as math if it has a digit/constant and looks arithmetic.
    let has_digit = lc.chars().any(|c| c.is_ascii_digit());
    let has_op = lc.contains(['+', '*', '/', '^', '%'])
        || lc.matches('-').count() > 0 && has_digit
        || ["sqrt", "sin", "cos", "tan", "log", "ln", "abs", "pi"]
            .iter()
            .any(|f| lc.contains(f));
    if !has_digit || !has_op {
        return None;
    }

    let mut p = Parser {
        s: lc.as_bytes(),
        i: 0,
    };
    let v = p.expr().ok()?;
    p.skip_ws();
    if p.i != p.s.len() {
        return None;
    }
    Some(CalcHit {
        value: fmt_num(v),
        label: q.to_string(),
        kind: CalcKind::Math,
    })
}

/// "X% of Y", "Y + X%", "Y - X%", "X percent of Y".
fn percent_phrase(lc: &str) -> Option<f64> {
    let t = lc.replace("percent", "%").replace(" of ", " of ");
    if let Some((a, b)) = t.split_once(" of ") {
        let a = a.trim().trim_end_matches('%').trim();
        let x = parse_leading_number(a)?.0;
        let y = parse_leading_number(b.trim())?.0;
        return Some(x / 100.0 * y);
    }
    for (sign, op) in [(" + ", 1.0), (" - ", -1.0), (" plus ", 1.0), (" minus ", -1.0)] {
        if let Some((a, b)) = t.split_once(sign) {
            let b = b.trim();
            if let Some(pc) = b.strip_suffix('%') {
                let base = parse_leading_number(a.trim())?.0;
                let pct = parse_leading_number(pc.trim())?.0;
                return Some(base + op * (base * pct / 100.0));
            }
        }
    }
    None
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn expr(&mut self) -> Result<f64, ()> {
        let mut v = self.term()?;
        loop {
            self.skip_ws();
            match self.s.get(self.i) {
                Some(b'+') => {
                    self.i += 1;
                    v += self.term()?;
                }
                Some(b'-') => {
                    self.i += 1;
                    v -= self.term()?;
                }
                _ => break,
            }
        }
        Ok(v)
    }

    fn term(&mut self) -> Result<f64, ()> {
        let mut v = self.power()?;
        loop {
            self.skip_ws();
            match self.s.get(self.i) {
                Some(b'*') => {
                    self.i += 1;
                    v *= self.power()?;
                }
                Some(b'/') => {
                    self.i += 1;
                    let d = self.power()?;
                    if d == 0.0 {
                        return Err(());
                    }
                    v /= d;
                }
                Some(b'%') => {
                    self.i += 1;
                    let d = self.power()?;
                    if d == 0.0 {
                        return Err(());
                    }
                    v %= d;
                }
                _ => break,
            }
        }
        Ok(v)
    }

    fn power(&mut self) -> Result<f64, ()> {
        let b = self.unary()?;
        self.skip_ws();
        if self.s.get(self.i) == Some(&b'^') {
            self.i += 1;
            let e = self.power()?;
            return Ok(b.powf(e));
        }
        Ok(b)
    }

    fn unary(&mut self) -> Result<f64, ()> {
        self.skip_ws();
        match self.s.get(self.i) {
            Some(b'-') => {
                self.i += 1;
                Ok(-self.unary()?)
            }
            Some(b'+') => {
                self.i += 1;
                self.unary()
            }
            _ => self.atom(),
        }
    }

    fn atom(&mut self) -> Result<f64, ()> {
        self.skip_ws();
        match self.s.get(self.i) {
            Some(b'(') => {
                self.i += 1;
                let v = self.expr()?;
                self.skip_ws();
                if self.s.get(self.i) != Some(&b')') {
                    return Err(());
                }
                self.i += 1;
                Ok(v)
            }
            Some(c) if c.is_ascii_alphabetic() => {
                let start = self.i;
                while self.i < self.s.len() && self.s[self.i].is_ascii_alphabetic() {
                    self.i += 1;
                }
                let name = std::str::from_utf8(&self.s[start..self.i]).map_err(|_| ())?;
                match name {
                    "pi" => Ok(std::f64::consts::PI),
                    "tau" => Ok(std::f64::consts::TAU),
                    "e" => Ok(std::f64::consts::E),
                    _ => {
                        self.skip_ws();
                        if self.s.get(self.i) != Some(&b'(') {
                            return Err(());
                        }
                        self.i += 1;
                        let arg = self.expr()?;
                        self.skip_ws();
                        if self.s.get(self.i) != Some(&b')') {
                            return Err(());
                        }
                        self.i += 1;
                        Ok(match name {
                            "sqrt" => arg.sqrt(),
                            "cbrt" => arg.cbrt(),
                            "abs" => arg.abs(),
                            "round" => arg.round(),
                            "floor" => arg.floor(),
                            "ceil" => arg.ceil(),
                            "sin" => arg.sin(),
                            "cos" => arg.cos(),
                            "tan" => arg.tan(),
                            "asin" => arg.asin(),
                            "acos" => arg.acos(),
                            "atan" => arg.atan(),
                            "ln" => arg.ln(),
                            "log" => arg.log10(),
                            "log2" => arg.log2(),
                            "exp" => arg.exp(),
                            _ => return Err(()),
                        })
                    }
                }
            }
            _ => {
                let rest = std::str::from_utf8(&self.s[self.i..]).map_err(|_| ())?;
                let (v, after) = parse_leading_number(rest).ok_or(())?;
                self.i += rest.len() - after.len();
                Ok(v)
            }
        }
    }
}

// ===========================================================================
// Unit conversion
// ===========================================================================

#[derive(PartialEq, Clone, Copy)]
enum Cat {
    Length,
    Mass,
    Temp,
    Volume,
    Speed,
    Data,
    Time,
    Area,
    Angle,
}

/// Returns (category, factor-to-base). Temperature is handled separately.
fn unit_def(u: &str) -> Option<(Cat, f64)> {
    let u = u.trim().trim_end_matches('.').trim();
    Some(match u {
        // length — base metre
        "mm" | "millimeter" | "millimeters" | "millimetre" => (Cat::Length, 0.001),
        "cm" | "centimeter" | "centimeters" => (Cat::Length, 0.01),
        "m" | "meter" | "meters" | "metre" | "metres" => (Cat::Length, 1.0),
        "km" | "kilometer" | "kilometers" | "kilometre" => (Cat::Length, 1000.0),
        "in" | "inch" | "inches" => (Cat::Length, 0.0254),
        "ft" | "foot" | "feet" => (Cat::Length, 0.3048),
        "yd" | "yard" | "yards" => (Cat::Length, 0.9144),
        "mi" | "mile" | "miles" => (Cat::Length, 1609.344),
        "nmi" | "nauticalmile" => (Cat::Length, 1852.0),
        // mass — base gram
        "mg" | "milligram" | "milligrams" => (Cat::Mass, 0.001),
        "g" | "gram" | "grams" => (Cat::Mass, 1.0),
        "kg" | "kilogram" | "kilograms" | "kilo" | "kilos" => (Cat::Mass, 1000.0),
        "t" | "tonne" | "tonnes" | "ton" => (Cat::Mass, 1_000_000.0),
        "oz" | "ounce" | "ounces" => (Cat::Mass, 28.349523125),
        "lb" | "lbs" | "pound" | "pounds" => (Cat::Mass, 453.59237),
        "st" | "stone" | "stones" => (Cat::Mass, 6350.29318),
        // temperature
        "c" | "celsius" | "centigrade" => (Cat::Temp, 0.0),
        "f" | "fahrenheit" => (Cat::Temp, 1.0),
        "k" | "kelvin" => (Cat::Temp, 2.0),
        // volume — base litre
        "ml" | "milliliter" | "milliliters" | "millilitre" => (Cat::Volume, 0.001),
        "l" | "liter" | "liters" | "litre" | "litres" => (Cat::Volume, 1.0),
        "m3" => (Cat::Volume, 1000.0),
        "gal" | "gallon" | "gallons" => (Cat::Volume, 3.785411784),
        "qt" | "quart" | "quarts" => (Cat::Volume, 0.946352946),
        "pt" | "pint" | "pints" => (Cat::Volume, 0.473176473),
        "cup" | "cups" => (Cat::Volume, 0.2365882365),
        "floz" | "fluidounce" => (Cat::Volume, 0.0295735296),
        "tbsp" | "tablespoon" | "tablespoons" => (Cat::Volume, 0.0147867648),
        "tsp" | "teaspoon" | "teaspoons" => (Cat::Volume, 0.00492892159),
        // speed — base m/s
        "mps" => (Cat::Speed, 1.0),
        "kph" | "kmh" | "kmph" => (Cat::Speed, 0.277777778),
        "mph" => (Cat::Speed, 0.44704),
        "fps" => (Cat::Speed, 0.3048),
        "knot" | "knots" | "kn" => (Cat::Speed, 0.514444444),
        // data — base byte
        "bit" | "bits" => (Cat::Data, 0.125),
        "byte" | "bytes" => (Cat::Data, 1.0),
        "kb" | "kilobyte" | "kilobytes" => (Cat::Data, 1_000.0),
        "mb" | "megabyte" | "megabytes" => (Cat::Data, 1_000_000.0),
        "gb" | "gigabyte" | "gigabytes" => (Cat::Data, 1_000_000_000.0),
        "tb" | "terabyte" | "terabytes" => (Cat::Data, 1_000_000_000_000.0),
        "kib" => (Cat::Data, 1024.0),
        "mib" => (Cat::Data, 1_048_576.0),
        "gib" => (Cat::Data, 1_073_741_824.0),
        "tib" => (Cat::Data, 1_099_511_627_776.0),
        // time — base second
        "ms" | "millisecond" | "milliseconds" => (Cat::Time, 0.001),
        "s" | "sec" | "secs" | "second" | "seconds" => (Cat::Time, 1.0),
        "min" | "mins" | "minute" | "minutes" => (Cat::Time, 60.0),
        "h" | "hr" | "hrs" | "hour" | "hours" => (Cat::Time, 3600.0),
        "day" | "days" => (Cat::Time, 86400.0),
        "week" | "weeks" => (Cat::Time, 604800.0),
        "month" | "months" => (Cat::Time, 2_629_746.0),
        "year" | "years" | "yr" => (Cat::Time, 31_556_952.0),
        // area — base m²
        "m2" | "sqm" => (Cat::Area, 1.0),
        "cm2" => (Cat::Area, 0.0001),
        "km2" | "sqkm" => (Cat::Area, 1_000_000.0),
        "ft2" | "sqft" => (Cat::Area, 0.09290304),
        "mi2" | "sqmi" => (Cat::Area, 2_589_988.110336),
        "acre" | "acres" => (Cat::Area, 4046.8564224),
        "ha" | "hectare" | "hectares" => (Cat::Area, 10_000.0),
        // angle — base degree
        "deg" | "degree" | "degrees" => (Cat::Angle, 1.0),
        "rad" | "radian" | "radians" => (Cat::Angle, 57.29577951308232),
        "grad" | "gradian" | "gradians" => (Cat::Angle, 0.9),
        _ => return None,
    })
}

fn split_connector(q: &str) -> Option<(&str, &str)> {
    for c in [" to ", " in ", " as ", " into ", "->", "→", " >> "] {
        if let Some(idx) = q.find(c) {
            return Some((&q[..idx], &q[idx + c.len()..]));
        }
    }
    None
}

fn temp_to_c(v: f64, code: f64) -> f64 {
    match code as i32 {
        1 => (v - 32.0) * 5.0 / 9.0, // F
        2 => v - 273.15,            // K
        _ => v,                     // C
    }
}
fn c_to_temp(c: f64, code: f64) -> f64 {
    match code as i32 {
        1 => c * 9.0 / 5.0 + 32.0,
        2 => c + 273.15,
        _ => c,
    }
}

fn unit_calc(q: &str) -> Option<CalcHit> {
    let lc = q.to_lowercase();
    let (lhs, rhs) = split_connector(&lc)?;
    let (amount, after) = parse_leading_number(lhs.trim())?;
    let from_u = after.trim().replace(' ', "");
    let to_u = rhs.trim().replace(' ', "");
    if from_u.is_empty() || to_u.is_empty() {
        return None;
    }
    let (fc, ff) = unit_def(&from_u)?;
    let (tc, tf) = unit_def(&to_u)?;
    if fc != tc {
        return None;
    }
    let result = if fc == Cat::Temp {
        c_to_temp(temp_to_c(amount, ff), tf)
    } else {
        amount * ff / tf
    };
    Some(CalcHit {
        value: format!("{} {}", fmt_num(result), pretty_unit(&to_u)),
        label: format!("{} {} =", fmt_num(amount), pretty_unit(&from_u)),
        kind: CalcKind::Unit,
    })
}

fn pretty_unit(u: &str) -> String {
    match u {
        "c" => "°C".into(),
        "f" => "°F".into(),
        "k" => "K".into(),
        other => other.to_string(),
    }
}

// ===========================================================================
// Currency conversion (USD-based rate table)
// ===========================================================================

fn currency_code(tok: &str) -> Option<String> {
    let t = tok.trim().trim_end_matches('s');
    let c = match t {
        "$" | "usd" | "dollar" | "dollars" | "us$" => "USD",
        "€" | "eur" | "euro" | "euros" => "EUR",
        "£" | "gbp" | "pound" | "quid" => "GBP",
        "¥" | "jpy" | "yen" => "JPY",
        "₹" | "inr" | "rupee" | "rs" => "INR",
        "₽" | "rub" | "ruble" => "RUB",
        "c$" | "cad" => "CAD",
        "a$" | "aud" => "AUD",
        "chf" | "franc" => "CHF",
        "cny" | "rmb" | "yuan" => "CNY",
        "₩" | "krw" | "won" => "KRW",
        "₺" | "try" | "lira" => "TRY",
        "د.إ" | "aed" | "dirham" => "AED",
        "﷼" | "sar" | "riyal" => "SAR",
        "r$" | "brl" | "real" => "BRL",
        "mxn" | "peso" => "MXN",
        "sgd" => "SGD",
        "hkd" => "HKD",
        "nzd" => "NZD",
        "sek" => "SEK",
        "nok" => "NOK",
        "dkk" => "DKK",
        "zar" | "rand" => "ZAR",
        "pkr" => "PKR",
        "btc" | "bitcoin" => "BTC",
        s if s.len() == 3 && s.chars().all(|c| c.is_ascii_alphabetic()) => {
            return Some(s.to_uppercase())
        }
        _ => return None,
    };
    Some(c.to_string())
}

fn currency_calc(q: &str, rates: &HashMap<String, f64>) -> Option<CalcHit> {
    if rates.is_empty() {
        return None;
    }
    let lc = q.to_lowercase();
    let (lhs, rhs) = split_connector(&lc)?;
    let (amount, after) = parse_leading_number(lhs.trim())?;
    let from = currency_code(after.trim())?;
    let to = currency_code(rhs.trim())?;
    let rf = rates.get(&from)?;
    let rt = rates.get(&to)?;
    if *rf == 0.0 {
        return None;
    }
    let result = amount / rf * rt;
    Some(CalcHit {
        value: format!("{} {}", fmt_num(result), to),
        label: format!("{} {} · live rate", fmt_num(amount), from),
        kind: CalcKind::Currency,
    })
}

// ===========================================================================
// Date / time
// ===========================================================================

/// Days since 1970-01-01 for a proleptic-Gregorian date (Howard Hinnant's
/// algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

const MONTHS: [&str; 12] = [
    "january", "february", "march", "april", "may", "june", "july", "august",
    "september", "october", "november", "december",
];
const WEEKDAYS: [&str; 7] = [
    "Thursday", "Friday", "Saturday", "Sunday", "Monday", "Tuesday", "Wednesday",
];

fn now_ms() -> f64 {
    js_sys::Date::now()
}

fn today_days() -> i64 {
    (now_ms() / 86_400_000.0).floor() as i64
}

/// Parse a date phrase into days-since-epoch.
fn parse_date(s: &str) -> Option<i64> {
    let s = s.trim().trim_end_matches(['?', '.']).trim();
    match s {
        "today" | "now" => return Some(today_days()),
        "tomorrow" => return Some(today_days() + 1),
        "yesterday" => return Some(today_days() - 1),
        "christmas" | "xmas" => {
            let (y, _, _) = civil_from_days(today_days());
            return Some(days_from_civil(y, 12, 25));
        }
        "new year" | "new years" | "newyear" => {
            let (y, _, _) = civil_from_days(today_days());
            return Some(days_from_civil(y + 1, 1, 1));
        }
        _ => {}
    }
    // ISO: YYYY-MM-DD or YYYY/MM/DD
    let norm = s.replace('/', "-");
    let parts: Vec<&str> = norm.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(a), Ok(b), Ok(c)) =
            (parts[0].parse::<i64>(), parts[1].parse::<i64>(), parts[2].parse::<i64>())
        {
            // YYYY-MM-DD when first field is a 4-digit year, else DD-MM-YYYY.
            let (y, m, d) = if parts[0].len() == 4 { (a, b, c) } else { (c, b, a) };
            if (1..=12).contains(&m) && (1..=31).contains(&d) {
                return Some(days_from_civil(y, m, d));
            }
        }
    }
    // "25 december 2026" / "december 25 2026" / "december 25"
    let toks: Vec<&str> = s.split([' ', ',']).filter(|t| !t.is_empty()).collect();
    if toks.len() >= 2 {
        let mut day = None;
        let mut mon = None;
        let mut year = None;
        for t in &toks {
            if let Some(mi) = MONTHS.iter().position(|m| m.starts_with(*t) && t.len() >= 3) {
                mon = Some(mi as i64 + 1);
            } else if let Ok(n) =
                t.trim_end_matches(|c: char| c.is_ascii_alphabetic()).parse::<i64>()
            {
                if n > 31 {
                    year = Some(n);
                } else if day.is_none() {
                    day = Some(n);
                } else {
                    year = Some(n);
                }
            }
        }
        if let (Some(m), Some(d)) = (mon, day) {
            let y = year.unwrap_or_else(|| civil_from_days(today_days()).0);
            return Some(days_from_civil(y, m, d));
        }
    }
    None
}

fn fmt_date(days: i64) -> String {
    let (y, m, d) = civil_from_days(days);
    let wd = WEEKDAYS[(days.rem_euclid(7)) as usize];
    let mn = MONTHS[(m - 1) as usize];
    let mn = format!("{}{}", mn[..1].to_uppercase(), &mn[1..]);
    format!("{wd}, {d} {mn} {y}")
}

/// City / zone → fixed UTC offset in hours (standard time, ignores DST).
fn tz_offset(place: &str) -> Option<f64> {
    let p = place.trim();
    Some(match p {
        "utc" | "gmt" => 0.0,
        "london" | "uk" | "lisbon" | "dublin" => 0.0,
        "paris" | "berlin" | "madrid" | "rome" | "amsterdam" | "frankfurt"
        | "zurich" | "vienna" | "brussels" | "milan" | "cet" => 1.0,
        "cairo" | "athens" | "helsinki" | "johannesburg" => 2.0,
        "moscow" | "istanbul" | "nairobi" | "riyadh" => 3.0,
        "dubai" | "abu dhabi" | "baku" => 4.0,
        "karachi" | "islamabad" | "lahore" | "tashkent" => 5.0,
        "delhi" | "new delhi" | "mumbai" | "india" | "bangalore" | "kolkata" => 5.5,
        "dhaka" | "almaty" => 6.0,
        "bangkok" | "jakarta" | "hanoi" => 7.0,
        "beijing" | "shanghai" | "china" | "singapore" | "hong kong"
        | "manila" | "perth" | "taipei" => 8.0,
        "tokyo" | "japan" | "seoul" | "osaka" => 9.0,
        "sydney" | "melbourne" | "brisbane" => 10.0,
        "auckland" | "wellington" => 12.0,
        "honolulu" | "hawaii" => -10.0,
        "anchorage" | "alaska" => -9.0,
        "los angeles" | "la" | "san francisco" | "sf" | "seattle"
        | "vancouver" | "pst" | "pt" => -8.0,
        "denver" | "phoenix" | "mst" => -7.0,
        "chicago" | "dallas" | "houston" | "mexico city" | "cst" => -6.0,
        "new york" | "nyc" | "new york city" | "toronto" | "boston"
        | "miami" | "washington" | "est" | "et" => -5.0,
        "sao paulo" | "brasilia" | "buenos aires" => -3.0,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn v(q: &str) -> String {
        let r = HashMap::new();
        evaluate(q, &r).expect("should parse").value
    }
    #[test]
    fn math() {
        assert_eq!(v("2+2*3"), "8");
        assert_eq!(v("(3+4)*2"), "14");
        assert_eq!(v("2^10"), "1,024");
        assert_eq!(v("sqrt(144)"), "12");
        assert_eq!(v("15% of 200"), "30");
        assert_eq!(v("1200 - 10%"), "1,080");
        assert_eq!(v("10 / 4"), "2.5");
    }
    #[test]
    fn units() {
        assert_eq!(v("100 f to c"), "37.777778 °C");
        assert_eq!(v("0 c to f"), "32 °F");
        assert_eq!(v("1 km to m"), "1,000 m");
        assert_eq!(v("2 cups to ml"), "473.176473 ml");
        assert_eq!(v("60 mph to kph"), "96.56064 kph");
    }
    #[test]
    fn currency() {
        let mut r = HashMap::new();
        r.insert("USD".to_string(), 1.0);
        r.insert("EUR".to_string(), 0.9);
        assert_eq!(evaluate("100 usd to eur", &r).unwrap().value, "90 EUR");
    }
    #[test]
    fn clock() {
        assert_eq!(parse_tod("930 am"), Some((9 * 60 + 30, true)));
        assert_eq!(parse_tod("9:30"), Some((9 * 60 + 30, true)));
        assert_eq!(parse_tod("21:00"), Some((21 * 60, true)));
        assert_eq!(parse_tod("12 am"), Some((0, true)));
        assert_eq!(parse_tod("12 pm"), Some((12 * 60, true)));
        assert_eq!(parse_tod("0930"), Some((9 * 60 + 30, false)));
        assert_eq!(parse_tod("930"), Some((9 * 60 + 30, false)));
        assert_eq!(parse_tod("spotify"), None);
        // bare normalisation
        assert_eq!(v("930 am"), "9:30 AM");
        assert_eq!(v("9:30"), "9:30 AM");
        assert_eq!(v("21:00"), "9:00 PM");
        // explicit-zone conversion (no local clock needed)
        let r = HashMap::new();
        let h = evaluate("9 am london to tokyo", &r).unwrap();
        assert_eq!(h.value, "6:00 PM");
        assert_eq!(h.label, "9:00 AM London \u{2192} Tokyo");
        let h2 = evaluate("11 pm new york to london", &r).unwrap();
        assert_eq!(h2.value, "4:00 AM (+1 day)");
    }
    #[test]
    fn not_a_calc() {
        let r = HashMap::new();
        assert!(evaluate("spotify", &r).is_none());
        assert!(evaluate("log in", &r).is_none());
        assert!(evaluate("design to code", &r).is_none());
    }
}

fn cap_words(s: &str) -> String {
    s.split(' ')
        .filter(|w| !w.is_empty())
        .map(|w| format!("{}{}", w[..1].to_uppercase(), &w[1..]))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The machine's own UTC offset, in hours (honours real local DST).
fn local_offset() -> f64 {
    -js_sys::Date::new_0().get_timezone_offset() / 60.0
}

fn offset_of(place: &str) -> Option<f64> {
    let p = place.trim();
    if p.is_empty() || p == "here" || p == "local" || p == "me" {
        Some(local_offset())
    } else {
        tz_offset(p)
    }
}

/// Parse a clock time: "9:30", "0930", "930am", "9 pm", "21:00".
/// Returns (minutes-since-midnight, strong) where `strong` means the input
/// was unambiguously a time (had a colon or am/pm).
fn parse_tod(s: &str) -> Option<(i64, bool)> {
    let s = s.trim().to_lowercase();
    let pm = s.contains("pm") || s.contains("p.m");
    let am = s.contains("am") || s.contains("a.m");
    let core: String = s.chars().filter(|c| c.is_ascii_digit() || *c == ':').collect();
    if core.is_empty() {
        return None;
    }
    let had_colon = core.contains(':');
    let (mut h, mi) = if had_colon {
        let mut it = core.split(':');
        let h: i64 = it.next()?.parse().ok()?;
        let mi: i64 = match it.next() {
            Some(x) if !x.is_empty() => x.parse().ok()?,
            _ => 0,
        };
        (h, mi)
    } else {
        let d = &core;
        match d.len() {
            1 | 2 => (d.parse().ok()?, 0),
            3 => (d[..1].parse().ok()?, d[1..].parse().ok()?),
            4 => (d[..2].parse().ok()?, d[2..].parse().ok()?),
            _ => return None,
        }
    };
    if pm && h != 12 {
        h += 12;
    }
    if am && h == 12 {
        h = 0;
    }
    if !(0..=23).contains(&h) || !(0..=59).contains(&mi) {
        return None;
    }
    Some((h * 60 + mi, had_colon || am || pm))
}

fn fmt_12h(min: i64) -> String {
    let min = min.rem_euclid(1440);
    let (h24, mm) = (min / 60, min % 60);
    let ap = if h24 < 12 { "AM" } else { "PM" };
    let h12 = match h24 % 12 {
        0 => 12,
        x => x,
    };
    format!("{}:{:02} {}", h12, mm, ap)
}

/// "<time> [<place>|here] to/in <place>" timezone conversion, or a bare
/// "<time>" which is just normalised to a 12-hour clock.
fn clock_calc(lc: &str) -> Option<CalcHit> {
    if let Some((lhs, rhs)) = split_connector(lc) {
        let to = rhs.trim().trim_end_matches('?').trim();
        let to_off = offset_of(to)?;
        // Split lhs into a time part and an optional source place by trying
        // every boundary (place may be multiple words, e.g. "new york").
        let words: Vec<&str> = lhs.split_whitespace().collect();
        for k in 1..=words.len() {
            let time_str = words[..k].join(" ");
            let place = words[k..].join(" ");
            let Some((mins, strong)) = parse_tod(&time_str) else {
                continue;
            };
            if !strong && place.is_empty() {
                continue; // need am/pm/colon, or an explicit source place
            }
            let Some(from_off) = offset_of(&place) else {
                continue;
            };
            let total = mins + ((to_off - from_off) * 60.0).round() as i64;
            let day = total.div_euclid(1440);
            let shift = match day {
                d if d > 0 => format!(" (+{d} day)"),
                d if d < 0 => format!(" (\u{2212}{} day)", -d),
                _ => String::new(),
            };
            let src_name = if place.is_empty() || place == "here" {
                "here".to_string()
            } else {
                cap_words(&place)
            };
            return Some(CalcHit {
                value: format!("{}{}", fmt_12h(total), shift),
                label: format!(
                    "{} {} \u{2192} {}",
                    fmt_12h(mins),
                    src_name,
                    cap_words(to)
                ),
                kind: CalcKind::Time,
            });
        }
        return None;
    }
    // Bare time: normalise "930 am" / "9:30" → "9:30 AM".
    let (mins, strong) = parse_tod(lc)?;
    if !strong {
        return None;
    }
    Some(CalcHit {
        value: fmt_12h(mins),
        label: "Time".into(),
        kind: CalcKind::Time,
    })
}

fn time_calc(q: &str) -> Option<CalcHit> {
    let lc = q.to_lowercase();
    let lc = lc.trim();

    // "time in <place>" / "what time is it in <place>"
    if let Some(idx) = lc.find("time in ") {
        let place = lc[idx + 8..].trim().trim_end_matches('?').trim();
        let off = offset_of(place)?;
        let local_ms = now_ms() + off * 3_600_000.0;
        let day = (local_ms / 86_400_000.0).floor() as i64;
        let day_ms = local_ms - (day as f64) * 86_400_000.0;
        let total_min = (day_ms / 60_000.0).floor() as i64;
        let (h, mm) = (total_min / 60, total_min % 60);
        let suffix = if off.fract() == 0.0 {
            format!("UTC{:+}", off as i64)
        } else {
            format!("UTC{:+.1}", off)
        };
        return Some(CalcHit {
            value: format!("{:02}:{:02}", h, mm),
            label: format!("{} · {} · {suffix}", cap_words(place), fmt_date(day)),
            kind: CalcKind::Time,
        });
    }

    // Clock-time normalisation and timezone conversion.
    if let Some(hit) = clock_calc(lc) {
        return Some(hit);
    }

    // "days until / since <date>"
    let after_kw = ["until ", "till ", "til ", "since "]
        .iter()
        .find_map(|kw| lc.split(kw).nth(1));
    if let Some(rest) = after_kw {
        let target = parse_date(rest.trim())?;
        let diff = target - today_days();
        let n = diff.abs();
        let unit = if n == 1 { "day" } else { "days" };
        let dir = if diff >= 0 { "from now" } else { "ago" };
        return Some(CalcHit {
            value: format!("{} {}", n, unit),
            label: format!("{} ({})", fmt_date(target), dir),
            kind: CalcKind::Time,
        });
    }

    // Explicit date range "A to B" / "A - B" — only if both sides are dates.
    let sep = if lc.contains(" to ") {
        " to "
    } else if lc.contains(" until ") {
        " until "
    } else {
        return None;
    };
    let (a, b) = lc.split_once(sep)?;
    let da = parse_date(a)?;
    let db = parse_date(b)?;
    let diff = (db - da).abs();
    Some(CalcHit {
        value: format!("{} {}", diff, if diff == 1 { "day" } else { "days" }),
        label: format!("{} → {}", fmt_date(da.min(db)), fmt_date(da.max(db))),
        kind: CalcKind::Time,
    })
}
