# ADR-001: Sistema de Decoradores Arquitectónicos

## Estado
✅ **Aceptado** - Implementado en Sprint 9 (VELA-571)

## Fecha
2025-12-01

## Contexto

Vela necesita un sistema de metaprogramación declarativa para:
1. Dependency Injection (Angular/NestJS-style)
2. HTTP Routing y Controllers (Express/Spring-style)
3. Validación de datos (class-validator-style)
4. Middleware y Guards (NestJS-style)
5. Metadata para módulos (Angular-style)

El lenguaje debe soportar decoradores de manera **first-class** con parsing completo y type checking.

---

## Decisión

Implementar sistema de decoradores con:
- **Sintaxis**: `@decoratorName` o `@decoratorName(args)`
- **Token**: `AT (@)` en el lexer
- **AST Node**: `Decorator(name: str, arguments: List[Expression])`
- **Parsing**: `parse_decorators()` en el parser
- **Object Literals**: Soporte completo para metadata complejas

---

## Decoradores Implementados

### 1. Dependency Injection

#### @injectable
Marca una clase como inyectable en el contenedor DI.

**Sintaxis:**
```vela
@injectable
class UserService { }

@injectable({ scope: "singleton" })
class DatabaseConnection { }

@injectable({ scope: "transient" })
class RequestHandler { }
```

**Scopes soportados:**
- `singleton`: Una sola instancia para toda la aplicación (por defecto)
- `transient`: Nueva instancia en cada inyección
- `scoped`: Una instancia por scope (request, session, etc.)

---

#### @inject
Inyecta una dependencia usando un token.

**Sintaxis:**
```vela
class UserController {
  @inject({ token: "IUserService" })
  userService: IUserService
}
```

**Uso:**
- Inyección por interfaz (usando string token)
- Inyección por clase (usando tipo)

---

### 2. REST/HTTP

#### @controller
Define un controller HTTP con path base.

**Sintaxis:**
```vela
@controller("/api/users")
class UserController { }

@controller({ 
  path: "/api/v1/products",
  middleware: [AuthMiddleware, LoggerMiddleware]
})
class ProductController { }
```

---

#### @get, @post, @put, @patch, @delete
Define endpoints HTTP.

**Sintaxis:**
```vela
class UserController {
  @get("/")
  fn getAllUsers() -> Result<List<User>> { }
  
  @get("/:id")
  fn getUserById(id: Number) -> Result<User> { }
  
  @post("/")
  fn createUser(dto: CreateUserDTO) -> Result<User> { }
  
  @put("/:id")
  fn updateUser(id: Number, dto: UpdateUserDTO) -> Result<User> { }
  
  @patch("/:id/status")
  fn updateStatus(id: Number, status: Status) -> Result<User> { }
  
  @delete("/:id")
  fn deleteUser(id: Number) -> Result<void> { }
}
```

**Características:**
- Path parameters: `/:id`, `/:userId`
- Query parameters: inferidos del tipo de parámetro
- Body: inferido del DTO parameter

---

### 3. Middleware & Guards

#### @middleware
Define un middleware HTTP.

**Sintaxis:**
```vela
@middleware
class AuthMiddleware {
  fn handle(request: Request, next: () -> Response) -> Response {
    # Validar token, etc.
    return next()
  }
}
```

---

#### @guard
Define un guard (autorización).

**Sintaxis:**
```vela
@guard
class AdminGuard {
  fn canActivate(context: ExecutionContext) -> Bool {
    # Verificar permisos
    return user.isAdmin
  }
}
```

---

#### @interceptor
Define un interceptor (request/response manipulation).

**Sintaxis:**
```vela
@interceptor
class LoggingInterceptor {
  fn intercept(request: Request) -> Request {
    # Log request
    return request
  }
}
```

---

### 4. Validación

#### @validate
Aplica validación automática.

**Sintaxis:**
```vela
@validate
fn processInput(data: String) -> Result<String> { }

@validate({ min: 18, max: 100 })
fn validateAge(age: Number) -> Result<Bool> { }
```

---

#### @required
Campo obligatorio.

**Sintaxis:**
```vela
class CreateUserDTO {
  @required
  name: String
  
  @required
  email: String
}
```

---

#### @email
Valida formato de email.

**Sintaxis:**
```vela
class UserDTO {
  @email
  email: String
}
```

---

#### @min, @max
Valida valores numéricos mínimos/máximos.

**Sintaxis:**
```vela
class AgeDTO {
  @min(18)
  @max(100)
  age: Number
}
```

---

#### @length
Valida longitud de strings.

**Sintaxis:**
```vela
class PasswordDTO {
  @length({ min: 8, max: 64 })
  password: String
}
```

---

#### @regex
Valida contra expresión regular.

**Sintaxis:**
```vela
class UsernameDTO {
  @regex({ pattern: "^[a-zA-Z0-9_]+$" })
  username: String
}
```

---

#### @url
Valida formato de URL.

**Sintaxis:**
```vela
class WebsiteDTO {
  @url
  website: String
}
```

---

### 5. Sistema de Módulos

#### @module
Define un módulo (MULTIPLATAFORMA: Angular + NestJS style).

**Sintaxis (Backend Module):**
```vela
@module({
  controllers: [LoginController, RegisterController],  # REST endpoints
  providers: [AuthService, TokenService, DatabaseConnection],  # Business logic
  imports: ['system:http', 'module:shared'],
  exports: [AuthService]
})
module AuthModule {
  # Módulo NO instanciable
}
```

