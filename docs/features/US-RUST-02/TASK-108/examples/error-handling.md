# Error Handling Example

## Descripción
Ejemplo que demuestra el manejo de errores en Vela usando el tipo `Result<T, E>` y el sistema de excepciones.

## Código

```vela
// Tipos de error personalizados
enum ValidationError {
    TooShort { min_length: Number },
    TooLong { max_length: Number },
    InvalidCharacters { chars: String },
    Empty,
}

// Función que puede fallar
fn validate_username(username: String) -> Result<String, ValidationError> {
    if username.is_empty() {
        return Err(ValidationError.Empty)
    }

    if username.length() < 3 {
        return Err(ValidationError.TooShort { min_length: 3 })
    }

    if username.length() > 20 {
        return Err(ValidationError.TooLong { max_length: 20 })
    }

    // Verificar caracteres válidos
    let invalid_chars = username.find_chars(c => !c.is_alphanumeric() && c != '_')
    if !invalid_chars.is_empty() {
        return Err(ValidationError.InvalidCharacters { chars: invalid_chars })
    }

    return Ok(username)
}

// Función que puede lanzar excepciones
fn risky_operation() -> String throws {
    let random = Math.random()

    if random < 0.3 {
        throw Error("Operación fallida aleatoriamente")
    }

    if random < 0.6 {
        throw ValidationError.InvalidCharacters { chars: "!@#" }
    }

    return "Operación exitosa"
}

// Función principal
fn main() -> void {
    demonstrate_result_handling()
    demonstrate_exception_handling()
    demonstrate_advanced_patterns()
}

fn demonstrate_result_handling() -> void {
    print("=== Manejo de Result ===")

    let usernames = ["", "a", "valid_user", "user with spaces", "very_long_username_that_exceeds_limit"]

    usernames.forEach(username => {
        print("Validando: '${username}'")

        match validate_username(username) {
            Ok(valid_username) => {
                print("  ✅ Válido: ${valid_username}")
            }
            Err(error) => {
                match error {
                    ValidationError.Empty => {
                        print("  ❌ Error: El nombre de usuario no puede estar vacío")
                    }
                    ValidationError.TooShort { min_length } => {
                        print("  ❌ Error: El nombre debe tener al menos ${min_length} caracteres")
                    }
                    ValidationError.TooLong { max_length } => {
                        print("  ❌ Error: El nombre no puede tener más de ${max_length} caracteres")
                    }
                    ValidationError.InvalidCharacters { chars } => {
                        print("  ❌ Error: Caracteres inválidos: ${chars}")
                    }
                }
            }
        }
    })
}

fn demonstrate_exception_handling() -> void {
    print("\n=== Manejo de Excepciones ===")

    // Intentar operación riesgosa múltiples veces
    (1..=5).forEach(i => {
        print("Intento ${i}:")

        try {
            let result = risky_operation()
            print("  ✅ ${result}")
        } catch (e: Error) {
            print("  ❌ Error: ${e.message}")
        } catch (e: ValidationError) {
            match e {
                ValidationError.InvalidCharacters { chars } => {
                    print("  ❌ Caracteres inválidos en operación: ${chars}")
                }
                _ => {
                    print("  ❌ Otro error de validación: ${e}")
                }
            }
        } finally {
            print("  🔄 Intento completado")
        }
    })
}

fn demonstrate_advanced_patterns() -> void {
    print("\n=== Patrones Avanzados ===")

    // Chaining con map y and_then
    let result = validate_username("valid_user")
        .map(username => username.to_uppercase())
        .and_then(upper_username => {
            if upper_username.contains("ADMIN") {
                Err(ValidationError.InvalidCharacters { chars: "ADMIN" })
            } else {
                Ok(upper_username)
            }
        })

    match result {
        Ok(username) => print("Usuario procesado: ${username}")
        Err(error) => print("Error en procesamiento: ${error}")
    }

    // Unwrap con valores por defecto
    let safe_username = validate_username("bad user")
        .unwrap_or("guest_user")

    print("Usuario seguro: ${safe_username}")

    // Pattern matching con Option
    let maybe_user = Some("test_user")
    let validated = maybe_user
        .map(username => validate_username(username))
        .flatten()  // Result<Option<T>, E> -> Option<Result<T, E>>

    match validated {
        Some(Ok(username)) => print("Usuario validado desde Option: ${username}")
        Some(Err(error)) => print("Error de validación desde Option: ${error}")
        None => print("No hay usuario para validar")
    }
}

// Extensión de String para validaciones
extension String {
    fn find_chars(predicate: (char) -> Bool) -> String {
        let chars = []
        this.chars().forEach(c => {
            if predicate(c) {
                chars.push(c)
            }
        })
        return chars.join("")
    }
}
```

## Compilación y Ejecución

```bash
# Compilar
vela-compiler compile error-handling.vela -o error-handling.bytecode

# Ejecutar
vela-vm run error-handling.bytecode
```

## Salida Esperada

```
=== Manejo de Result ===
Validando: ''
  ❌ Error: El nombre de usuario no puede estar vacío
Validando: 'a'
  ❌ Error: El nombre debe tener al menos 3 caracteres
Validando: 'valid_user'
  ✅ Válido: valid_user
Validando: 'user with spaces'
  ❌ Error: Caracteres inválidos:  
Validando: 'very_long_username_that_exceeds_limit'
  ❌ Error: El nombre no puede tener más de 20 caracteres

=== Manejo de Excepciones ===
Intento 1:
  ❌ Error: Operación fallida aleatoriamente
  🔄 Intento completado
Intento 2:
  ❌ Caracteres inválidos en operación: !@#
  🔄 Intento completado
Intento 3:
  ✅ Operación exitosa
  🔄 Intento completado
Intento 4:
  ❌ Error: Operación fallida aleatoriamente
  🔄 Intento completado
Intento 5:
  ✅ Operación exitosa
  🔄 Intento completado

=== Patrones Avanzados ===
Usuario procesado: VALID_USER
Usuario seguro: guest_user
Usuario validado desde Option: test_user
```

## Conceptos Demostrados

- **Result<T, E>**: Tipo para operaciones que pueden fallar
- **Enums con datos**: `ValidationError` con campos asociados
- **Pattern Matching**: `match` con destructuring
- **Excepciones**: `throw`, `try/catch/finally`
- **Method Chaining**: `map`, `and_then`, `unwrap_or`
- **Option<T>**: Para valores opcionales
- **Extensiones**: Agregar métodos a tipos existentes

## Notas

- `Result<T, E>` es preferido sobre excepciones para lógica de negocio
- Las excepciones son para errores realmente excepcionales
- El pattern matching hace el manejo de errores expresivo y seguro
- Los métodos de chaining permiten composición funcional