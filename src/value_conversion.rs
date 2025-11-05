use serde_json::Value;

/// Converts a field value to appropriate JSON Value based on type inference
pub fn convert_field_value(
    field: &str,
    header_name: &str,
    no_type_conversion: bool,
    string_fields: &[String],
) -> Value {
    if no_type_conversion {
        // No type conversion - keep everything as strings except empty fields
        if field.is_empty() {
            Value::Null
        } else {
            Value::String(field.to_string())
        }
    } else if string_fields.iter().any(|f| f == header_name) {
        // Field is in the string_fields list - always keep as string
        if field.is_empty() {
            Value::Null
        } else {
            Value::String(field.to_string())
        }
    } else {
        // Smart type conversion, but preserve leading zeros (zipcodes, phone numbers, etc)
        let has_leading_zero =
            field.starts_with('0') && field.len() > 1 && !field.starts_with("0.");

        if field.is_empty() {
            Value::Null
        } else if field.eq_ignore_ascii_case("true") {
            Value::Bool(true)
        } else if field.eq_ignore_ascii_case("false") {
            Value::Bool(false)
        } else if !has_leading_zero {
            // Only try to parse as number if no leading zero
            if let Ok(num) = field.parse::<i64>() {
                Value::Number(num.into())
            } else if let Ok(num) = field.parse::<f64>() {
                if let Some(n) = serde_json::Number::from_f64(num) {
                    Value::Number(n)
                } else {
                    Value::String(field.to_string())
                }
            } else {
                Value::String(field.to_string())
            }
        } else {
            // Has leading zero - keep as string to preserve it
            Value::String(field.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_field_value_integers() {
        let value = convert_field_value("42", "age", false, &[]);
        assert!(matches!(value, Value::Number(_)));
        if let Value::Number(n) = value {
            assert_eq!(n.as_i64(), Some(42));
        }
    }

    #[test]
    fn test_convert_field_value_floats() {
        let value = convert_field_value("3.14", "price", false, &[]);
        assert!(matches!(value, Value::Number(_)));
        if let Value::Number(n) = value {
            assert_eq!(n.as_f64(), Some(3.14));
        }
    }

    #[test]
    fn test_convert_field_value_booleans() {
        let value_true = convert_field_value("true", "active", false, &[]);
        assert_eq!(value_true, Value::Bool(true));

        let value_false = convert_field_value("FALSE", "active", false, &[]);
        assert_eq!(value_false, Value::Bool(false));
    }

    #[test]
    fn test_convert_field_value_leading_zeros() {
        let value = convert_field_value("02134", "zipcode", false, &[]);
        assert_eq!(value, Value::String("02134".to_string()));
    }

    #[test]
    fn test_convert_field_value_decimal_leading_zero() {
        let value = convert_field_value("0.5", "score", false, &[]);
        assert!(matches!(value, Value::Number(_)));
    }

    #[test]
    fn test_convert_field_value_empty_to_null() {
        let value = convert_field_value("", "field", false, &[]);
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn test_convert_field_value_string_fields() {
        let string_fields = vec!["zipcode".to_string()];
        let value = convert_field_value("12345", "zipcode", false, &string_fields);
        assert_eq!(value, Value::String("12345".to_string()));
    }

    #[test]
    fn test_convert_field_value_no_type_conversion() {
        let value = convert_field_value("42", "age", true, &[]);
        assert_eq!(value, Value::String("42".to_string()));

        let value = convert_field_value("true", "active", true, &[]);
        assert_eq!(value, Value::String("true".to_string()));
    }

    #[test]
    fn test_convert_field_value_strings() {
        let value = convert_field_value("Hello World", "name", false, &[]);
        assert_eq!(value, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_convert_field_value_negative_numbers() {
        let value = convert_field_value("-42", "temp", false, &[]);
        assert!(matches!(value, Value::Number(_)));
        if let Value::Number(n) = value {
            assert_eq!(n.as_i64(), Some(-42));
        }
    }
}
