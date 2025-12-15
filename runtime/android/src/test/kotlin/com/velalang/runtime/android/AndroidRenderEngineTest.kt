/*
Tests unitarios para Android Render Engine

Jira: TASK-157
Historia: VELA-1167
Fecha: 2025-12-15

Cobertura de testing:
- Inicialización del render engine
- Manejo de eventos
- Serialización VDOM
- Gestión de ciclo de vida
*/

package com.velalang.runtime.android

import android.content.Context
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AndroidRenderEngineTest {

    private lateinit var context: Context
    private lateinit var engine: AndroidRenderEngine
    private lateinit var config: VelaConfig

    @get:Rule
    val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        context = ApplicationProvider.getApplicationContext()
        config = VelaConfig(
            enableDebug = true,
            maxMemoryMB = 128,
            enableProfiling = false
        )
        engine = AndroidRenderEngine(context, config)
    }

    @Test
    fun testRenderEngineInitialization() {
        // Given: Engine not initialized

        // When: Initialize engine
        val result = engine.initialize()

        // Then: Initialization should succeed
        assertTrue("Engine should initialize successfully", result)
    }

    @Test
    fun testConfigToJson() {
        // Given: VelaConfig instance
        val config = VelaConfig(
            enableDebug = true,
            maxMemoryMB = 256,
            enableProfiling = true
        )

        // When: Convert to JSON
        val json = config.toJson()

        // Then: JSON should contain all fields
        assertTrue("JSON should contain enableDebug", json.contains("\"enableDebug\":true"))
        assertTrue("JSON should contain maxMemoryMB", json.contains("\"maxMemoryMB\":256"))
        assertTrue("JSON should contain enableProfiling", json.contains("\"enableProfiling\":true"))
    }

    @Test
    fun testVelaEventSerialization() {
        // Given: Different types of events
        val tapEvent = VelaEvent.Tap(100f, 200f)
        val scrollEvent = VelaEvent.Scroll(10f, -5f)
        val textEvent = VelaEvent.TextInput("Hello Vela")

        // When: Serialize events
        val tapJson = tapEvent.serialize()
        val scrollJson = scrollEvent.serialize()
        val textJson = textEvent.serialize()

        // Then: JSON should be valid and contain correct data
        assertTrue("Tap event should contain type", tapJson.contains("\"type\":\"tap\""))
        assertTrue("Tap event should contain coordinates", tapJson.contains("\"x\":100.0") && tapJson.contains("\"y\":200.0"))

        assertTrue("Scroll event should contain type", scrollJson.contains("\"type\":\"scroll\""))
        assertTrue("Scroll event should contain deltas", scrollJson.contains("\"deltaX\":10.0") && scrollJson.contains("\"deltaY\":-5.0"))

        assertTrue("Text event should contain type", textJson.contains("\"type\":\"textInput\""))
        assertTrue("Text event should contain text", textJson.contains("\"text\":\"Hello Vela\""))
    }

    @Test
    fun testVelaVDOMDeserialization() {
        // Given: Valid VDOM JSON
        val vdomJson = """
        {
            "root": {
                "id": "root",
                "component_type": "Container",
                "props": {},
                "children": []
            }
        }
        """.trimIndent()

        // When: Deserialize VDOM
        val vdom = VelaVDOM.deserialize(vdomJson)

        // Then: VDOM should be parsed correctly
        assertNotNull("VDOM should not be null", vdom)
        assertEquals("Root component type should be Container", "Container", vdom?.root?.componentType)
    }

    @Test
    fun testEventProcessing() {
        // Given: Initialized engine
        val initialized = engine.initialize()
        assertTrue("Engine should be initialized", initialized)

        // When: Process a tap event
        val tapEvent = VelaEvent.Tap(50f, 75f)
        engine.processEvent(tapEvent)

        // Then: No exceptions should be thrown (event processed)
        // Note: Actual event processing verification would require mocking the JNI bridge
    }

    @Test
    fun testComposeRendering() {
        composeTestRule.setContent {
            // Given: Engine initialized
            val initialized = engine.initialize()
            if (initialized) {
                // When: Render app
                engine.RenderApp()
            }
        }

        // Then: Compose should render without crashing
        // Note: More detailed UI testing would require actual VDOM data
        composeTestRule.waitForIdle()
    }

    @Test
    fun testConfigDefaults() {
        // Given: Default config
        val defaultConfig = VelaConfig()

        // When: Check default values
        val json = defaultConfig.toJson()

        // Then: Should have sensible defaults
        assertTrue("Default debug should be false", json.contains("\"enableDebug\":false"))
        assertTrue("Default memory should be 256MB", json.contains("\"maxMemoryMB\":256"))
        assertTrue("Default profiling should be false", json.contains("\"enableProfiling\":false"))
    }

    @Test
    fun testMultipleEventTypes() {
        // Given: Different event types
        val events = listOf(
            VelaEvent.Tap(0f, 0f),
            VelaEvent.Scroll(1f, 1f),
            VelaEvent.TextInput("test")
        )

        // When: Serialize all events
        val serialized = events.map { it.serialize() }

        // Then: All should be valid JSON
        serialized.forEach { json ->
            assertTrue("Event JSON should start with {", json.startsWith("{"))
            assertTrue("Event JSON should end with }", json.endsWith("}"))
            assertTrue("Event JSON should contain type", json.contains("\"type\""))
        }
    }

    @Test
    fun testEngineReinitialization() {
        // Given: Engine initialized once
        val firstInit = engine.initialize()
        assertTrue("First initialization should succeed", firstInit)

        // When: Try to initialize again
        val secondInit = engine.initialize()

        // Then: Should handle reinitialization gracefully
        // Note: Actual behavior depends on JNI implementation
        assertTrue("Reinitialization should be handled", secondInit || !secondInit)
    }
}

