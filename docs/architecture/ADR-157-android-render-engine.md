# ADR-157: Android Render Engine Architecture

## Estado
✅ Aceptado

## Fecha
2025-12-15

## Contexto
Necesitamos diseñar un render engine nativo para Android que permita ejecutar aplicaciones Vela en dispositivos Android. El engine debe integrar el runtime de Vela (Rust) con el sistema de UI nativo de Android (Jetpack Compose) de manera eficiente y performante.

## Decisión
Implementaremos un Android Render Engine basado en Jetpack Compose con bridging FFI entre Rust y Kotlin, siguiendo la arquitectura modular que probamos exitosamente en iOS.

### Arquitectura Elegida

```
┌─────────────────────────────────────────────────────────────┐
│                    Android Application                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌──────────────────┐    ┌─────────┐  │
│  │ Vela Runtime    │───▶│ Android Bridge   │───▶│ Compose │  │
│  │ (Rust)          │    │ (JNI/FFI)        │    │ Renderer│  │
│  └─────────────────┘    └──────────────────┘    └─────────┘  │
├─────────────────────────────────────────────────────────────┤
│                Android OS (JVM/Kotlin)                      │
└─────────────────────────────────────────────────────────────┘
```

#### Componentes Principales

1. **VelaAndroidBridge**: Puente FFI usando JNI para comunicación Rust ↔ Kotlin
2. **ComposeRenderer**: Renderer que convierte VDOM Vela a componentes Jetpack Compose
3. **AndroidEventHandler**: Procesador de eventos nativos Android → Vela
4. **AndroidRenderEngine**: Motor principal que coordina todos los componentes

## Consecuencias

### Positivas
- ✅ **Performance nativa**: Jetpack Compose ofrece excelente performance en Android
- ✅ **Consistencia con iOS**: Arquitectura similar permite compartir conocimientos
- ✅ **Ecosistema maduro**: Jetpack Compose es el standard moderno de Android
- ✅ **Reactividad**: Integración natural con el sistema reactivo de Vela
- ✅ **Mantenibilidad**: Separación clara de responsabilidades

### Negativas
- ❌ **Complejidad JNI**: Bridging Rust ↔ JVM requiere manejo cuidadoso de memoria
- ❌ **Dependencia de Kotlin**: Requiere Kotlin para el bridge (no solo Java)
- ❌ **Testing complejo**: Tests requieren emuladores Android o dispositivos

## Alternativas Consideradas

### 1. Render Engine Custom (Canvas-based)
**Descripción:** Implementar renderer propio usando Canvas API de Android
- **Pros:** Control total, sin dependencias externas
- **Cons:** Mucho trabajo, performance inferior, mantenimiento complejo
- **Rechazada porque:** Jetpack Compose ofrece mejor performance y DX

### 2. WebView-based Rendering
**Descripción:** Renderizar usando WebView con HTML/CSS generado desde Vela
- **Pros:** Reutilizar lógica web existente
- **Cons:** Performance inferior, no nativo, UX no consistente
- **Rechazada porque:** Queremos experiencia 100% nativa

### 3. Flutter Embedding
**Descripción:** Integrar Flutter como runtime intermedio
- **Pros:** Flutter tiene excelente Android support
- **Cons:** Añade complejidad innecesaria, dependencias grandes
- **Rechazada porque:** Queremos control directo sobre el rendering

## Implementación

### Fase 1: Android Bridge (JNI)
```kotlin
// Android side
class VelaAndroidBridge {
    external fun initializeRuntime(config: ByteArray): Long
    external fun renderFrame(runtimePtr: Long, vdom: ByteArray): ByteArray
    external fun processEvent(runtimePtr: Long, event: ByteArray)
    external fun destroyRuntime(runtimePtr: Long)
}
```

```rust
// Rust side
#[no_mangle]
pub extern "C" fn initialize_runtime(config_ptr: *const c_char) -> *mut VelaRuntime {
    // Initialize Vela runtime
}

#[no_mangle]
pub extern "C" fn render_frame(runtime: *mut VelaRuntime, vdom_ptr: *const c_char) -> *mut c_char {
    // Render frame and return updates
}
```

### Fase 2: Compose Renderer
```kotlin
@Composable
fun VelaApp(runtimePtr: Long) {
    val vdom by remember { mutableStateOf<VelaVDOM?>(null) }

    LaunchedEffect(Unit) {
        while (true) {
            val updates = renderFrame(runtimePtr, vdom?.serialize())
            vdom = VelaVDOM.deserialize(updates)
            delay(16) // 60 FPS
        }
    }

    vdom?.render()
}
```

### Fase 3: Event Handling
```kotlin
@Composable
fun VelaEventHandler(
    runtimePtr: Long,
    content: @Composable () -> Unit
) {
    val coroutineScope = rememberCoroutineScope()

    Box(modifier = Modifier
        .pointerInput(Unit) {
            detectTapGestures { offset ->
                val event = VelaEvent.Tap(offset.x, offset.y)
                coroutineScope.launch {
                    processEvent(runtimePtr, event.serialize())
                }
            }
        }
    ) {
        content()
    }
}
```

## Referencias
- Jira: [TASK-157](https://velalang.atlassian.net/browse/TASK-157)
- iOS Architecture: docs/architecture/ADR-152-ios-render-engine.md
- Jetpack Compose: https://developer.android.com/jetpack/compose
- JNI Best Practices: https://developer.android.com/training/articles/perf-jni