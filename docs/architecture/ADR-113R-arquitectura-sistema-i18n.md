# ADR-113R: Arquitectura del Sistema de Internacionalización (i18n)

## Estado
✅ Aceptado

## Fecha
2025-01-08

## Contexto
El proyecto Vela requiere un sistema completo de internacionalización (i18n) para apoyar aplicaciones globales. Los usuarios necesitan:

- Soporte para múltiples idiomas y locales
- Traducciones dinámicas con interpolación de variables
- Formateo de fechas, números y monedas según locale
- Sistema de decoradores para clases de traducción
- Recarga en caliente de traducciones durante desarrollo
- Fallback automático a locales base cuando traducciones faltan

Este sistema debe integrarse perfectamente con el paradigma funcional puro de Vela y su sistema de módulos.

## Decisión
Implementaremos un sistema de i18n modular con la siguiente arquitectura:

### 1. Estructura del Sistema i18n

```
vela-i18n/
├── src/
│   ├── locale.rs          # Gestión de locales y configuración
│   ├── loader.rs          # Carga de archivos de traducción (JSON/YAML)
│   ├── interpolator.rs    # Interpolación de strings con variables
│   ├── formatter.rs       # Formateo de fechas/números/monedas
│   ├── translator.rs      # API principal de traducción
│   ├── decorator.rs       # Sistema de decoradores @i18n
│   ├── hot_reload.rs      # Recarga en caliente
│   └── pluralization.rs   # Reglas de pluralización por idioma
├── translations/
│   ├── en.json
│   ├── es.json
│   └── pt.json
└── tests/
    ├── locale_tests.rs
    ├── interpolation_tests.rs
    └── formatter_tests.rs
```

### 2. API Principal

```rust
// API principal del traductor
pub struct Translator {
    locale: Locale,
    fallback_locale: Locale,
    translations: HashMap<Locale, TranslationMap>,
}

// Decorador @i18n para clases
#[i18n]
pub struct MessageService {
    translator: Translator,
}

// Interpolación con variables
let message = translator.translate("welcome.user", &[
    ("name", "Alice"),
    ("count", "5")
]);

// Formateo localizado
let date = formatter.format_date(date, locale);
let number = formatter.format_number(1234.56, locale);
let currency = formatter.format_currency(99.99, "USD", locale);
```

### 3. Sistema de Locales

- **Locale**: Estructura que representa un locale (ej: "en-US", "es-ES")
- **LocaleManager**: Gestiona locales activos y fallback
- **LocaleConfig**: Configuración de formato por locale

### 4. Sistema de Traducciones

- **TranslationLoader**: Carga archivos JSON/YAML
- **TranslationCache**: Cache de traducciones cargadas
- **FallbackChain**: Cadena de fallback (es-ES → es → en)

### 5. Interpolación Avanzada

- Variables simples: `{{name}}`
- Variables con formato: `{{count, number}}`
- Pluralización: `{{count, plural, one{# item} other{# items}}}`
- Select: `{{gender, select, male{él} female{ella} other{elle}}}`

### 6. Formateo Localizado

- **DateTime**: Formatos cortos, largos, completos
- **Numbers**: Separadores de miles, decimales
- **Currency**: Símbolos, posiciones, formatos
- **Lists**: Separadores locales ("y", "o", etc.)

### 7. Decoradores

```rust
#[i18n(locale = "en-US", fallback = "en")]
pub struct AuthMessages {
    pub welcome: String,
    pub error_invalid_credentials: String,
}

#[i18n(hot_reload = true)]
pub struct UIMessages {
    pub button_save: String,
    pub button_cancel: String,
}
```

### 8. Hot Reload

- **FileWatcher**: Monitorea cambios en archivos de traducción
- **HotReloadManager**: Recarga traducciones automáticamente
- **DevMode**: Modo desarrollo con recarga automática

## Consecuencias

### Positivas
- ✅ Soporte completo para aplicaciones globales
- ✅ Integración perfecta con sistema de módulos de Vela
- ✅ API declarativa con decoradores
- ✅ Performance optimizada con cache y lazy loading
- ✅ Desarrollo ágil con hot reload
- ✅ Type safety en traducciones
- ✅ Fallback automático para traducciones faltantes

### Negativas
- ❌ Complejidad adicional en el compilador para decoradores
- ❌ Dependencia de crates externos para formato localizado
- ❌ Overhead de memoria para cache de traducciones
- ❌ Curva de aprendizaje para pluralización compleja

## Alternativas Consideradas

### 1. Sistema Basado en Macros (Rechazada)
**Decisión**: Macros en lugar de structs con decoradores
**Razones de rechazo**:
- Menos legible que structs con decoradores
- Más difícil de debuggear
- No aprovecha el sistema de tipos de Vela

### 2. Traducciones en Base de Datos (Rechazada)
**Decisión**: Almacenar traducciones en BD en lugar de archivos
**Razones de rechazo**:
- Aumenta complejidad de deployment
- Menos adecuado para desarrollo local
- Dificulta versionado de traducciones

### 3. Sistema Sin Fallback (Rechazada)
**Decisión**: Sin cadena de fallback automática
**Razones de rechazo**:
- Fuerza traducciones 100% completas
- UX pobre cuando faltan traducciones
- Más trabajo para mantenedores

## Implementación
Ver código en: `src/i18n/`

## Referencias
- Jira: [VELA-598](https://velalang.atlassian.net/browse/VELA-598)
- Task: [TASK-113R](https://velalang.atlassian.net/browse/TASK-113R)
- Documentación: [Guía de i18n](../features/VELA-598/README.md)