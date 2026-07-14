use serde_json::Value;

pub enum Format {
    Table,
    Json,
    Plain,
}

impl Format {
    pub fn from_args(json: bool, plain: bool) -> Self {
        if json {
            Format::Json
        } else if plain {
            Format::Plain
        } else {
            Format::Table
        }
    }
}

pub fn print_value(value: &Value, format: &Format) {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(value).unwrap()),
        Format::Plain => print_plain(value),
        Format::Table => print_table(value),
    }
}

fn print_plain(value: &Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                println!("{}: {}", k, val_to_string(v));
            }
        }
        Value::Array(arr) => {
            for item in arr {
                print_plain(item);
                println!();
            }
        }
        other => println!("{}", val_to_string(other)),
    }
}

fn print_table(value: &Value) {
    match value {
        Value::Array(arr) if arr.first().map(|v| v.is_object()).unwrap_or(false) => {
            let keys: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_object())
                .flat_map(|m| m.keys().cloned())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            if keys.is_empty() {
                return;
            }

            let col_widths: Vec<usize> = keys
                .iter()
                .map(|k| {
                    let max_data = arr
                        .iter()
                        .filter_map(|v| v.get(k))
                        .map(|v| val_to_string(v).len())
                        .max()
                        .unwrap_or(0);
                    k.len().max(max_data)
                })
                .collect();

            let header: String = keys
                .iter()
                .enumerate()
                .map(|(i, k)| format!("{:width$}", k, width = col_widths[i].min(30)))
                .collect::<Vec<_>>()
                .join("  ");
            println!("{}", header);
            println!("{}", "-".repeat(header.len()));

            for item in arr {
                let row: String = keys
                    .iter()
                    .enumerate()
                    .map(|(i, k)| {
                        let v = item.get(k).map(val_to_string).unwrap_or_default();
                        format!("{:width$}", v, width = col_widths[i].min(30))
                    })
                    .collect::<Vec<_>>()
                    .join("  ");
                println!("{}", row);
            }
        }
        Value::Object(map) => {
            let max_key = map.keys().map(|k| k.len()).max().unwrap_or(0);
            for (k, v) in map {
                println!("{:width$}  {}", k, val_to_string(v), width = max_key);
            }
        }
        other => println!("{}", val_to_string(other)),
    }
}

fn val_to_string(v: &Value) -> String {
    match v {
        Value::Null => "".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(val_to_string).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(_) => "...".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_from_args_default() {
        let f = Format::from_args(false, false);
        assert!(matches!(f, Format::Table));
    }

    #[test]
    fn test_format_from_args_json() {
        let f = Format::from_args(true, false);
        assert!(matches!(f, Format::Json));
    }

    #[test]
    fn test_format_from_args_plain() {
        let f = Format::from_args(false, true);
        assert!(matches!(f, Format::Plain));
    }

    #[test]
    fn test_val_to_string_null() {
        assert_eq!(val_to_string(&Value::Null), "");
    }

    #[test]
    fn test_val_to_string_number() {
        assert_eq!(val_to_string(&json!(42)), "42");
    }

    #[test]
    fn test_val_to_string_string() {
        assert_eq!(val_to_string(&json!("hello")), "hello");
    }

    #[test]
    fn test_val_to_string_array() {
        assert_eq!(val_to_string(&json!([1, 2, 3])), "[1, 2, 3]");
    }

    #[test]
    fn test_val_to_string_object() {
        assert_eq!(val_to_string(&json!({"a": 1})), "...");
    }
}
