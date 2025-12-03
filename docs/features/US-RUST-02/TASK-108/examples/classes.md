# Classes and Objects Example

## Descripción
Ejemplo que demuestra el sistema de clases y objetos de Vela, incluyendo herencia, constructores, y métodos.

## Código

```vela
// Clase base
class Animal {
    constructor(name: String) {
        this.name = name
    }

    fn speak() -> String {
        return "${this.name} hace un sonido"
    }

    fn get_name() -> String {
        return this.name
    }
}

// Clase derivada
class Dog extends Animal {
    constructor(name: String, breed: String) {
        super(name)  // Llama al constructor padre
        this.breed = breed
    }

    // Override del método padre
    override fn speak() -> String {
        return "${this.name} ladra: ¡Guau!"
    }

    // Método específico de Dog
    fn fetch(item: String) -> String {
        return "${this.name} trae ${item}"
    }
}

// Otra clase derivada
class Cat extends Animal {
    constructor(name: String, color: String) {
        super(name)
        this.color = color
    }

    override fn speak() -> String {
        return "${this.name} maúlla: ¡Miau!"
    }

    fn purr() -> String {
        return "${this.name} ronronea"
    }
}

// Función principal
fn main() -> void {
    demonstrate_classes()
    demonstrate_polymorphism()
}

fn demonstrate_classes() -> void {
    print("=== Clases Básicas ===")

    // Crear instancias
    let dog = Dog("Max", "Labrador")
    let cat = Cat("Luna", "Negra")

    // Usar métodos
    print(dog.speak())      // "Max ladra: ¡Guau!"
    print(cat.speak())      // "Luna maúlla: ¡Miau!"

    // Métodos específicos
    print(dog.fetch("pelota"))  // "Max trae pelota"
    print(cat.purr())           // "Luna ronronea"

    // Acceder a propiedades
    print("${dog.get_name()} es un ${dog.breed}")
    print("${cat.get_name()} es ${cat.color}")
}

fn demonstrate_polymorphism() -> void {
    print("\n=== Polimorfismo ===")

    // Array de animales (polimorfismo)
    let animals: [Animal] = [
        Dog("Buddy", "Golden Retriever"),
        Cat("Whiskers", "Blanco"),
        Dog("Rex", "Pastor Alemán")
    ]

    // Todos responden a speak() pero cada uno a su manera
    animals.forEach(animal => {
        print(animal.speak())
    })

    // Type checking en runtime
    animals.forEach(animal => {
        if animal is Dog {
            let dog = animal as Dog
            print("Perro encontrado: ${dog.fetch('bastón')}")
        } else if animal is Cat {
            let cat = animal as Cat
            print("Gato encontrado: ${cat.purr()}")
        }
    })
}

// Clase con métodos estáticos
class MathUtils {
    // Método estático
    static fn add(a: Number, b: Number) -> Number {
        return a + b
    }

    static fn multiply(a: Number, b: Number) -> Number {
        return a + b  // Bug intencional para el ejemplo
    }
}

fn demonstrate_static_methods() -> void {
    print("\n=== Métodos Estáticos ===")

    let sum = MathUtils.add(5, 3)
    let product = MathUtils.multiply(4, 2)

    print("5 + 3 = ${sum}")
    print("4 * 2 = ${product}")  // Mostrará 6 en lugar de 8 (bug)
}
```

## Compilación y Ejecución

```bash
# Compilar
vela-compiler compile classes.vela -o classes.bytecode

# Ejecutar
vela-vm run classes.bytecode
```

## Salida Esperada

```
=== Clases Básicas ===
Max ladra: ¡Guau!
Luna maúlla: ¡Miau!
Max trae pelota
Luna ronronea
Max es un Labrador
Luna es Negra

=== Polimorfismo ===
Buddy ladra: ¡Guau!
Whiskers maúlla: ¡Miau!
Rex ladra: ¡Guau!
Perro encontrado: Buddy trae bastón
Gato encontrado: Whiskers ronronea
Perro encontrado: Rex trae bastón

=== Métodos Estáticos ===
5 + 3 = 8
4 * 2 = 6
```

## Conceptos Demostrados

- **Clases**: `class Animal`, `class Dog extends Animal`
- **Constructores**: `constructor(name: String)`
- **Herencia**: `extends Animal`
- **Métodos**: `fn speak() -> String`
- **Override**: `override fn speak()`
- **Polimorfismo**: Array de `Animal` con diferentes tipos
- **Type Checking**: `is` y `as` operators
- **Métodos Estáticos**: `static fn add()`
- **Propiedades**: `this.name`, `this.breed`

## Notas

- En Vela, todas las clases son referencias (no hay copia por valor)
- El polimorfismo funciona automáticamente con arrays heterogéneos
- Los métodos estáticos se llaman en la clase, no en instancias