# Hello World Example

## Descripción
Ejemplo básico que demuestra la sintaxis fundamental de Vela: variables, funciones, y salida por consola.

## Código

```vela
// Función principal
fn main() -> void {
    // Variable inmutable (por defecto)
    let message = "Hello, Vela!"

    // Llamada a función
    greet(message)
}

// Función auxiliar
fn greet(text: String) -> void {
    // Interpolación de strings
    let greeting = "👋 ${text}"

    // Salida por consola
    print(greeting)
}
```

## Compilación y Ejecución

```bash
# Compilar
vela-compiler compile hello.vela -o hello.bytecode

# Ejecutar
vela-vm run hello.bytecode
```

## Salida Esperada

```
👋 Hello, Vela!
```

## Conceptos Demostrados

- **Variables inmutables**: `let message = "Hello, Vela!"`
- **Funciones**: `fn main() -> void` y `fn greet(text: String) -> void`
- **Tipos**: `String`, `void`
- **Interpolación**: `"👋 ${text}"`
- **Llamadas**: `greet(message)` y `print(greeting)`