//! JSON Decorators Example for Vela
//!
//! This example demonstrates the JSON decorator system for automatic
//! serialization and deserialization of Vela types.

use std::collections::HashMap;
use vela_stdlib::json::decorators::*;
use vela_stdlib::json::*;

// Example struct with basic JSON decorator
#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: u32,
    email: String,
}

impl_json_decorated!(
    Person,
    JsonDecoratorConfig {
        include: None,
        exclude: None,
        rename: HashMap::new(),
        default_values: HashMap::new(),
    },
    {
        let mut configs = HashMap::new();
        configs.insert("name".to_string(), JsonFieldDecorator::default());
        configs.insert("age".to_string(), JsonFieldDecorator::default());
        configs.insert("email".to_string(), JsonFieldDecorator::default());
        configs
    }
);

// Example struct with advanced decorators
#[derive(Debug, Clone)]
struct Product {
    id: String,
    name: String,
    price: f64,
    category: String,
    internal_code: String, // This will be skipped
    created_at: String,
}

impl_json_decorated!(
    Product,
    JsonDecoratorConfig {
        exclude: Some(vec!["internal_code".to_string()]),
        rename: {
            let mut map = HashMap::new();
            map.insert("created_at".to_string(), "createdAt".to_string());
            map
        },
        ..Default::default()
    },
    {
        let mut configs = HashMap::new();
        configs.insert("id".to_string(), JsonFieldDecorator::default());
        configs.insert("name".to_string(), JsonFieldDecorator::default());
        configs.insert("price".to_string(), JsonFieldDecorator::default());
        configs.insert("category".to_string(), JsonFieldDecorator::default());
        configs.insert("internal_code".to_string(), JsonFieldDecorator {
            skip: true,
            ..Default::default()
        });
        configs.insert("created_at".to_string(), JsonFieldDecorator {
            rename: Some("createdAt".to_string()),
            ..Default::default()
        });
        configs
    }
);

// Example with nested objects
#[derive(Debug, Clone)]
struct Order {
    id: String,
    customer: Person,
    products: Vec<Product>,
    total: f64,
}

impl_json_decorated!(
    Order,
    JsonDecoratorConfig::default(),
    {
        let mut configs = HashMap::new();
        configs.insert("id".to_string(), JsonFieldDecorator::default());
        configs.insert("customer".to_string(), JsonFieldDecorator::default());
        configs.insert("products".to_string(), JsonFieldDecorator::default());
        configs.insert("total".to_string(), JsonFieldDecorator::default());
        configs
    }
);

fn main() {
    println!("=== Vela JSON Decorators Example ===\n");

    // Example 1: Basic serialization
    println!("1. Basic Person serialization:");
    let person = Person {
        name: "Alice Johnson".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    let json = person.to_json_decorated();
    println!("Person: {:?}", person);
    println!("JSON: {}\n", json);

    // Example 2: Product with decorators
    println!("2. Product with field decorators:");
    let product = Product {
        id: "prod-123".to_string(),
        name: "Laptop".to_string(),
        price: 999.99,
        category: "Electronics".to_string(),
        internal_code: "INT-456".to_string(),
        created_at: "2024-01-15".to_string(),
    };

    let json = product.to_json_decorated();
    println!("Product: {:?}", product);
    println!("JSON (internal_code skipped, created_at renamed): {}\n", json);

    // Example 3: Nested objects
    println!("3. Order with nested objects:");
    let order = Order {
        id: "order-789".to_string(),
        customer: person,
        products: vec![product],
        total: 999.99,
    };

    let json = order.to_json_decorated();
    println!("Order: {:?}", order);
    println!("JSON: {}\n", json);

    // Example 4: Using helper functions
    println!("4. Using helper functions:");
    let all_fields = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let include = Some(vec!["a".to_string(), "c".to_string()]);
    let exclude = Some(vec!["b".to_string()]);

    let filtered = helpers::filter_fields(&all_fields, &include, &exclude);
    println!("All fields: {:?}", all_fields);
    println!("Include: {:?}, Exclude: {:?}", include, exclude);
    println!("Filtered: {:?}\n", filtered);

    println!("=== Example completed successfully! ===");
}