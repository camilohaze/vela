# TASK-113R: Arquitectura del Sistema de Internacionalización (i18n)

## 📋 Información General
- **Historia:** VELA-598 - Sistema de Internacionalización para Aplicaciones Globales
- **Estado:** Completada ✅
- **Fecha:** 2025-01-08
- **Tipo:** Decisión Arquitectónica + Implementación

## 🎯 Objetivo
Diseñar e implementar la arquitectura completa del sistema de internacionalización (i18n) para Vela, incluyendo gestión de locales, carga de traducciones, interpolación de variables, formateo localizado y sistema de decoradores con hot reload.

## 🔨 Implementación

### Arquitectura Implementada

#### 1. **Sistema de Locales** (`locale.rs`)
- **Locale**: Estructura que representa identificadores de locale (ej: "en-US", "es-ES")
- **LocaleManager**: Gestiona configuraciones específicas por locale
- **LocaleConfig**: Configuraciones de formato (fechas, números, monedas, listas)
- **Soporte de fallback**: Cadena automática de fallback (es-ES → es → en)

#### 2. **Carga de Traducciones** (`loader.rs`)
- **TranslationLoader**: Carga archivos JSON/YAML con cache inteligente
- **TranslationMap**: Estructura de datos para traducciones anidadas
- **TranslationFile**: Metadatos y estructura de archivos de traducción
- **Soporte multi-formato**: JSON, YAML, YML

#### 3. **Interpolación Avanzada** (`interpolator.rs`)
- **InterpolationContext**: Contexto con variables y locale actual
- **Interpolator**: Motor de interpolación con regex patterns
- **Variables simples**: `{{name}}`
- **Variables con formato**: `{{price, currency}}`
- **Pluralización**: `{{count, plural, one{# item} other{# items}}}`
- **Select**: `{{gender, select, male{él} female{ella} other{elle}}}`

#### 4. **Formateo Localizado** (`formatter.rs`)
- **Formatter**: Formateo de fechas, números, monedas y listas
- **DateFormatStyle**: Estilos de formato de fecha (Short, Medium, Long, Full)
- **ListFormatStyle**: Separadores de lista ("and"/"or")
- **Utils**: Funciones auxiliares (relative time, file size)

#### 5. **API Principal** (`translator.rs`)
- **Translator**: API principal con async/await
- **TranslatorBuilder**: Constructor con patrón builder
- **Métodos principales**:
  - `translate()`: Traducción con variables
  - `translate_plural()`: Traducción con pluralización
  - `translate_or()`: Traducción con fallback a default
  - `has_key()`: Verificación de existencia de clave

#### 6. **Sistema de Decoradores** (`decorator.rs`)
- **I18nDecorator**: Decorador para clases con capacidades i18n
- **I18nClass**: Trait para clases decoradas
- **I18nClassWrapper**: Wrapper para clases decoradas
- **Registro automático**: Decoración de clases con metadatos

#### 7. **Hot Reload** (`hot_reload.rs`)
- **HotReloadManager**: Monitor de cambios en archivos
- **HotReloadConfig**: Configuración del hot reload
- **HotReloadEvent**: Eventos de cambio de traducción
- **Watcher loop**: Bucle de monitoreo con polling

#### 8. **Pluralización** (`pluralization.rs`)
- **PluralCategory**: Categorías de plural (Zero, One, Two, Few, Many, Other)
- **PluralRule**: Trait para reglas de pluralización por idioma
- **Reglas implementadas**: English, Spanish, French, German, Portuguese, Arabic, Russian, Japanese, Chinese
- **Utils**: Funciones auxiliares para parsing de especificaciones plurales

### Archivos Creados

#### Código Fuente (`src/`)
- `lib.rs` - Biblioteca principal con re-exports
- `error.rs` - Tipos de error unificados
- `locale.rs` - Gestión de locales y configuración
- `loader.rs` - Carga y cache de traducciones
- `interpolator.rs` - Interpolación de strings con variables
- `formatter.rs` - Formateo localizado
- `translator.rs` - API principal del traductor
- `decorator.rs` - Sistema de decoradores @i18n
- `hot_reload.rs` - Recarga en caliente
- `pluralization.rs` - Reglas de pluralización

