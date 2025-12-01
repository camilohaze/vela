# 11. Sistema de Módulos Estilo Angular para Vela

**Fecha**: 30 de noviembre de 2025  
**Estado**: Propuesta de diseño  
**Prioridad**: P0 (Crítico para MVP 1.0)

---

## 📋 Visión General

Vela implementará un **sistema de módulos estilo Angular/NestJS** donde cada módulo es una unidad autocontenida que:

1. **Encapsula** servicios, componentes, widgets, y otros elementos
2. **Exporta** elementos específicos para ser usados por otros módulos
3. **Importa** otros módulos para acceder a sus exportaciones
4. **Declara** providers para inyección de dependencias

---

## 🎯 Conceptos Clave

### 0. ⚠️ IMPORTANTE: Sistema de Imports de Vela

**Vela NO usa namespaces ni declaraciones de paquete tipo Java/Kotlin.**

**Estilo CORRECTO de imports en Vela**:
```vela
# APIs INTERNAS DE VELA (ui, reactive, http, actors, etc.)
import 'system:ui'              # Sistema de UI de Vela (Container, Column, Text, etc.)
import 'system:reactive'        # Sistema reactivo de Vela (signal, computed, effect)
import 'system:http'            # Sistema HTTP de Vela (Request, Response, HttpClient)
import 'system:actors'          # Sistema de actores de Vela
import 'system:state'           # Sistema de state management

# DEPENDENCIAS EXTERNAS INSTALADAS (npm, pub, etc.)
import 'package:lodash'         # Librería externa instalada
import 'package:axios'          # Cliente HTTP externo
import 'package:date-fns'       # Utilidades de fecha externa

# MÓDULOS DEL PROYECTO (definidos con @module)
import 'module:auth'            # AuthModule (definido con @module)
import 'module:users'           # UsersModule

# LIBRERÍAS INTERNAS DEL PROYECTO (definidas con @library)
import 'library:utils'          # Librería de utilidades interna (definida con @library)
import 'library:validators'     # Librería de validadores interna

# EXTENSIONES INTERNAS DEL PROYECTO (definidas con @extension)
import 'extension:charts'       # Extensión de gráficos (definida con @extension)
import 'extension:maps'         # Extensión de mapas

# ASSETS
import 'assets:images'          # Assets de imágenes
import 'assets:fonts'           # Assets de fuentes
```

**❌ NO existe en Vela**:
```vela
module com.example.myapp.auth;  // ❌ ESTO NO EXISTE EN VELA
import com.example.myapp.X;     // ❌ ESTO NO EXISTE EN VELA
import vela.ui.Widget;          // ❌ NO - usar import 'system:ui'
```

**✅ Forma correcta**:
```vela
import 'system:ui'              // ✅ API interna de Vela
import 'package:lodash'         // ✅ Dependencia externa
import 'module:auth'            // ✅ Módulo del proyecto (@module)
import 'library:utils'          // ✅ Librería interna (@library)
import 'extension:charts'       // ✅ Extensión interna (@extension)
```

### 1. Organización por Estructura de Directorios

- La ubicación del archivo define su path de import
- No hay declaraciones de paquete/namespace
- El compilador infiere el módulo desde la estructura de carpetas

**Ejemplo**:
```
src/
└── auth/
    ├── services/
    │   └── auth.service.vela   → import 'module:auth/services'
    └── widgets/
        └── login.widget.vela   → import 'module:auth/widgets'
```

### 2. Módulo Funcional (NUEVO - `@module`)
```vela
// Archivo: src/auth/auth.module.vela
import 'module:auth/services'  // AuthService, AuthRepository
import 'module:auth/widgets'   // LoginWidget
import 'module:shared/http'    // HttpModule
import 'module:shared/logger'  // LoggerModule

@module({
  declarations: [AuthService, LoginWidget, AuthGuard],
  exports: [AuthService, LoginWidget],
  providers: [AuthRepository],
  imports: [HttpModule, LoggerModule]
})
class AuthModule { }
```
- Define una **unidad funcional** autocontenida
- Agrupa elementos relacionados (estilo Angular)
- Maneja dependencias y visibilidad
- **Usa imports con prefijos** - NO namespaces

---

## 🔧 Sintaxis Completa del Decorador `@module`

