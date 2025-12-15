/*
Android Render Engine para Vela

Implementación del render engine nativo para Android usando Jetpack Compose.
Este engine permite ejecutar aplicaciones Vela en dispositivos Android con
performance nativa y experiencia de usuario consistente.

Jira: TASK-157
Historia: VELA-1167
Fecha: 2025-12-15

Arquitectura:
- VelaAndroidBridge: Puente JNI entre Rust y Kotlin
- ComposeRenderer: Renderer basado en Jetpack Compose
- AndroidEventHandler: Procesador de eventos nativos
- AndroidRenderEngine: Motor principal coordinador
*/

package com.velalang.runtime.android

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.nio.charset.StandardCharsets
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject

/**
 * Motor principal de renderizado para Android.
 * Coordina el runtime de Vela con Jetpack Compose.
 */
class AndroidRenderEngine(
    private val context: Context,
    private val config: VelaConfig = VelaConfig()
) {
    private var runtimePtr: Long = 0L
    private val bridge = VelaAndroidBridge()

    init {
        System.loadLibrary("vela_android_runtime")
    }

    /**
     * Inicializa el runtime de Vela
     */
    fun initialize(): Boolean {
        try {
            val configBytes = config.toJson().toByteArray(StandardCharsets.UTF_8)
            runtimePtr = bridge.initializeRuntime(configBytes)
            return runtimePtr != 0L
        } catch (e: Exception) {
            android.util.Log.e("VelaAndroid", "Failed to initialize runtime", e)
            return false
        }
    }

    /**
     * Renderiza un frame usando Jetpack Compose
     */
    @Composable
    fun RenderApp() {
        val coroutineScope = rememberCoroutineScope()
        var vdom by remember { mutableStateOf<VelaVDOM?>(null) }

        // Cleanup cuando el composable se destruye
        DisposableEffect(Unit) {
            onDispose {
                if (runtimePtr != 0L) {
                    bridge.destroyRuntime(runtimePtr)
                    runtimePtr = 0L
                }
            }
        }

        // Loop de renderizado a 60 FPS
        LaunchedEffect(Unit) {
            while (true) {
                try {
                    val vdomBytes = vdom?.serialize()?.toByteArray(StandardCharsets.UTF_8)
                    val updates = bridge.renderFrame(runtimePtr, vdomBytes ?: byteArrayOf())
                    val updatesStr = String(updates, StandardCharsets.UTF_8)
                    vdom = VelaVDOM.deserialize(updatesStr)
                } catch (e: Exception) {
                    android.util.Log.e("VelaAndroid", "Render error", e)
                }
                delay(16) // ~60 FPS
            }
        }

        // Renderizar VDOM con manejo de eventos
        VelaEventHandler(runtimePtr, bridge) {
            vdom?.render()
        }
    }

    /**
     * Procesa un evento nativo de Android
     */
    fun processEvent(event: VelaEvent) {
        try {
            val eventBytes = event.serialize().toByteArray(StandardCharsets.UTF_8)
            bridge.processEvent(runtimePtr, eventBytes)
        } catch (e: Exception) {
            android.util.Log.e("VelaAndroid", "Event processing error", e)
        }
    }
}

/**
 * Puente JNI para comunicación con el runtime de Rust
 */
class VelaAndroidBridge {
    external fun initializeRuntime(config: ByteArray): Long
    external fun renderFrame(runtimePtr: Long, vdom: ByteArray?): ByteArray
    external fun processEvent(runtimePtr: Long, event: ByteArray)
    external fun destroyRuntime(runtimePtr: Long)
}

/**
 * Manejador de eventos para componentes Vela
 */
@Composable
fun VelaEventHandler(
    runtimePtr: Long,
    bridge: VelaAndroidBridge,
    content: @Composable () -> Unit
) {
    val coroutineScope = rememberCoroutineScope()

    Box(modifier = Modifier
        .pointerInput(Unit) {
            detectTapGestures { offset ->
                val event = VelaEvent.Tap(offset.x, offset.y)
                coroutineScope.launch {
                    try {
                        val eventBytes = event.serialize().toByteArray(StandardCharsets.UTF_8)
                        bridge.processEvent(runtimePtr, eventBytes)
                    } catch (e: Exception) {
                        android.util.Log.e("VelaAndroid", "Event handling error", e)
                    }
                }
            }
        }
    ) {
        content()
    }
}

/**
 * Configuración del runtime de Vela
 */
data class VelaConfig(
    val enableDebug: Boolean = false,
    val maxMemoryMB: Int = 256,
    val enableProfiling: Boolean = false
) {
    fun toJson(): String = """
        {
            "enableDebug": $enableDebug,
            "maxMemoryMB": $maxMemoryMB,
            "enableProfiling": $enableProfiling
        }
    """.trimIndent()
}

/**
 * Representación del Virtual DOM de Vela
 */
data class VelaVDOM(
    val root: VelaNode
) {
    fun serialize(): String = root.serialize()

    @Composable
    fun render() = root.render()

    companion object {
        private val json = Json {
            ignoreUnknownKeys = true
            isLenient = true
        }

        fun deserialize(jsonString: String): VelaVDOM? {
            return try {
                // Parsear el JSON y crear el árbol de nodos
                val jsonElement = json.parseToJsonElement(jsonString)
                val rootNode = parseNode(jsonElement)
                rootNode?.let { VelaVDOM(it) }
            } catch (e: Exception) {
                android.util.Log.e("VelaVDOM", "Deserialization error", e)
                null
            }
        }

        private fun parseNode(element: kotlinx.serialization.json.JsonElement): VelaNode? {
            return try {
                when {
                    // Detectar tipo de nodo por campos presentes
                    element.jsonObject.containsKey("text") && !element.jsonObject.containsKey("children") ->
                        json.decodeFromJsonElement(TextNode.serializer(), element)

                    element.jsonObject.containsKey("children") ->
                        json.decodeFromJsonElement(ContainerNode.serializer(), element)

                    element.jsonObject.containsKey("onClick") ->
                        json.decodeFromJsonElement(ButtonNode.serializer(), element)

                    element.jsonObject.containsKey("url") ->
                        json.decodeFromJsonElement(ImageNode.serializer(), element)

                    element.jsonObject.containsKey("onValueChange") ->
                        json.decodeFromJsonElement(TextFieldNode.serializer(), element)

                    else -> {
                        android.util.Log.w("VelaVDOM", "Unknown node type: ${element}")
                        null
                    }
                }
            } catch (e: Exception) {
                android.util.Log.e("VelaVDOM", "Node parsing error", e)
                null
            }
        }
    }
}

/**
 * Nodo del Virtual DOM
 */
interface VelaNode {
    fun serialize(): String

    @Composable
    fun render()
}

/**
 * Eventos que pueden ocurrir en la UI
 */
sealed class VelaEvent {
    data class Tap(val x: Float, val y: Float) : VelaEvent()
    data class Scroll(val deltaX: Float, val deltaY: Float) : VelaEvent()
    data class TextInput(val text: String) : VelaEvent()

    fun serialize(): String = when (this) {
        is Tap -> """{"type":"tap","x":$x,"y":$y}"""
        is Scroll -> """{"type":"scroll","deltaX":$deltaX,"deltaY":$deltaY}"""
        is TextInput -> """{"type":"textInput","text":"$text"}"""
    }
}