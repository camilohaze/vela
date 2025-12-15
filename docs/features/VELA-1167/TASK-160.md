# TASK-160: Implementar vela build --target=android

## 📋 Información General
- **Historia:** VELA-1167
- **Estado:** En curso ✅
- **Fecha:** 2025-12-15

## 🎯 Objetivo
Implementar el comando `vela build --target=android` que genera un proyecto Android completo con Gradle, código Kotlin, y integración con el runtime Android de Vela.

## 🔨 Implementación

### Arquitectura del Build System

```
vela build --target=android
├── BuildExecutor.generate_android_artifacts()
│   ├── build.gradle.kts (app module)
│   ├── settings.gradle.kts (workspace)
│   ├── AndroidManifest.xml
│   ├── MainActivity.kt (Kotlin app)
│   ├── runtime-android/ (copia del runtime)
│   └── bytecode/ (archivos .velac compilados)
```

### Componentes Generados

#### 1. build.gradle.kts
Configuración completa de Gradle para la aplicación Android:
- Plugins: application, kotlin-android, serialization
- Configuración de Android (minSdk 21, targetSdk 34)
- Dependencias: Jetpack Compose, Kotlinx Serialization, Coil
- Integración con runtime-android module

#### 2. settings.gradle.kts
Configuración del workspace Gradle:
- Incluye módulos: app y runtime-android
- Repositorios: Google, Maven Central

#### 3. AndroidManifest.xml
Manifiesto Android con:
- Permisos: INTERNET, ACCESS_NETWORK_STATE
- Activity principal: MainActivity
- Configuración de backup y temas

#### 4. MainActivity.kt
Actividad principal de Android:
- Inicialización del AndroidRenderEngine
- Integración con Jetpack Compose
- Manejo del ciclo de vida de Vela

#### 5. Runtime Android Integration
- Copia completa del proyecto runtime/android
- Configuración como módulo Gradle
- JNI libraries y código nativo

#### 6. Bytecode Integration
- Copia de archivos .velac compilados
- Assets empaquetados en APK

### Flujo de Build Completo

```bash
# 1. Compilar código Vela a bytecode
vela build --target=android

# Resultado en target/android/:
target/android/
├── build.gradle.kts          # Config Gradle app
├── settings.gradle.kts       # Config workspace
├── src/main/
│   ├── AndroidManifest.xml
│   └── kotlin/com/velalang/app/
│       └── MainActivity.kt
├── runtime-android/          # Runtime copiado
└── assets/                   # Bytecode .velac
```

### Integración con Gradle

El proyecto generado puede compilarse con:

```bash
cd target/android
./gradlew build
./gradlew installDebug  # Instalar en dispositivo
```

### Dependencias del Runtime

El build system asegura que:
- Runtime Android esté disponible como módulo
- JNI libraries sean incluidas
- Todas las dependencias Kotlin/Java estén resueltas
- Compose compiler esté configurado

## ✅ Criterios de Aceptación
- [x] Comando `vela build --target=android` funciona
- [x] Proyecto Gradle válido generado
- [x] MainActivity.kt integra correctamente con AndroidRenderEngine
- [x] Runtime Android copiado e incluido como módulo
- [x] Bytecode .velac empaquetado en assets
- [x] `./gradlew build` exitoso
- [x] APK generable e instalable
- [x] Tests de integración pasan

## 🔗 Referencias
- **Jira:** [TASK-160](https://velalang.atlassian.net/browse/TASK-160)
- **Historia:** [VELA-1167](https://velalang.atlassian.net/browse/VELA-1167)
- **Dependencias:** TASK-157 (Android render engine), TASK-158 (JNI bridging), TASK-159 (Android renderer)</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-1167\TASK-160.md