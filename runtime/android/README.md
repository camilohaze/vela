# Vela Android Runtime

Render engine nativo para ejecutar aplicaciones Vela en Android usando Jetpack Compose.

## 📋 Información General

- **Versión:** 0.1.0
- **API Level:** 21+ (Android 5.0+)
- **Arquitectura:** JNI Bridge (Rust ↔ Kotlin)
- **UI Framework:** Jetpack Compose
- **Performance Target:** 60 FPS

## 🏗️ Arquitectura

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

## 🚀 Inicio Rápido

### 1. Agregar dependencia

```kotlin
dependencies {
    implementation 'com.velalang:runtime-android:0.1.0'
}
```

### 2. Inicializar el engine

```kotlin
class MainActivity : ComponentActivity() {
    private lateinit var velaEngine: AndroidRenderEngine

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Configurar engine
        val config = VelaConfig(
            enableDebug = BuildConfig.DEBUG,
            maxMemoryMB = 256
        )

        velaEngine = AndroidRenderEngine(this, config)

        // Inicializar
        if (velaEngine.initialize()) {
            setContent {
                velaEngine.RenderApp()
            }
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        // El engine se limpia automáticamente via DisposableEffect
    }
}
```

### 3. Ejecutar aplicación Vela

```kotlin
// En tu aplicación Vela
@component
class MyApp {
    state counter: Number = 0

    render() {
        Column {
            Text("Contador: ${counter}")
            Button(text = "Incrementar") {
                counter = counter + 1
            }
        }
    }
}
```

## 📁 Estructura del Proyecto

```
runtime/android/
├── src/main/
│   ├── kotlin/com/velalang/runtime/android/
│   │   └── AndroidRenderEngine.kt      # Motor principal
│   ├── rust/
│   │   └── jni_bridge.rs               # Puente JNI
│   └── AndroidManifest.xml
├── src/test/kotlin/.../                # Tests unitarios
├── build.gradle.kts                    # Configuración Gradle
└── Cargo.toml                         # Dependencias Rust
```

## 🔧 Configuración

### VelaConfig

```kotlin
data class VelaConfig(
    val enableDebug: Boolean = false,        // Logging detallado
    val maxMemoryMB: Int = 256,              // Memoria máxima
    val enableProfiling: Boolean = false     // Perfilado de performance
)
```

### Build Types

```kotlin
android {
    buildTypes {
        debug {
            // Configuración para desarrollo
            velaConfig.enableDebug = true
        }
        release {
            // Configuración para producción
            velaConfig.enableProfiling = false
            minifyEnabled = true
        }
    }
}
```

## 🎯 API Reference

### AndroidRenderEngine

```kotlin
class AndroidRenderEngine(
    context: Context,
    config: VelaConfig = VelaConfig()
) {
    // Inicializa el runtime
    fun initialize(): Boolean

    // Renderiza la aplicación Vela
    @Composable
    fun RenderApp()

    // Procesa eventos nativos
    fun processEvent(event: VelaEvent)
}
```

### VelaEvent

```kotlin
sealed class VelaEvent {
    data class Tap(val x: Float, val y: Float) : VelaEvent()
    data class Scroll(val deltaX: Float, val deltaY: Float) : VelaEvent()
    data class TextInput(val text: String) : VelaEvent()
}
```

## 🧪 Testing

### Unit Tests

```kotlin
@RunWith(AndroidJUnit4::class)
class AndroidRenderEngineTest {

    @Test
    fun testRenderEngineInitialization() {
        val engine = AndroidRenderEngine(context, config)
        assertTrue(engine.initialize())
    }
}
```

### UI Tests con Compose

```kotlin
@Test
fun testVelaAppRendering() {
    composeTestRule.setContent {
        engine.RenderApp()
    }

    composeTestRule.onNodeWithText("Contador")
        .assertIsDisplayed()
}
```

## 🔨 Build y Deployment

### Comando de build

```bash
# Build completo
vela build --target=android

# Build solo librería
./gradlew :runtime:android:assembleDebug

# Build con profiling
./gradlew :runtime:android:assembleRelease
```

### Generar AAB/APK

```bash
# Debug APK
./gradlew :runtime:android:assembleDebug

# Release AAB (para Play Store)
./gradlew :runtime:android:bundleRelease
```

### Dependencias nativas

El runtime incluye librerías JNI compiladas para:
- `arm64-v8a` (64-bit ARM)
- `armeabi-v7a` (32-bit ARM)
- `x86` (Intel x86)
- `x86_64` (Intel x64)

## 📊 Performance

### Métricas objetivo

- **Frame Rate:** 60 FPS estable
- **Memory Usage:** < 50MB base + app size
- **Startup Time:** < 2 segundos en dispositivos modernos
- **Battery Impact:** Mínimo (similar a apps nativas)

### Profiling

```kotlin
val config = VelaConfig(
    enableProfiling = true,
    enableDebug = true
)
// Logs detallados en logcat con tag "VelaAndroid"
```

## 🐛 Troubleshooting

### Problemas comunes

#### 1. Library not found
```
java.lang.UnsatisfiedLinkError: dalvik.system.PathClassLoader...
```
**Solución:** Verificar que `libvela_android_runtime.so` esté incluido en APK.

#### 2. Runtime initialization failed
```
Runtime initialization failed
```
**Solución:** Verificar permisos y configuración de memoria.

#### 3. Rendering performance issues
**Solución:**
- Reducir `maxMemoryMB`
- Deshabilitar `enableDebug` en release
- Verificar target API level

### Debug logging

```kotlin
// Habilitar logs detallados
val config = VelaConfig(enableDebug = true)
// Ver logs en: adb logcat | grep VelaAndroid
```

## 🔗 Referencias

### Documentación
- [ADR-157: Android Render Engine Architecture](../docs/architecture/ADR-157-android-render-engine.md)
- [TASK-157: Implementation Details](../docs/features/VELA-1167/TASK-157.md)

### Tecnologías
- [Jetpack Compose](https://developer.android.com/jetpack/compose)
- [JNI Documentation](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/)
- [Rust Android](https://mozilla.github.io/rust-android/)

### Jira
- [VELA-1167: Android Deployment](https://velalang.atlassian.net/browse/VELA-1167)
- [TASK-157: Android Render Engine](https://velalang.atlassian.net/browse/TASK-157)

---

**Versión:** 0.1.0
**Última actualización:** 2025-12-15
**Estado:** ✅ Completo y listo para uso