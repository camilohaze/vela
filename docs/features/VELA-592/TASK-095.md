# TASK-095: Tests Finales JSON

## 📋 Información General
- **Historia:** VELA-592 (JSON serialization)
- **Estado:** Pendiente ⏳
- **Fecha:** 2024-12-30

## 🎯 Objetivo
Implementar suite completa de tests para validar toda la funcionalidad JSON: parser, encoder, decorators, edge cases, performance y compatibilidad.

## 🔨 Tests a Implementar

### Tests de Integración

#### Round-trip Completo
```rust
#[test]
fn test_full_round_trip() {
    // Parse → Encode → Parse → Encode → ...
    let complex_json = r#"
    {
        "users": [
            {
                "id": 1,
                "name": "Alice",
                "profile": {
                    "age": 25,
                    "hobbies": ["reading", "coding", "gaming"],
                    "active": true
                }
            },
            {
                "id": 2,
                "name": "Bob",
                "profile": {
                    "age": 30,
                    "hobbies": ["sports", "music"],
                    "active": false
                }
            }
        ],
        "metadata": {
            "version": "1.0",
            "timestamp": 1234567890,
            "config": {
                "debug": true,
                "max_users": 1000
            }
        }
    }"#;

    // Verificar que parse → encode → parse produce el mismo resultado
    let parsed1 = parse(complex_json).unwrap();
    let encoded = parsed1.to_json();
    let parsed2 = parse(&encoded).unwrap();
    
    assert_eq!(parsed1, parsed2);
}
```

#### Tests de Performance
```rust
#[test]
fn test_large_json_performance() {
    // Generar JSON grande (1000+ elementos)
    let large_array: Vec<JsonValue> = (0..1000)
        .map(|i| JsonValue::Object({
            let mut obj = HashMap::new();
            obj.insert("id".to_string(), JsonValue::Number(i as f64));
            obj.insert("data".to_string(), JsonValue::String(format!("item_{}", i)));
            obj
        }))
        .collect();
    
    let large_json = JsonValue::Array(large_array);
    
    // Medir tiempo de encoding
    let start = std::time::Instant::now();
    let encoded = large_json.to_json();
    let duration = start.elapsed();
    
    // Verificar que es razonablemente rápido (< 100ms)
    assert!(duration < std::time::Duration::from_millis(100));
    
    // Verificar que el JSON es válido
    let reparsed = parse(&encoded).unwrap();
    assert!(matches!(reparsed, JsonValue::Array(_)));
}
```

### Tests de Edge Cases

#### Caracteres Especiales Extremos
```rust
#[test]
fn test_unicode_edge_cases() {
    // Emojis, caracteres de diferentes alfabetos, caracteres de control
    let test_cases = vec![
        "🚀 Rocket emoji",
        "Hello 世界 World",
        "Тест на русском",
        "café résumé naïve",
        "控制字符\u{0000}\u{0001}\u{001F}",
    ];
    
    for text in test_cases {
        let json_value = JsonValue::String(text.to_string());
        let encoded = json_value.to_json();
        let decoded = parse(&encoded).unwrap();
        
        assert_eq!(json_value, decoded);
    }
}
```

#### Números Extremos
```rust
#[test]
fn test_number_edge_cases() {
    let test_cases = vec![
        0.0,
        -0.0,
        f64::MIN,
        f64::MAX,
        f64::EPSILON,
        1e-10,
        1e10,
        1.23456789012345,
        -1.23456789012345,
    ];
    
    for &num in &test_cases {
        let json_value = JsonValue::Number(num);
        let encoded = json_value.to_json();
        let decoded = parse(&encoded).unwrap();
        
        match decoded {
            JsonValue::Number(decoded_num) => {
                // Permitir pequeña diferencia por precisión flotante
                assert!((num - decoded_num).abs() < f64::EPSILON * 10.0);
            }
            _ => panic!("Expected number"),
        }
    }
}
```

### Tests de Error Handling

#### JSON Malformado
```rust
#[test]
fn test_malformed_json_errors() {
    let malformed_cases = vec![
        "{",           // Objeto sin cerrar
        "[",           // Array sin cerrar
        r#""unclosed"#, // String sin cerrar
        "{,}",         // Coma sin valor
        "[,]",         // Coma sin valor
        r#"{"key"}"#,   // Key sin valor
        "tru",         // Boolean incompleto
        "fals",        // Boolean incompleto
        "nul",         // Null incompleto
        "123abc",      // Número inválido
        r#""control char: \u001F""#, // Control char sin escape
    ];
    
    for case in malformed_cases {
        assert!(parse(case).is_err(), "Should fail to parse: {}", case);
    }
}
```

### Tests de Decorators (TASK-094)

#### Serialización Automática
```rust
#[test]
fn test_decorator_serialization() {
    // Asumiendo que TASK-094 implementa decorators
    // Este test se implementará después de TASK-094
    
    /*
    @json_serializable
    class TestUser {
        id: Number
        name: String
        active: Bool
    }
    
    let user = TestUser { id: 1, name: "Alice", active: true };
    let json = user.to_json();
    let expected = r#"{"id":1,"name":"Alice","active":true}"#;
    assert_eq!(json, expected);
    */
}
```

## ✅ Criterios de Aceptación

### Cobertura de Tests
- [ ] Parser: 100% de casos JSON válidos
- [ ] Encoder: 100% de tipos JsonValue
- [ ] Edge cases: Unicode, números extremos, caracteres especiales
- [ ] Error handling: Todos los casos de JSON malformado
- [ ] Performance: Tests de carga para estructuras grandes
- [ ] Round-trip: Compatibilidad parse ↔ encode
- [ ] Decorators: Tests de serialización automática (después de TASK-094)

### Métricas de Calidad
- [ ] Cobertura de código: >95%
- [ ] Tests pasando: 100%
- [ ] Performance aceptable: <100ms para JSON de 1MB
- [ ] Memoria: Sin leaks en tests de carga

### Validación
- [ ] RFC 8259 compliance verificada
- [ ] Compatibilidad con JSON estándar
- [ ] Interoperabilidad con otras implementaciones JSON

## 📊 Métricas Esperadas

- **Tests totales:** 50+ tests unitarios + integración
- **Casos edge:** 20+ casos de borde probados
- **Performance:** 10-50 MB/s encoding/parsing
- **Cobertura:** >95% de líneas y branches

## 🔗 Referencias

- **Jira:** [TASK-095](https://velalang.atlassian.net/browse/TASK-095)
- **Historia:** [VELA-592](https://velalang.atlassian.net/browse/VELA-592)
- **RFC 8259:** Casos edge y compliance
- **Benchmarks:** Comparación con otras implementaciones JSON