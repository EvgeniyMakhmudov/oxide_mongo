use std::str::FromStr;

use mongodb::bson::{Bson, DateTime, Decimal128, oid::ObjectId};

use crate::{i18n::tr, mongo::shell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueEditKind {
    String,
    Boolean,
    Int32,
    Int64,
    Double,
    Decimal128,
    DateTime,
    ObjectId,
    Null,
    Document,
    Array,
    Binary,
    Regex,
    Code,
    CodeWithScope,
    Timestamp,
    DbPointer,
    MinKey,
    MaxKey,
    Undefined,
    Other,
}

impl ValueEditKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::String => tr("String"),
            Self::Boolean => tr("Boolean"),
            Self::Int32 => tr("Int32"),
            Self::Int64 => tr("Int64"),
            Self::Double => tr("Double"),
            Self::Decimal128 => tr("Decimal128"),
            Self::DateTime => tr("DateTime"),
            Self::ObjectId => tr("ObjectId"),
            Self::Null => tr("Null"),
            Self::Document => tr("Document"),
            Self::Array => tr("Array"),
            Self::Binary => tr("Binary"),
            Self::Regex => tr("Regex"),
            Self::Code => tr("Code"),
            Self::CodeWithScope => tr("CodeWithScope"),
            Self::Timestamp => tr("Timestamp"),
            Self::DbPointer => tr("DbPointer"),
            Self::MinKey => tr("MinKey"),
            Self::MaxKey => tr("MaxKey"),
            Self::Undefined => tr("Undefined"),
            Self::Other => tr("Other"),
        }
    }

    pub fn infer(input: &str) -> Option<Self> {
        if let Ok(bson) = shell::parse_shell_bson_value(input) {
            return Some(Self::from_bson(&bson));
        }

        let trimmed = input.trim();

        if trimmed.is_empty() {
            return None;
        }

        if trimmed.eq_ignore_ascii_case("null") {
            return Some(Self::Null);
        }

        if Self::parse_boolean_literal(trimmed).is_some() {
            return Some(Self::Boolean);
        }

        if Self::parse_object_id_literal(trimmed).is_ok() {
            return Some(Self::ObjectId);
        }

        if Self::parse_datetime_literal(trimmed).is_ok() {
            return Some(Self::DateTime);
        }

        let has_decimal_wrapper =
            Self::strip_call(trimmed, &["NumberDecimal", "numberDecimal"]).is_some();
        if has_decimal_wrapper && Self::parse_decimal_literal(trimmed).is_ok() {
            return Some(Self::Decimal128);
        }

        let has_double_wrapper =
            Self::strip_call(trimmed, &["NumberDouble", "numberDouble"]).is_some();
        let looks_like_float = has_double_wrapper
            || trimmed.contains('.')
            || trimmed.contains('e')
            || trimmed.contains('E');
        if looks_like_float && Self::parse_double_literal(trimmed).is_ok() {
            return Some(Self::Double);
        }

        if let Ok(value) = Self::parse_int_literal(trimmed) {
            if value >= i32::MIN as i128 && value <= i32::MAX as i128 {
                return Some(Self::Int32);
            } else {
                return Some(Self::Int64);
            }
        }

        Some(Self::String)
    }

    pub fn from_bson(bson: &Bson) -> Self {
        match bson {
            Bson::String(_) => Self::String,
            Bson::Boolean(_) => Self::Boolean,
            Bson::Int32(_) => Self::Int32,
            Bson::Int64(_) => Self::Int64,
            Bson::Double(_) => Self::Double,
            Bson::Decimal128(_) => Self::Decimal128,
            Bson::DateTime(_) => Self::DateTime,
            Bson::ObjectId(_) => Self::ObjectId,
            Bson::Null => Self::Null,
            Bson::Document(_) => Self::Document,
            Bson::Array(_) => Self::Array,
            Bson::Binary(_) => Self::Binary,
            Bson::RegularExpression(_) => Self::Regex,
            Bson::JavaScriptCode(_) => Self::Code,
            Bson::JavaScriptCodeWithScope(_) => Self::CodeWithScope,
            Bson::Timestamp(_) => Self::Timestamp,
            Bson::DbPointer(_) => Self::DbPointer,
            Bson::MinKey => Self::MinKey,
            Bson::MaxKey => Self::MaxKey,
            Bson::Undefined => Self::Undefined,
            _ => Self::Other,
        }
    }

    pub fn parse(self, input: &str) -> Result<Bson, String> {
        if let Ok(bson) = shell::parse_shell_bson_value(input) {
            return Ok(bson);
        }

        match self {
            Self::String => Ok(Bson::String(Self::parse_string_literal(input))),
            Self::Boolean => Ok(Bson::Boolean(Self::parse_boolean_literal(input).unwrap_or(false))),
            Self::Int32 => Self::parse_int32_value(input),
            Self::Int64 => Self::parse_int64_value(input),
            Self::Double => Ok(Bson::Double(Self::parse_double_literal(input)?)),
            Self::Decimal128 => Ok(Bson::Decimal128(Self::parse_decimal_literal(input)?)),
            Self::DateTime => Ok(Bson::DateTime(Self::parse_datetime_literal(input)?)),
            Self::ObjectId => Ok(Bson::ObjectId(Self::parse_object_id_literal(input)?)),
            Self::Null => Ok(Bson::Null),
            Self::Document => shell::parse_shell_document(input),
            Self::Array => shell::parse_shell_array(input),
            Self::Binary => Err(String::from(tr("Binary parsing not implemented."))),
            Self::Regex => Err(String::from(tr("Regex parsing not implemented."))),
            Self::Code => Err(String::from(tr("Code parsing not implemented."))),
            Self::CodeWithScope => Err(String::from(tr("CodeWithScope parsing not implemented."))),
            Self::Timestamp => Err(String::from(tr("Timestamp parsing not implemented."))),
            Self::DbPointer => Err(String::from(tr("DbPointer parsing not implemented."))),
            Self::MinKey => Ok(Bson::MinKey),
            Self::MaxKey => Ok(Bson::MaxKey),
            Self::Undefined => Ok(Bson::Undefined),
            Self::Other => Err(String::from(tr("Cannot parse value of this type."))),
        }
    }

    fn parse_string_literal(input: &str) -> String {
        Self::trim_quotes(input).unwrap_or(input.trim()).to_string()
    }

    fn parse_boolean_literal(input: &str) -> Option<bool> {
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("true") {
            Some(true)
        } else if trimmed.eq_ignore_ascii_case("false") {
            Some(false)
        } else {
            None
        }
    }

    fn parse_int32_value(input: &str) -> Result<Bson, String> {
        let literal = Self::extract_numeric_literal(input, &["NumberInt", "numberInt"])
            .unwrap_or_else(|| input.trim().to_string());

        literal
            .parse::<i32>()
            .map(Bson::Int32)
            .map_err(|_| String::from(tr("Value must be a 32-bit integer.")))
    }

    fn parse_int64_value(input: &str) -> Result<Bson, String> {
        let literal = Self::extract_numeric_literal(input, &["NumberLong", "numberLong"])
            .unwrap_or_else(|| input.trim().to_string());

        literal
            .parse::<i64>()
            .map(Bson::Int64)
            .map_err(|_| String::from(tr("Value must be a 64-bit integer.")))
    }

    fn parse_double_literal(input: &str) -> Result<f64, String> {
        let literal = Self::extract_numeric_literal(input, &["NumberDouble", "numberDouble"])
            .unwrap_or_else(|| input.trim().to_string());

        literal.parse::<f64>().map_err(|_| String::from(tr("Value must be a Double.")))
    }

    fn parse_decimal_literal(input: &str) -> Result<Decimal128, String> {
        let literal = Self::extract_numeric_literal(input, &["NumberDecimal", "numberDecimal"])
            .unwrap_or_else(|| input.trim().to_string());

        Decimal128::from_str(literal.trim())
            .map_err(|_| String::from(tr("Value must be a Decimal128.")))
    }

    fn parse_datetime_literal(input: &str) -> Result<DateTime, String> {
        if let Some(argument) = Self::strip_call(input, &["ISODate", "Date"]) {
            return Self::coerce_datetime(argument);
        }

        Self::coerce_datetime(input)
    }

    fn parse_object_id_literal(input: &str) -> Result<ObjectId, String> {
        let literal = if let Some(argument) = Self::strip_call(input, &["ObjectId"]) {
            Self::trim_quotes(argument).unwrap_or(argument.trim()).to_string()
        } else {
            input.trim().to_string()
        };

        ObjectId::parse_str(literal).map_err(|_| String::from(tr("Value must be an ObjectId.")))
    }

    fn parse_int_literal(input: &str) -> Result<i128, String> {
        let literal = Self::extract_numeric_literal(
            input,
            &["NumberInt", "numberInt", "NumberLong", "numberLong"],
        )
        .unwrap_or_else(|| input.trim().to_string());

        literal.parse::<i128>().map_err(|_| String::from(tr("Value must be an integer.")))
    }

    fn coerce_datetime(input: &str) -> Result<DateTime, String> {
        let literal = Self::trim_quotes(input).unwrap_or(input.trim());

        if let Ok(dt) = DateTime::parse_rfc3339_str(literal) {
            return Ok(dt);
        }

        let millis: i64 = literal.parse().map_err(|_| {
            String::from(tr("Enter an ISO 8601 date or milliseconds since the epoch."))
        })?;
        Ok(DateTime::from_millis(millis))
    }

    fn extract_numeric_literal(input: &str, names: &[&str]) -> Option<String> {
        Self::strip_call(input, names).map(|s| s.trim().to_string())
    }

    fn strip_call<'a>(input: &'a str, names: &[&str]) -> Option<&'a str> {
        let trimmed = input.trim();

        for name in names {
            if trimmed.starts_with(name) && trimmed.ends_with(')') {
                let start = trimmed.find('(').unwrap_or(0);
                let end = trimmed.rfind(')').unwrap_or(trimmed.len());
                return Some(&trimmed[start + 1..end]);
            }
        }

        None
    }

    fn trim_quotes(input: &str) -> Option<&str> {
        let trimmed = input.trim();
        if trimmed.len() >= 2 {
            if trimmed.starts_with('"') && trimmed.ends_with('"') {
                return Some(&trimmed[1..trimmed.len() - 1]);
            }
            if trimmed.starts_with('\'') && trimmed.ends_with('\'') {
                return Some(&trimmed[1..trimmed.len() - 1]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_infer(input: &str, expected: ValueEditKind) {
        assert_eq!(ValueEditKind::infer(input), Some(expected));
    }

    #[test]
    fn infer_handles_basic_literals() {
        assert_infer("42", ValueEditKind::Int32);
        assert_infer("NumberLong(9007199254740991)", ValueEditKind::Int64);
        assert_infer("NumberDouble(3.14)", ValueEditKind::Double);
        assert_infer("NumberDecimal(\"1.23\")", ValueEditKind::Decimal128);
        assert_infer("true", ValueEditKind::Boolean);
        assert_infer("ISODate(\"2024-01-01T00:00:00Z\")", ValueEditKind::DateTime);
        assert_infer("ObjectId(\"64d2f9f18d964a7848d35300\")", ValueEditKind::ObjectId);
        assert_infer("null", ValueEditKind::Null);
        assert_infer("{ \"name\": \"Alice\" }", ValueEditKind::Document);
        assert_infer("[1, 2, 3]", ValueEditKind::Array);
    }

    #[test]
    fn infer_falls_back_to_string() {
        assert_eq!(ValueEditKind::infer("  hello  "), Some(ValueEditKind::String));
    }

    #[test]
    fn parse_respects_explicit_kind() {
        let string_value = ValueEditKind::String.parse("'text'").unwrap();
        assert_eq!(string_value, Bson::String(String::from("text")));

        let int32_value = ValueEditKind::Int32.parse("NumberInt(123)").unwrap();
        assert_eq!(int32_value, Bson::Int32(123));

        let int64_value = ValueEditKind::Int64.parse("NumberLong(12345678900)").unwrap();
        assert_eq!(int64_value, Bson::Int64(12_345_678_900));

        let double_value = ValueEditKind::Double.parse("NumberDouble(3.14)").unwrap();
        assert_eq!(double_value, Bson::Double(3.14));

        let decimal_value = ValueEditKind::Decimal128.parse("NumberDecimal(\"1.23\")").unwrap();
        assert_eq!(decimal_value, Bson::Decimal128(Decimal128::from_str("1.23").unwrap()));

        let date_value =
            ValueEditKind::DateTime.parse("ISODate(\"2024-01-01T00:00:00Z\")").unwrap();
        assert_eq!(
            date_value,
            Bson::DateTime(DateTime::parse_rfc3339_str("2024-01-01T00:00:00Z").unwrap())
        );

        let object_id_value =
            ValueEditKind::ObjectId.parse("ObjectId(\"64d2f9f18d964a7848d35300\")").unwrap();
        assert_eq!(
            object_id_value,
            Bson::ObjectId(ObjectId::parse_str("64d2f9f18d964a7848d35300").unwrap())
        );

        let document_source = "{ \"name\": \"Alice\" }";
        let document_value = ValueEditKind::Document.parse(document_source).unwrap();
        assert_eq!(document_value, shell::parse_shell_document(document_source).unwrap());

        let array_value = ValueEditKind::Array.parse("[1, 2]").unwrap();
        assert_eq!(array_value, shell::parse_shell_array("[1, 2]").unwrap());
    }

    #[test]
    fn parse_reports_errors_for_invalid_input() {
        let err = ValueEditKind::Int32.parse("abc").unwrap_err();
        assert!(err.contains("32-bit"));

        let err = ValueEditKind::ObjectId.parse("ObjectId(\"1234\")").unwrap_err();
        assert!(err.contains("ObjectId"));

        let err = ValueEditKind::Decimal128.parse("NumberDecimal(\"not-a-number\")").unwrap_err();
        assert!(err.contains("Decimal128"));

        let err = ValueEditKind::Binary.parse("abc").unwrap_err();
        assert!(err.contains("not implemented"));

        let err = ValueEditKind::Int64.parse("NumberLong(text)").unwrap_err();
        assert!(err.contains("64-bit"));
    }
}
