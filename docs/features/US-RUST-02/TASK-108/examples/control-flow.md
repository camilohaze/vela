# Control Flow Example

## Descripción
Ejemplo que demuestra diferentes estructuras de control de flujo en Vela: condicionales, pattern matching, y bucles funcionales.

## Código

```vela
// Función principal
fn main() -> void {
    demonstrate_conditionals()
    demonstrate_pattern_matching()
    demonstrate_functional_loops()
}

// Condicionales
fn demonstrate_conditionals() -> void {
    print("=== Condicionales ===")

    let age = 25

    if age >= 18 {
        print("Eres adulto")
    } else {
        print("Eres menor de edad")
    }

    // Condicional con expresión
    let status = if age >= 65 {
        "Jubilado"
    } else if age >= 18 {
        "Trabajador"
    } else {
        "Estudiante"
    }

    print("Estado: ${status}")
}

// Pattern matching
fn demonstrate_pattern_matching() -> void {
    print("\n=== Pattern Matching ===")

    let result: Result<Number, String> = Ok(42)

    match result {
        Ok(value) => print("Éxito: ${value}")
        Err(error) => print("Error: ${error}")
    }

    // Pattern matching con destructuring
    let point = {x: 10, y: 20}

    match point {
        {x: 0, y: 0} => print("Origen")
        {x, y} => print("Punto en (${x}, ${y})")
    }
}

// Bucles funcionales
fn demonstrate_functional_loops() -> void {
    print("\n=== Bucles Funcionales ===")

    // Map: transformar elementos
    let numbers = [1, 2, 3, 4, 5]
    let doubled = numbers.map(x => x * 2)
    print("Duplicados: ${doubled}")

    // Filter: filtrar elementos
    let evens = numbers.filter(x => x % 2 == 0)
    print("Pares: ${evens}")

    // Reduce: reducir a un valor
    let sum = numbers.reduce((acc, x) => acc + x, 0)
    print("Suma: ${sum}")

    // ForEach: ejecutar acción por elemento
    print("Contando:")
    numbers.forEach(x => print("  ${x}"))

    // Chain operations
    let result = numbers
        .filter(x => x > 2)
        .map(x => x * x)
        .reduce((acc, x) => acc + x, 0)

    print("Resultado de operaciones encadenadas: ${result}")
}
```

## Compilación y Ejecución

```bash
# Compilar
vela-compiler compile control-flow.vela -o control-flow.bytecode

# Ejecutar
vela-vm run control-flow.bytecode
```

## Salida Esperada

```
=== Condicionales ===
Eres adulto
Estado: Trabajador

=== Pattern Matching ===
Éxito: 42
Punto en (10, 20)

=== Bucles Funcionales ===
Duplicados: [2, 4, 6, 8, 10]
Pares: [2, 4]
Suma: 15
Contando:
  1
  2
  3
  4
  5
Resultado de operaciones encadenadas: 50
```

## Conceptos Demostrados

- **Condicionales**: `if/else` con expresiones
- **Pattern Matching**: `match` con destructuring
- **Bucles Funcionales**: `map`, `filter`, `reduce`, `forEach`
- **Encadenamiento**: Method chaining
- **Tipos**: `Result<T, E>`, arrays, objetos
- **Funciones Lambda**: `x => x * 2`