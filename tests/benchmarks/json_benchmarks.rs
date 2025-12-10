//! Performance benchmarks for JSON operations
//!
//! Benchmarks parsing, encoding, and decorator performance
//! across various data sizes and complexity levels.

use std::collections::HashMap;
use vela_stdlib::json::decorators::*;
use vela_stdlib::json::encoder::{convenience as json_encoder, JsonEncoder, JsonEncoderConfig};
use vela_stdlib::json::{parse, to_json, to_json_pretty, JsonValue};

#[cfg(test)]
mod json_benchmarks {
    use super::*;
    use std::time::{Duration, Instant};

    // Helper function to measure execution time
    fn measure_time<F, T>(f: F) -> (T, Duration)
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    // Benchmark parsing performance
    #[test]
    fn benchmark_parser_simple() {
        let json = r#"{"name":"John","age":30,"active":true}"#;

        let (result, duration) = measure_time(|| {
            for _ in 0..1000 {
                parse(json).unwrap();
            }
        });

        println!("Parser simple (1000 iterations): {:?}", duration);
        assert!(result.is_ok()); // Just to use the result
    }

    #[test]
    fn benchmark_parser_complex() {
        let json = create_complex_json();

        let (result, duration) = measure_time(|| {
            for _ in 0..100 {
                parse(&json).unwrap();
            }
        });

        println!("Parser complex (100 iterations): {:?}", duration);
        assert!(result.is_ok());
    }

    #[test]
    fn benchmark_parser_large() {
        let json = create_large_json(1000);

        let (result, duration) = measure_time(|| {
            for _ in 0..10 {
                parse(&json).unwrap();
            }
        });

        println!("Parser large (10 iterations): {:?}", duration);
        assert!(result.is_ok());
    }

    // Benchmark encoding performance
    #[test]
    fn benchmark_encoder_simple() {
        let value = create_simple_json_value();

        let (result, duration) = measure_time(|| {
            for _ in 0..1000 {
                to_json(&value);
            }
        });

        println!("Encoder simple (1000 iterations): {:?}", duration);
        assert!(!result.is_empty());
    }

    #[test]
    fn benchmark_encoder_complex() {
        let value = create_complex_json_value();

        let (result, duration) = measure_time(|| {
            for _ in 0..100 {
                to_json(&value);
            }
        });

        println!("Encoder complex (100 iterations): {:?}", duration);
        assert!(!result.is_empty());
    }

    #[test]
    fn benchmark_encoder_pretty() {
        let value = create_complex_json_value();

        let (result, duration) = measure_time(|| {
            for _ in 0..50 {
                to_json_pretty(&value);
            }
        });

        println!("Encoder pretty (50 iterations): {:?}", duration);
        assert!(!result.is_empty());
        assert!(result.contains('\n'));
    }

    // Benchmark streaming encoding
    #[test]
    fn benchmark_streaming_encoder() {
        let value = create_large_json_value(1000);

        let (result, duration) = measure_time(|| {
            for _ in 0..10 {
                let mut buffer = Vec::new();
                json_encoder::encode_to_writer(&value, &mut buffer).unwrap();
                buffer
            }
        });

        println!("Streaming encoder (10 iterations): {:?}", duration);
        assert!(!result.is_empty());
    }

    // Benchmark decorators
    #[test]
    fn benchmark_decorators_simple() {
        #[derive(Debug, Clone)]
        struct TestUser {
            id: String,
            name: String,
            email: String,
        }

        impl_json_decorated!(
            TestUser,
            JsonDecoratorConfig::default(),
            {
                let mut configs = HashMap::new();
                configs.insert("id".to_string(), JsonFieldDecorator::default());
                configs.insert("name".to_string(), JsonFieldDecorator::default());
                configs.insert("email".to_string(), JsonFieldDecorator::default());
                configs
            }
        );

        let user = TestUser {
            id: "123".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };

        let (result, duration) = measure_time(|| {
            for _ in 0..1000 {
                user.to_json_decorated();
            }
        });

        println!("Decorators simple (1000 iterations): {:?}", duration);
        assert!(!result.is_empty());
    }

    // Benchmark round-trip operations
    #[test]
    fn benchmark_round_trip() {
        let original_json = create_complex_json();

        let (result, duration) = measure_time(|| {
            for _ in 0..100 {
                let parsed = parse(&original_json).unwrap();
                let encoded = to_json(&parsed);
                parse(&encoded).unwrap();
            }
        });

        println!("Round-trip (100 iterations): {:?}", duration);
        assert!(result.is_ok());
    }