**Sintaxis (Frontend Module):**
```vela
@module({
  declarations: [LoginWidget, RegisterWidget, HeaderWidget],  # UI components
  providers: [AuthService, TokenService],  # Shared services
  imports: ['system:ui', 'module:shared'],
  exports: [AuthService, LoginWidget]
})
module AuthModule {
  # Módulo NO instanciable
}
```

**Sintaxis (Hybrid Module - TÍPICO EN VELA):**
```vela
@module({
  declarations: [AuthWidget, LoginForm],  # UI components
  controllers: [AuthController],  # REST API
  providers: [AuthService, TokenService, DatabaseConnection],  # Business logic
  imports: ['system:http', 'system:ui', 'module:shared'],
  exports: [AuthService, AuthWidget]  # Exporta AMBOS: service + widget
})
module AuthModule {
  # Módulo NO instanciable
}
```

**Reglas obligatorias (patrón MULTIPLATAFORMA):**
1. `declarations: []` → Widgets, components (frontend/general)
2. `controllers: []` → Controllers REST (backend, NO usan `@injectable`)
3. `providers: []` → Services, repositories, guards, middleware, pipes (con `@injectable`)
4. `imports: []` → Otros módulos
5. `exports ⊆ (declarations ∪ providers)` (puede exportar widgets O providers)
6. Módulo NO es instanciable (NO tiene constructor)
7. **Vela soporta AMBOS**: `declarations` (frontend) + `controllers` (backend)

---

## Consecuencias

### Positivas
✅ **Declarativo**: Metadata cerca del código  
✅ **Type-safe**: Validación en compile time  
✅ **Familiar**: Similar a TypeScript/Java/C#  
✅ **Extensible**: Fácil agregar nuevos decoradores  
✅ **Composable**: Múltiples decoradores por declaración  

### Negativas
⚠️ **Complejidad**: Más syntax que aprender  
⚠️ **Reflexión**: Requiere metadata en runtime  
⚠️ **Performance**: Overhead en runtime (mitigable con AOT)  

---

## Alternativas Consideradas

### 1. **Macros (Rust-style)**
```vela
derive![Injectable, Debug]
class UserService { }
```
- ✅ Más poderoso (code generation)
- ❌ Más complejo de implementar
- ❌ Menos familiar para developers JS/TS

**Rechazado**: Complejidad innecesaria para casos de uso comunes

---

### 2. **Atributos estáticos**
```vela
class UserService {
  static injectable = { scope: "singleton" }
}
```
- ✅ Más simple de parsear
- ❌ Menos declarativo
- ❌ Separado de la declaración

**Rechazado**: Menos idiomático, metadata separada del código

---

### 3. **Comentarios anotados (JSDoc-style)**
```vela
/**
 * @injectable({ scope: "singleton" })
 */
class UserService { }
```
- ✅ No requiere syntax especial
- ❌ Solo en comments (no first-class)
- ❌ No validable en compile time

**Rechazado**: No es first-class, pierde type safety

---

## Referencias

- **Jira**: [VELA-571](https://velalang.atlassian.net/browse/VELA-571)
- **Sprint**: Sprint 9 - Type System Validation + Module Parsing
- **Tasks**: TASK-016H, TASK-016I, TASK-016J

## Implementación

### Archivos modificados:
- `src/lexer/token.py` - Token AT agregado
- `src/lexer/lexer.py` - Case '@' agregado
- `src/parser/ast_nodes.py` - Decorator y ModuleDeclaration nodes
- `src/parser/parser.py` - parse_decorators(), parse_object_literal(), parse_module_declaration()

### Tests:
- `tests/unit/parser/test_module_parsing.py` - 30+ test cases
- `tests/unit/parser/test_decorators.py` - 40+ test cases

### Commits:
- `3ac5e13`: feat(parser): implementar parsing de module y decoradores
- `88e7149`: feat(parser): completar parsing de metadata object
- `0befe34`: test(parser): tests de module
- `17107d6`: test(parser): tests de decoradores arquitectónicos

---

## Actualización: Decoradores Removidos (2025-12-02)

### Decoradores DI Simplificados

**Decisión Original**: Implementar decoradores `@container` y `@provides` para DI avanzado

**Decisión Modificada**: Remover `@container` y `@provides` de la implementación inicial

**Decoradores Removidos**:
- ❌ `@container` - Contenedor DI explícito
- ❌ `@provides` - Factory providers por interfaz

**Razones**:
1. **Simplicidad**: Sistema DI inicial más simple con solo `@injectable` + `@inject`
2. **Suficiencia**: Cubre casos de uso principales
3. **Iteración**: Funcionalidad avanzada puede agregarse en versiones futuras

---

## Próximos Pasos

1. ✅ **Parser**: Implementado (Sprint 9)
2. ⏳ **Type Checker**: Validar metadata semánticamente (Sprint 10)
3. ⏳ **Runtime**: Reflexión y DI container (Sprint 11)
4. ⏳ **Codegen**: Generar código de inyección (Sprint 12)

---

**Última Actualización**: 2025-12-02  
**Estado**: Aceptado e Implementado (con modificaciones)  
**Versión**: 1.1.0