```vela
@module({
  // Elementos declarados en este módulo
  declarations: Array<Type>,
  
  // Elementos que otros módulos pueden usar
  exports: Array<Type>,
  
  // Servicios disponibles para inyección de dependencias
  providers: Array<Type | Provider>,
  
  // Módulos importados (sus exports están disponibles)
  imports: Array<Type>
})
class ModuleName { }
```

### Propiedades del Decorador

| Propiedad | Tipo | Descripción | Requerido |
|-----------|------|-------------|-----------|
| `declarations` | `Array<Type>` | Clases, widgets, componentes declarados en este módulo | ❌ |
| `exports` | `Array<Type>` | Subset de declarations disponibles para otros módulos | ❌ |
| `providers` | `Array<Type \| Provider>` | Servicios inyectables | ❌ |
| `imports` | `Array<Type>` | Módulos cuyas exportaciones se necesitan | ❌ |

---

## 📝 Ejemplo Completo: AuthModule

### Estructura de Archivos

```
src/
├── main.vela                 # Entry point
├── app.module.vela          # Módulo raíz
└── auth/
    ├── auth.module.vela     # Módulo Auth
    ├── services/
    │   ├── auth.service.vela
    │   └── auth.repository.vela
    ├── guards/
    │   └── auth.guard.vela
    └── widgets/
        └── login.widget.vela
```

### 1. AuthService (src/auth/services/auth.service.vela)

```vela
// Archivo: src/auth/services/auth.service.vela
import 'module:auth/repositories'  // AuthRepository

@injectable(scope: Scope.Singleton)
service AuthService {
  constructor(@inject private repository: AuthRepository) { }
  
  public fn login(email: String, password: String): Result<User, Error> {
    if (email.isEmpty() || password.length < 8) {
      return Result.Err("Invalid credentials");
    }
    return this.repository.login(email, password);
  }
  
  public fn logout(): void {
    this.repository.clearSession();
  }
  
  public fn getCurrentUser(): Option<User> {
    return this.repository.getCurrentUser();
  }
}
```

### 2. AuthRepository (src/auth/repositories/auth.repository.vela)

```vela
// Archivo: src/auth/repositories/auth.repository.vela
import 'system:http'  // HttpClient (API interna de Vela)
import 'package:jwt'  // JWT library externa

@injectable(scope: Scope.Singleton)
repository AuthRepository {
  constructor(
    @inject private httpClient: HttpClient
  ) { }
  
  private currentUser: Option<User> = Option.None;
  
  public async fn findAll(): Promise<List<User>> {
    return this.httpClient.get("/users");
  }
  
  public async fn findById(id: String): Promise<Option<User>> {
    return this.httpClient.get("/users/${id}");
  }
  
  public async fn save(user: User): Promise<User> {
    return this.httpClient.post("/users", user);
  }
  
  public async fn delete(id: String): Promise<Bool> {
    return this.httpClient.delete("/users/${id}");
  }
  
  public async fn login(email: String, password: String): Promise<Result<User, Error>> {
    return this.httpClient.post("/auth/login", { email, password })
      .map(user => {
        this.currentUser = Option.Some(user);
        return Result.Ok(user);
      });
  }
  
  public fn clearSession(): void {
    this.currentUser = Option.None;
  }
  
  public fn getCurrentUser(): Option<User> {
    return this.currentUser;
  }
}
```

### 3. AuthGuard (src/auth/guards/auth.guard.vela)

```vela
// Archivo: src/auth/guards/auth.guard.vela
import 'system:http'  // ExecutionContext (API interna)
import 'module:auth/services'  // AuthService

@injectable
guard AuthGuard {
  constructor(@inject private authService: AuthService) { }
  
  async fn canActivate(context: ExecutionContext): Promise<Result<Bool, Error>> {
    return match this.authService.getCurrentUser() {
      Option.Some(_) => Promise.resolve(Result.Ok(true)),
      Option.None => Promise.resolve(Result.Ok(false))
    };
  }
}
```

### 4. LoginWidget (src/auth/widgets/login.widget.vela)