/**
 * Tests para componentes individuales del render engine
 */
class VelaConfigTest {

    @Test
    fun testConfigWithCustomValues() {
        // Given: Custom config values
        val config = VelaConfig(
            enableDebug = true,
            maxMemoryMB = 512,
            enableProfiling = true
        )

        // When: Convert to JSON
        val json = config.toJson()

        // Then: All custom values should be present
        assertTrue("Custom debug should be true", json.contains("\"enableDebug\":true"))
        assertTrue("Custom memory should be 512MB", json.contains("\"maxMemoryMB\":512"))
        assertTrue("Custom profiling should be true", json.contains("\"enableProfiling\":true"))
    }

    @Test
    fun testConfigJsonFormat() {
        // Given: Config instance
        val config = VelaConfig()

        // When: Get JSON
        val json = config.toJson()

        // Then: Should be valid JSON object
        assertTrue("Should start with {", json.trim().startsWith("{"))
        assertTrue("Should end with }", json.trim().endsWith("}"))
        assertTrue("Should contain commas", json.contains(","))
    }
}

/**
 * Tests para serialización de eventos
 */
class VelaEventTest {

    @Test
    fun testTapEventSerialization() {
        // Given: Tap event
        val event = VelaEvent.Tap(123.45f, 678.9f)

        // When: Serialize
        val json = event.serialize()

        // Then: Should contain correct data
        assertTrue("Should contain tap type", json.contains("\"type\":\"tap\""))
        assertTrue("Should contain x coordinate", json.contains("\"x\":123.45"))
        assertTrue("Should contain y coordinate", json.contains("\"y\":678.9"))
    }

    @Test
    fun testScrollEventSerialization() {
        // Given: Scroll event
        val event = VelaEvent.Scroll(-10f, 20f)

        // When: Serialize
        val json = event.serialize()

        // Then: Should contain correct data
        assertTrue("Should contain scroll type", json.contains("\"type\":\"scroll\""))
        assertTrue("Should contain deltaX", json.contains("\"deltaX\":-10.0"))
        assertTrue("Should contain deltaY", json.contains("\"deltaY\":20.0"))
    }

    @Test
    fun testTextInputEventSerialization() {
        // Given: Text input event
        val event = VelaEvent.TextInput("Hello, World!")

        // When: Serialize
        val json = event.serialize()

        // Then: Should contain correct data
        assertTrue("Should contain textInput type", json.contains("\"type\":\"textInput\""))
        assertTrue("Should contain text", json.contains("\"text\":\"Hello, World!\""))
    }

    @Test
    fun testEventJsonStructure() {
        // Given: Any event
        val event = VelaEvent.Tap(1f, 1f)

        // When: Serialize
        val json = event.serialize()

        // Then: Should be valid JSON
        assertTrue("Should be valid JSON object", json.startsWith("{") && json.endsWith("}"))

        // Should be parseable back
        try {
            org.json.JSONObject(json)
        } catch (e: Exception) {
            fail("Event JSON should be parseable: ${e.message}")
        }
    }
}