    // Benchmark memory usage patterns
    #[test]
    fn benchmark_memory_usage() {
        // Test with increasing sizes
        let sizes = vec![10, 100, 1000];

        for size in sizes {
            let json = create_large_json(size);
            let value = create_large_json_value(size);

            // Parse memory usage
            let (parsed, parse_time) = measure_time(|| parse(&json).unwrap());

            // Encode memory usage
            let (encoded, encode_time) = measure_time(|| to_json(&value));

            println!("Size {}: Parse={:?}, Encode={:?}", size, parse_time, encode_time);

            // Verify results
            assert!(parsed.is_ok());
            assert!(!encoded.is_empty());
        }
    }

    // Benchmark different encoding configurations
    #[test]
    fn benchmark_encoding_configs() {
        let value = create_complex_json_value();

        let configs = vec![
            ("compact", JsonEncoderConfig::default()),
            ("pretty", JsonEncoderConfig {
                pretty: true,
                ..Default::default()
            }),
            ("sorted", JsonEncoderConfig {
                sort_keys: true,
                ..Default::default()
            }),
            ("pretty_sorted", JsonEncoderConfig {
                pretty: true,
                sort_keys: true,
                ..Default::default()
            }),
        ];

        for (name, config) in configs {
            let (result, duration) = measure_time(|| {
                for _ in 0..50 {
                    JsonEncoder::with_config(config.clone()).encode(&value);
                }
            });

            println!("Config {} (50 iterations): {:?}", name, duration);
            assert!(!result.is_empty());
        }
    }

    // Helper functions to create test data

    fn create_simple_json() -> String {
        r#"{"name":"John","age":30,"active":true}"#.to_string()
    }

    fn create_complex_json() -> String {
        r#"
        {
            "users": [
                {"id": 1, "name": "Alice", "email": "alice@example.com", "active": true},
                {"id": 2, "name": "Bob", "email": "bob@example.com", "active": false},
                {"id": 3, "name": "Charlie", "email": "charlie@example.com", "active": true}
            ],
            "metadata": {
                "version": "1.0",
                "total_users": 3,
                "last_updated": "2024-01-15T10:30:00Z"
            },
            "settings": {
                "theme": "dark",
                "notifications": true,
                "language": "en"
            }
        }
        "#.to_string()
    }

    fn create_large_json(count: usize) -> String {
        let mut items = Vec::new();
        for i in 0..count {
            items.push(format!(
                r#"{{"id":{},"name":"Item {}","active":{},"value":{}}}"#,
                i, i, i % 2 == 0, i as f64 * 1.5
            ));
        }
        format!("[{}]", items.join(","))
    }

    fn create_simple_json_value() -> JsonValue {
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), JsonValue::String("John".to_string()));
        obj.insert("age".to_string(), JsonValue::Number(30.0));
        obj.insert("active".to_string(), JsonValue::Bool(true));
        JsonValue::Object(obj)
    }

    fn create_complex_json_value() -> JsonValue {
        let mut users = Vec::new();
        for i in 1..=3 {
            let mut user = HashMap::new();
            user.insert("id".to_string(), JsonValue::Number(i as f64));
            user.insert("name".to_string(), JsonValue::String(format!("User {}", i)));
            user.insert("email".to_string(), JsonValue::String(format!("user{}@example.com", i)));
            user.insert("active".to_string(), JsonValue::Bool(i % 2 == 1));
            users.push(JsonValue::Object(user));
        }

        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), JsonValue::String("1.0".to_string()));
        metadata.insert("total_users".to_string(), JsonValue::Number(3.0));

        let mut settings = HashMap::new();
        settings.insert("theme".to_string(), JsonValue::String("dark".to_string()));
        settings.insert("notifications".to_string(), JsonValue::Bool(true));

        let mut root = HashMap::new();
        root.insert("users".to_string(), JsonValue::Array(users));
        root.insert("metadata".to_string(), JsonValue::Object(metadata));
        root.insert("settings".to_string(), JsonValue::Object(settings));

        JsonValue::Object(root)
    }

    fn create_large_json_value(count: usize) -> JsonValue {
        let mut items = Vec::new();
        for i in 0..count {
            let mut obj = HashMap::new();
            obj.insert("id".to_string(), JsonValue::Number(i as f64));
            obj.insert("name".to_string(), JsonValue::String(format!("Item {}", i)));
            obj.insert("active".to_string(), JsonValue::Bool(i % 2 == 0));
            obj.insert("value".to_string(), JsonValue::Number(i as f64 * 1.5));
            items.push(JsonValue::Object(obj));
        }
        JsonValue::Array(items)
    }
}</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\tests\benchmarks\json_benchmarks.rs