```vela
// Archivo: src/auth/widgets/login.widget.vela
import 'system:ui'              // Widget, Container, Column, TextField, Button, Text (API interna)
import 'system:reactive'        // signal (API interna)
import 'module:auth/services'   // AuthService

@injectable
widget LoginWidget extends StatefulWidget {
  constructor(@inject private authService: AuthService) { }
  
  state email: String = "";
  state password: String = "";
  state error: Option<String> = Option.None;
  state loading: Bool = false;
  
  fn init(): void {
    print("LoginWidget initialized");
  }
  
  fn dispose(): void {
    print("LoginWidget disposed");
  }
  
  fn handleLogin(): void {
    this.loading = true;
    this.error = Option.None;
    
    let result = this.authService.login(this.email, this.password);
    
    match result {
      Result.Ok(user) => {
        print("Login successful: ${user.name}");
        this.loading = false;
      },
      Result.Err(err) => {
        this.error = Option.Some(err);
        this.loading = false;
      }
    }
  }
  
  return Container {
    padding: 24,
    
    Column {
      spacing: 16,
      
      Text("Login", style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold)),
      
      TextField {
        value: email,
        placeholder: "Email",
        enabled: !loading.value
      },
      
      TextField {
        value: password,
        placeholder: "Password",
        obscureText: true,
        enabled: !loading.value
      },
      
      match error.value {
        Option.Some(msg) => Text(msg, style: TextStyle(color: Colors.red)),
        Option.None => Container {}
      },
      
      Button {
        label: loading.value ? "Loading..." : "Login",
        onClick: handleLogin,
        enabled: !loading.value
      }
    }
  };
}
```

### 5. AuthModule (src/auth/auth.module.vela)

```vela
// Archivo: src/auth/auth.module.vela
import 'module:auth/services'   // AuthService, AuthRepository
import 'module:auth/guards'     // AuthGuard
import 'module:auth/widgets'    // LoginWidget
import 'module:shared/http'     // HttpModule
import 'module:shared/logger'   // LoggerModule

// ⭐ MÓDULO FUNCIONAL (estilo Angular)
@module({
  // Elementos declarados internamente
  declarations: [
    AuthService,
    AuthRepository,
    AuthGuard,
    LoginWidget
  ],
  
  // Solo estos elementos son visibles fuera del módulo
  exports: [
    AuthService,      // Otros módulos pueden inyectar AuthService
    AuthGuard,        // Otros módulos pueden usar AuthGuard
    LoginWidget       // Otros módulos pueden usar LoginWidget
  ],
  
  // Providers para DI (disponibles dentro del módulo)
  providers: [
    AuthService,
    AuthRepository
  ],
  
  // Módulos externos necesarios
  imports: [
    HttpModule,    // Para hacer requests HTTP
    LoggerModule   // Para logging
  ]
})
public class AuthModule { }
```

---

## 🏗️ Módulo Raíz (AppModule)

### src/app.module.vela

```vela
// Archivo: src/app.module.vela
import 'module:auth'        // AuthModule
import 'module:users'       // UsersModule
import 'module:shared/http'    // HttpModule
import 'module:shared/logger'  // LoggerModule
import 'module:database'    // DatabaseModule

// ⭐ MÓDULO RAÍZ (con @container para DI)
@container
@module({
  imports: [
    // Módulos funcionales
    AuthModule,
    UsersModule,
    
    // Módulos compartidos
    HttpModule,
    LoggerModule,
    DatabaseModule
  ],
  
  providers: [
    // Providers globales (disponibles en toda la app)
  ],
  
  exports: []  // El módulo raíz normalmente no exporta nada
})
public class AppModule { }
```

### src/main.vela

```vela
// Archivo: src/main.vela
import 'library:vela/ui'      // App
import 'module:app'           // AppModule
import 'module:auth/widgets'  // LoginWidget

fn main(): void {
  // Bootstrapear la aplicación con el módulo raíz
  injector = Injector.create(AppModule);
  
  // Obtener el widget desde el injector
  loginWidget = injector.get<LoginWidget>();
  
  // Ejecutar la app
  App.run(loginWidget);
}
```

---

## 🔄 Flujo de Importación y Visibilidad

### Escenario 1: UsersModule usa AuthService

```vela
// Archivo: src/users/users.module.vela
import 'module:auth'          // AuthModule
import 'module:users/services'   // UserService
import 'module:users/widgets'    // UserListWidget

@module({
  declarations: [UserService, UserListWidget],
  exports: [UserService, UserListWidget],
  providers: [UserService],
  
  // ✅ Importar AuthModule para acceder a AuthService
  imports: [AuthModule]  // Ahora AuthService está disponible para inyectar
})
public class UsersModule { }
```

