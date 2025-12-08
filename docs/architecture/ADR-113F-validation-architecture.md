# ADR-113F: Arquitectura del Sistema de Validación

## Estado
✅ Aceptado

## Fecha
2025-01-30

## Contexto
Necesitamos implementar un sistema de validación de datos integrado en Vela que permita validar datos de entrada de manera declarativa y segura. Este sistema debe integrarse con:

- DTOs (Data Transfer Objects)
- Formularios de UI
- Controllers REST
- Middleware de validación
- Guards de autorización

El sistema debe ser:
- **Type-safe**: Validación en tiempo de compilación
- **Declarativo**: Usando decoradores
- **Componible**: Validadores reutilizables
- **Extensible**: Fácil agregar validadores custom
- **Performante**: Validación eficiente sin overhead excesivo

## Decisión
Implementaremos un sistema de validación híbrido que combina:

### 1. Validación Declarativa con Decoradores
```vela
@validate
class CreateUserDTO {
  @required @length(min=2, max=50)
  name: String

  @required @email
  email: String

  @min(18) @max(120)
  age: Number
}
```

### 2. Validación Programática con Schema Builder
```vela
let userSchema = Schema::new()
  .field("name", string().required().min(2).max(50))
  .field("email", string().required().email())
  .field("age", number().min(18).max(120))
```

### 3. Arquitectura de Tres Capas

#### Capa de Validadores (Validators)
- **Built-in validators**: `@required`, `@email`, `@min`, `@max`, `@length`, `@regex`, `@url`
- **Custom validators**: Función que retorna `ValidationResult`
- **Async validators**: Para validaciones que requieren I/O (ej: unicidad en BD)

#### Capa de Esquemas (Schemas)
- **Field-level validation**: Validación por campo
- **Object-level validation**: Validación cross-field
- **Nested validation**: Validación de objetos anidados
- **Conditional validation**: Validación condicional basada en otros campos

#### Capa de Integración (Integration)
- **DTO validation**: Automática en controllers
- **Form validation**: En UI components
- **Middleware validation**: En pipelines HTTP
- **Guard validation**: En authorization guards

### 4. Tipos de Validación

#### Síncrona vs Asíncrona
```vela
// Síncrona - validación inmediata
@validate
class UserDTO {
  @required @email
  email: String
}

// Asíncrona - requiere I/O
@validate(async=true)
class UserDTO {
  @unique(table="users", field="email")
  email: String
}
```

#### Validación de Campo vs Validación de Objeto
```vela
@validate
class PasswordChangeDTO {
  @required
  currentPassword: String

  @required @min(8)
  newPassword: String

  @required
  confirmPassword: String

  // Validación cross-field
  @custom("passwordsMustMatch")
  fn validatePasswordsMatch() -> ValidationResult {
    if newPassword != confirmPassword {
      return ValidationError("Passwords do not match")
    }
    return ValidationSuccess
  }
}
```

### 5. Sistema de Errores

#### ValidationError Type
```vela
struct ValidationError {
  field: String
  code: String
  message: String
  value: Any
  constraints: Map<String, Any>
}

struct ValidationResult {
  isValid: Bool
  errors: List<ValidationError>
}
```

#### Códigos de Error Estandarizados
- `REQUIRED`: Campo requerido faltante
- `EMAIL`: Formato de email inválido
- `MIN`: Valor menor al mínimo
- `MAX`: Valor mayor al máximo
- `LENGTH`: Longitud fuera de rango
- `REGEX`: No cumple patrón regex
- `CUSTOM`: Error de validador custom

## Consecuencias

### Positivas
- **Type Safety**: Validación integrada con el type system
- **Developer Experience**: API declarativa y intuitiva
- **Performance**: Validación eficiente sin reflection overhead
- **Extensibility**: Fácil agregar validadores custom
- **Integration**: Funciona con todo el ecosistema Vela (DI, HTTP, UI)
- **Error Handling**: Sistema de errores detallado y consistente

### Negativas
- **Complexity**: Sistema más complejo que validación básica
- **Learning Curve**: Nuevos conceptos (decorators, schemas)
- **Runtime Overhead**: Validación tiene costo computacional
- **Memory Usage**: Esquemas de validación consumen memoria

### Trade-offs
- **Síncrono vs Asíncrono**: Optamos por API unificada con flag `async`
- **Decoradores vs Programático**: Ambos soportados para diferentes use cases
- **Built-in vs Custom**: Built-in para casos comunes, custom para específicos

## Alternativas Consideradas

### 1. Solo Validación Programática (Rechazada)
```vela
// Sin decoradores, solo código
fn validateUser(user: User) -> ValidationResult {
  let mut errors = []
  if user.name.isEmpty() { errors.push("name required") }
  if !isValidEmail(user.email) { errors.push("invalid email") }
  // ...
}
```
**Rechazada porque**: No es declarativo, boilerplate excesivo, error-prone

### 2. Reflection-based Validation (Rechazada)
```vela
// Usando reflection como en Java/Spring
@Validate
class UserDTO { /* reflection infiere validaciones */ }
```
**Rechazada porque**: Performance overhead, no type-safe en compile time, complejo

### 3. Schema-only Validation (Rechazada)
```vela
// Solo schemas programáticos
let schema = Schema::new()...
```
**Rechazada porque**: No declarativo, difícil de mantener, no integrado con tipos

## Implementación

### Fase 1: Core Validators (TASK-113G)
- Implementar decoradores básicos: `@required`, `@email`, `@min`, `@max`, `@length`
- ValidationError y ValidationResult types
- Integración básica con DTOs

### Fase 2: Schema Builder (TASK-113H)
- API fluent para construcción programática
- Nested object validation
- Conditional validation

### Fase 3: Integration (TASK-113J)
- Middleware HTTP validation
- Form validation en UI
- Controller integration

### Fase 4: Advanced Features
- Async validators
- Custom validators
- Internationalization de mensajes

## Referencias
- Jira: VELA-596 (US-24B)
- Patrón: Validation Pattern
- Inspiración: Joi (JavaScript), Hibernate Validator (Java), FluentValidation (.NET)
- Documentación: docs/features/VELA-596/

## Implementación
Ver código en: `src/validation/`
Tests en: `tests/unit/validation/`