#### Traducciones de Ejemplo (`translations/`)
- `en.json` - Traducciones en inglés
- `es.json` - Traducciones en español
- `pt.json` - Traducciones en portugués

#### Configuración (`Cargo.toml`)
- Dependencias: `serde`, `tokio`, `regex`, `chrono`, `rust_decimal`, `num-format`, `icu`
- Features: JSON/YAML parsing, async operations, localized formatting

### Decisiones Arquitectónicas

#### ✅ Patrón Builder para Translator
**Decisión**: Usar patrón Builder para configuración flexible
**Razones**:
- Configuración compleja con múltiples opciones
- API fluida y legible
- Validación de configuración en tiempo de construcción

#### ✅ Async/Await en Toda la API
**Decisión**: API completamente asíncrona
**Razones**:
- Operaciones de I/O (carga de archivos)
- Escalabilidad para aplicaciones web
- Consistencia con ecosistema Rust moderno

#### ✅ Sistema de Fallback Robusto
**Decisión**: Cadena de fallback automática (locale → language → en)
**Razones**:
- Traducciones incompletas no rompen la aplicación
- Desarrollo incremental de traducciones
- UX consistente aunque traducciones estén incompletas

#### ✅ Interpolación con Regex
**Decisión**: Usar regex para parsing de patrones de interpolación
**Razones**:
- Parsing robusto y eficiente
- Soporte para sintaxis compleja (pluralización, select)
- Extensible para futuras features

#### ✅ Traits para Extensibilidad
**Decisión**: Usar traits para reglas de pluralización y decoradores
**Razones**:
- Fácil agregar nuevos idiomas
- Testing independiente
- Composición flexible

### API de Uso

```rust
use vela_i18n::{Translator, Locale};

// Crear traductor
let translator = Translator::new();

// Cargar traducciones
translator.load_translations_from_dir("translations/").await?;

// Traducción simple
let greeting = translator.translate("greeting.hello", &[])?;

// Con variables
let welcome = translator.translate("messages.user_greeting", &[("name", "Alice")])?;

// Con pluralización
let items = translator.translate("messages.items", &[("count", "5")])?;

// Formateo localizado
let date = translator.formatter().format_date("2023-12-25", &locale)?;
let currency = translator.formatter().format_currency("29.99", "USD", &locale)?;
```

### Métricas de Implementación

- **Archivos creados**: 10 módulos Rust + 3 archivos de traducción + Cargo.toml
- **Líneas de código**: ~2,500+ líneas
- **Tests unitarios**: 50+ tests cubriendo todas las funcionalidades
- **Locales soportados**: 9 idiomas con reglas de pluralización
- **Formatos soportados**: JSON, YAML
- **Features implementadas**: Interpolación, pluralización, formateo, hot reload, decoradores

## ✅ Criterios de Aceptación
- [x] Arquitectura modular y extensible
- [x] Soporte completo para interpolación de variables
- [x] Sistema de pluralización por idioma
- [x] Formateo localizado (fechas, números, monedas)
- [x] API asíncrona con patrón builder
- [x] Sistema de decoradores @i18n
- [x] Hot reload para desarrollo
- [x] Fallback automático de locales
- [x] Tests unitarios completos
- [x] Documentación completa
- [x] Ejemplos de uso funcionales

## 🔗 Referencias
- **Jira:** [TASK-113R](https://velalang.atlassian.net/browse/TASK-113R)
- **Historia:** [VELA-598](https://velalang.atlassian.net/browse/VELA-598)
- **ADR:** [docs/architecture/ADR-113R-arquitectura-sistema-i18n.md](../architecture/ADR-113R-arquitectura-sistema-i18n.md)
- **Código:** `src/i18n/`
- **Tests:** `src/i18n/` (tests integrados)