```vela
// Archivo: src/users/user.service.vela
import 'module:auth/services'  // AuthService - ✅ Disponible porque UsersModule importa AuthModule

@injectable
public class UserService {
  private authService: AuthService;
  
  constructor(@inject authService: AuthService) {  // ✅ Inyección funciona
    this.authService = authService;
  }
  
  public fn getCurrentUserId(): Option<Int> {
    return match this.authService.getCurrentUser() {
      Option.Some(user) => Option.Some(user.id),
      Option.None => Option.None
    };
  }
}
```

### Escenario 2: Elemento NO exportado es privado

```vela
// AuthRepository NO está en exports de AuthModule
// Por lo tanto, SOLO está disponible dentro de AuthModule

// ❌ ESTO FALLARÍA en UsersModule
import 'module:auth/services'  // ❌ Error: AuthRepository no está exportado por AuthModule

@injectable
public class UserService {
  constructor(@inject repository: AuthRepository) {  // ❌ Error de compilación
    // ...
  }
}
```

**Mensaje de error**:
```
Error: Cannot import 'AuthRepository' from 'AuthModule'
  AuthRepository is declared but not exported by AuthModule.
  Available exports: AuthService, AuthGuard, LoginWidget
  
  at src/users/user.service.vela:3:42
```

---

## 🔑 Reglas de Visibilidad

### 1. Dentro del Módulo
- ✅ Todos los elementos en `declarations` están disponibles
- ✅ Todos los elementos en `providers` pueden ser inyectados
- ✅ Todos los exports de módulos en `imports` están disponibles

### 2. Fuera del Módulo
- ✅ Solo elementos en `exports` son visibles
- ❌ Elementos no exportados son **privados** al módulo
- ❌ No se puede acceder directamente a providers internos

### 3. Validación en Tiempo de Compilación
```vela
@module({
  declarations: [A, B, C],
  exports: [D],  // ❌ Error: D no está en declarations
  providers: [E]  // ❌ Error: E no está en declarations
})
class MyModule { }
```

**Regla**: `exports` y `providers` deben ser **subconjuntos** de `declarations`

---

## 📦 Re-exports (Módulos Barrel)

### Patrón: Módulo que re-exporta otros módulos

```vela
// Archivo: src/shared/shared.module.vela
import 'module:shared/http'    // HttpModule
import 'module:shared/logger'  // LoggerModule
import 'module:shared/utils'   // UtilsModule

@module({
  imports: [HttpModule, LoggerModule, UtilsModule],
  
  // Re-exportar todos los módulos importados
  exports: [HttpModule, LoggerModule, UtilsModule]
})
public class SharedModule { }
```

**Uso**:
```vela
// Antes: Importar cada módulo individual
imports: [HttpModule, LoggerModule, UtilsModule]

// Después: Importar solo SharedModule
imports: [SharedModule]  // ✅ Acceso a todos los exports de Http, Logger, Utils
```

---

## 🎨 Patrones Comunes

### 1. Feature Module (Módulo de Funcionalidad)

```vela
@module({
  declarations: [/* componentes, servicios, guards */],
  exports: [/* elementos públicos */],
  providers: [/* servicios privados */],
  imports: [/* dependencias */]
})
public class FeatureModule { }
```

### 2. Shared Module (Módulo Compartido)

```vela
@module({
  declarations: [CommonButton, CommonInput, CommonCard],
  exports: [CommonButton, CommonInput, CommonCard],  // Todo es exportado
  providers: [],
  imports: []
})
public class SharedModule { }
```

### 3. Core Module (Módulo Singleton)

```vela
@module({
  declarations: [ApiClient, AuthInterceptor, ErrorHandler],
  exports: [ApiClient],
  providers: [
    { provide: ApiClient, scope: Scope.Singleton },
    { provide: ErrorHandler, scope: Scope.Singleton }
  ],
  imports: []
})
public class CoreModule {
  // Prevenir múltiples importaciones del CoreModule
  private static alreadyImported: Bool = false;
  
  constructor() {
    if (CoreModule.alreadyImported) {
      throw Error("CoreModule should only be imported once in AppModule");
    }
    state {
      CoreModule.alreadyImported = true;
    }
  }
}
```

---

## ⚙️ Integración con Sistema DI

### Módulo + Container

