//! Diff generation for audit logging
//!
//! Generates human-readable diffs between before and after values
//! for audit log entries.

use serde_json::Value;

/// Generate a human-readable diff between two JSON values
///
/// Returns a string describing the changes in a user-friendly format.
/// Only includes top-level field changes for readability.
pub fn generate_diff(before: &Value, after: &Value) -> Option<String> {
    match (before, after) {
        (Value::Object(before_obj), Value::Object(after_obj)) => {
            let mut changes = Vec::new();

            // Check for modified and removed fields
            for (key, before_val) in before_obj {
                if let Some(after_val) = after_obj.get(key) {
                    if before_val != after_val {
                        changes.push(format!(
                            "{}: {} -> {}",
                            key,
                            format_value(before_val),
                            format_value(after_val)
                        ));
                    }
                } else {
                    changes.push(format!(
                        "{}: {} -> (removed)",
                        key,
                        format_value(before_val)
                    ));
                }
            }

            // Check for added fields
            for (key, after_val) in after_obj {
                if !before_obj.contains_key(key) {
                    changes.push(format!("{}: (added) -> {}", key, format_value(after_val)));
                }
            }

            if changes.is_empty() {
                None
            } else {
                Some(changes.join(", "))
            }
        }
        _ => {
            // For non-object values, just show the change
            if before != after {
                Some(format!(
                    "{} -> {}",
                    format_value(before),
                    format_value(after)
                ))
            } else {
                None
            }
        }
    }
}

/// Format a JSON value for human-readable display
fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            // Truncate long strings
            if s.len() > 50 {
                format!("\"{}...\"", &s[..47])
            } else {
                format!("\"{}\"", s)
            }
        }
        Value::Array(arr) => format!("[{} items]", arr.len()),
        Value::Object(obj) => format!("{{{} fields}}", obj.len()),
    }
}

/// Generate a detailed diff that includes nested changes
///
/// More verbose than `generate_diff`, useful for detailed auditing.
pub fn generate_detailed_diff(before: &Value, after: &Value, prefix: &str) -> Vec<String> {
    let mut changes = Vec::new();

    match (before, after) {
        (Value::Object(before_obj), Value::Object(after_obj)) => {
            // Check for modified and removed fields
            for (key, before_val) in before_obj {
                let field_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };

                if let Some(after_val) = after_obj.get(key) {
                    if before_val != after_val {
                        // Recurse for nested objects
                        if before_val.is_object() && after_val.is_object() {
                            changes.extend(generate_detailed_diff(
                                before_val,
                                after_val,
                                &field_prefix,
                            ));
                        } else {
                            changes.push(format!(
                                "{}: {} -> {}",
                                field_prefix,
                                format_value(before_val),
                                format_value(after_val)
                            ));
                        }
                    }
                } else {
                    changes.push(format!(
                        "{}: {} -> (removed)",
                        field_prefix,
                        format_value(before_val)
                    ));
                }
            }

            // Check for added fields
            for (key, after_val) in after_obj {
                if !before_obj.contains_key(key) {
                    let field_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    changes.push(format!(
                        "{}: (added) -> {}",
                        field_prefix,
                        format_value(after_val)
                    ));
                }
            }
        }
        (Value::Array(before_arr), Value::Array(after_arr)) => {
            if before_arr.len() != after_arr.len() {
                changes.push(format!(
                    "{}: [{} items] -> [{} items]",
                    prefix,
                    before_arr.len(),
                    after_arr.len()
                ));
            } else {
                for (i, (b, a)) in before_arr.iter().zip(after_arr.iter()).enumerate() {
                    if b != a {
                        let item_prefix = format!("{}[{}]", prefix, i);
                        changes.extend(generate_detailed_diff(b, a, &item_prefix));
                    }
                }
            }
        }
        _ => {
            if before != after {
                changes.push(format!(
                    "{}: {} -> {}",
                    prefix,
                    format_value(before),
                    format_value(after)
                ));
            }
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_field_change() {
        let before = json!({"name": "Checking", "balance": 1000});
        let after = json!({"name": "Checking", "balance": 1500});

        let diff = generate_diff(&before, &after).unwrap();
        assert!(diff.contains("balance: 1000 -> 1500"));
        assert!(!diff.contains("name")); // unchanged field
    }

    #[test]
    fn test_string_field_change() {
        let before = json!({"name": "Old Name"});
        let after = json!({"name": "New Name"});

        let diff = generate_diff(&before, &after).unwrap();
        assert!(diff.contains("name: \"Old Name\" -> \"New Name\""));
    }

    #[test]
    fn test_field_added() {
        let before = json!({"name": "Test"});
        let after = json!({"name": "Test", "balance": 100});

        let diff = generate_diff(&before, &after).unwrap();
        assert!(diff.contains("balance: (added) -> 100"));
    }

    #[test]
    fn test_field_removed() {
        let before = json!({"name": "Test", "old_field": "value"});
        let after = json!({"name": "Test"});

        let diff = generate_diff(&before, &after).unwrap();
        assert!(diff.contains("old_field: \"value\" -> (removed)"));
    }

    #[test]
    fn test_no_changes() {
        let before = json!({"name": "Test", "value": 100});
        let after = json!({"name": "Test", "value": 100});

        let diff = generate_diff(&before, &after);
        assert!(diff.is_none());
    }

    #[test]
    fn test_multiple_changes() {
        let before = json!({"a": 1, "b": 2, "c": 3});
        let after = json!({"a": 10, "b": 2, "c": 30});

        let diff = generate_diff(&before, &after).unwrap();
        assert!(diff.contains("a: 1 -> 10"));
        assert!(diff.contains("c: 3 -> 30"));
        assert!(!diff.contains("b:")); // unchanged
    }

    #[test]
    fn test_bool_change() {
        let before = json!({"active": true});
        let after = json!({"active": false});

        let diff = generate_diff(&before, &after).unwrap();
        assert!(diff.contains("active: true -> false"));
    }

    #[test]
    fn test_null_handling() {
        let before = json!({"value": null});
        let after = json!({"value": 100});

        let diff = generate_diff(&before, &after).unwrap();
        assert!(diff.contains("value: null -> 100"));
    }

    #[test]
    fn test_array_change_summary() {
        let before = json!({"items": [1, 2, 3]});
        let after = json!({"items": [1, 2, 3, 4, 5]});

        let diff = generate_diff(&before, &after).unwrap();
        assert!(diff.contains("items: [3 items] -> [5 items]"));
    }

    #[test]
    fn test_detailed_diff_nested() {
        let before = json!({"account": {"name": "Old", "balance": 100}});
        let after = json!({"account": {"name": "New", "balance": 100}});

        let changes = generate_detailed_diff(&before, &after, "");
        assert!(changes.iter().any(|c| c.contains("account.name")));
    }

    #[test]
    fn test_long_string_truncation() {
        let long_string = "a".repeat(100);
        let before = json!({"memo": long_string});
        let after = json!({"memo": "short"});

        let diff = generate_diff(&before, &after).unwrap();
        assert!(diff.contains("...\""));
    }

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(&json!(null)), "null");
        assert_eq!(format_value(&json!(true)), "true");
        assert_eq!(format_value(&json!(42)), "42");
        assert_eq!(format_value(&json!("test")), "\"test\"");
        assert_eq!(format_value(&json!([1, 2, 3])), "[3 items]");
        assert_eq!(format_value(&json!({"a": 1, "b": 2})), "{2 fields}");
    }
}