```vela
// AppModule es TANTO un @module COMO un @container
@container  // Sistema DI (inyector raíz)
@module({   // Sistema de módulos (organización)
  imports: [AuthModule, UsersModule],
  providers: [GlobalService]
})
public class AppModule { }
```

### Scope en Módulos

```vela
@module({
  providers: [
    // Scope explícito
    { provide: AuthService, scope: Scope.Singleton },
    
    // Factory provider
    {
      provide: Database,
      useFactory: fn() => Database(url: "mongodb://localhost"),
      scope: Scope.Singleton
    },
    
    // Alias provider
    { provide: ILogger, useClass: ConsoleLogger }
  ]
})
public class AuthModule { }
```

---

## 🚀 Compilación y Resolución

### Fase 1: Análisis de Módulos
1. Escanear todos los archivos `.vela`
2. Identificar decoradores `@module`
3. Construir grafo de dependencias de módulos

### Fase 2: Validación
1. Verificar que `exports` ⊆ `declarations`
2. Verificar que `providers` ⊆ `declarations`
3. Detectar dependencias circulares entre módulos
4. Verificar que imports referencien módulos válidos

### Fase 3: Resolución de Visibilidad
1. Para cada módulo, calcular elementos visibles (declarations + imports.exports)
2. Validar que imports sean accesibles
3. Generar tabla de símbolos por módulo

### Fase 4: Generación de Injector
1. Crear grafo de providers global
2. Resolver scopes y lifetimes
3. Generar código de inyección

---

## 📊 Comparación: Angular vs Vela

| Característica | Angular (TypeScript) | Vela | Notas |
|----------------|---------------------|------|-------|
| Decorador | `@NgModule` | `@module` | ✅ Similar |
| declarations | ✅ | ✅ | Componentes, directivas, pipes |
| exports | ✅ | ✅ | Visibilidad externa |
| providers | ✅ | ✅ | DI providers |
| imports | ✅ | ✅ | Módulos dependientes |
| bootstrap | ✅ | Usa `fn main()` | Diferente - más explícito |
| Lazy loading | ✅ | 🔮 Futuro | No en MVP 1.0 |
| Metadata | Runtime | Compile-time | ✅ Más eficiente en Vela |

---

## 🛠️ Tareas para Implementar

### EPIC-03E: Module System (Angular-style)

| Task ID | Descripción | Horas | Sprint |
|---------|-------------|-------|--------|
| TASK-035AB | Diseñar sintaxis de @module decorator | 24 | 13 |
| TASK-035AC | Implementar parser para @module metadata | 40 | 13 |
| TASK-035AD | Implementar module graph builder | 48 | 14 |
| TASK-035AE | Implementar validación de exports/providers | 32 | 14 |
| TASK-035AF | Implementar resolución de visibilidad | 56 | 15 |
| TASK-035AG | Detectar dependencias circulares | 32 | 15 |
| TASK-035AH | Integrar con sistema DI (@container) | 48 | 16 |
| TASK-035AI | Generar mensajes de error descriptivos | 24 | 16 |
| TASK-035AJ | Tests de module system | 56 | 16 |

**Total**: 9 tareas, **360 horas** (~9 semanas de 1 dev)

---

## ✅ Ventajas de este Enfoque

1. **Encapsulación**: Módulos ocultan implementación interna
2. **Reutilización**: Módulos son unidades portables
3. **Escalabilidad**: Proyectos grandes organizados en módulos
4. **Type-safety**: Validación en tiempo de compilación
5. **Familiar**: Desarrolladores Angular/NestJS se sentirán cómodos
6. **Testeable**: Módulos se pueden testear independientemente
7. **Tree-shaking**: Solo imports usados en bundle final

---

## 📖 Documentación Necesaria

1. **Guía de Módulos**: Cómo crear y organizar módulos
2. **Patrones de Módulos**: Feature, Shared, Core modules
3. **Troubleshooting**: Errores comunes de visibilidad
4. **Migration Guide**: Cómo organizar proyectos existentes

---

## 🔮 Futuro (Post-MVP)

- **Lazy Loading**: Cargar módulos bajo demanda
- **Dynamic Modules**: Módulos configurables en runtime
- **Module Federation**: Compartir módulos entre apps
- **Circular Dependency Detection**: Warnings avanzados

---

**FIN DEL DOCUMENTO**
