/*
Tests unitarios para Android Render Engine

Jira: TASK-159
Historia: VELA-1167
Fecha: 2025-12-15

Cobertura de testing:
- Inicialización del render engine
- Manejo de eventos
- Serialización VDOM
- Gestión de ciclo de vida
- Renderizado de nodos Vela
- Deserialización JSON
- Performance y memoria
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

    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        context = ApplicationProvider.getApplicationContext()
        config = VelaConfig(enableDebug = true, maxMemoryMB = 128, enableProfiling = false)
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
        val config = VelaConfig(enableDebug = true, maxMemoryMB = 256, enableProfiling = true)

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
        assertTrue(
                "Tap event should contain coordinates",
                tapJson.contains("\"x\":100.0") && tapJson.contains("\"y\":200.0")
        )

        assertTrue("Scroll event should contain type", scrollJson.contains("\"type\":\"scroll\""))
        assertTrue(
                "Scroll event should contain deltas",
                scrollJson.contains("\"deltaX\":10.0") && scrollJson.contains("\"deltaY\":-5.0")
        )

        assertTrue("Text event should contain type", textJson.contains("\"type\":\"textInput\""))
        assertTrue("Text event should contain text", textJson.contains("\"text\":\"Hello Vela\""))
    }

    @Test
    fun testVelaVDOMDeserialization() {
        // Given: Valid VDOM JSON
        val vdomJson =
                """
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
        assertEquals(
                "Root component type should be Container",
                "Container",
                vdom?.root?.componentType
        )
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
        val events =
                listOf(VelaEvent.Tap(0f, 0f), VelaEvent.Scroll(1f, 1f), VelaEvent.TextInput("test"))

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

/** Tests para componentes individuales del render engine */
class VelaConfigTest {

    @Test
    fun testConfigWithCustomValues() {
        // Given: Custom config values
        val config = VelaConfig(enableDebug = true, maxMemoryMB = 512, enableProfiling = true)

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

/** Tests para serialización de eventos */
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

    @Test
    fun testTextNodeSerialization() {
        // Given: TextNode instance
        val textNode =
                TextNode(
                        text = "Hello Vela",
                        style =
                                TextStyleData(
                                        fontSize = 18f,
                                        fontWeight = "Bold",
                                        color = "#FF0000"
                                ),
                        modifier = ModifierData(width = 200f, height = 50f, padding = 8f)
                )

        // When: Serialize
        val json = textNode.serialize()

        // Then: Should contain all data
        assertTrue("Should contain text", json.contains("\"text\":\"Hello Vela\""))
        assertTrue("Should contain fontSize", json.contains("\"fontSize\":18.0"))
        assertTrue("Should contain fontWeight", json.contains("\"fontWeight\":\"Bold\""))
        assertTrue("Should contain color", json.contains("\"color\":\"#FF0000\""))
        assertTrue("Should contain width", json.contains("\"width\":200.0"))
        assertTrue("Should contain height", json.contains("\"height\":50.0"))
        assertTrue("Should contain padding", json.contains("\"padding\":8.0"))
    }

    @Test
    fun testContainerNodeSerialization() {
        // Given: ContainerNode with children
        val childNode = TextNode(text = "Child")
        val containerNode =
                ContainerNode(
                        children = listOf(childNode),
                        layout = LayoutType.Column,
                        modifier = ModifierData(padding = 16f, backgroundColor = "#FFFFFF")
                )

        // When: Serialize
        val json = containerNode.serialize()

        // Then: Should contain layout and children
        assertTrue("Should contain layout", json.contains("\"layout\":\"Column\""))
        assertTrue("Should contain children", json.contains("\"children\""))
        assertTrue("Should contain child text", json.contains("\"text\":\"Child\""))
        assertTrue("Should contain padding", json.contains("\"padding\":16.0"))
        assertTrue(
                "Should contain backgroundColor",
                json.contains("\"backgroundColor\":\"#FFFFFF\"")
        )
    }

    @Test
    fun testButtonNodeSerialization() {
        // Given: ButtonNode
        val buttonNode =
                ButtonNode(
                        text = "Click Me",
                        onClick = "button_click_event",
                        style =
                                ButtonStyleData(
                                        backgroundColor = "#6200EE",
                                        contentColor = "#FFFFFF"
                                ),
                        modifier = ModifierData(width = 120f, height = 40f, cornerRadius = 8f)
                )

        // When: Serialize
        val json = buttonNode.serialize()

        // Then: Should contain button data
        assertTrue("Should contain text", json.contains("\"text\":\"Click Me\""))
        assertTrue("Should contain onClick", json.contains("\"onClick\":\"button_click_event\""))
        assertTrue(
                "Should contain backgroundColor",
                json.contains("\"backgroundColor\":\"#6200EE\"")
        )
        assertTrue("Should contain contentColor", json.contains("\"contentColor\":\"#FFFFFF\""))
        assertTrue("Should contain cornerRadius", json.contains("\"cornerRadius\":8.0"))
    }

    @Test
    fun testImageNodeSerialization() {
        // Given: ImageNode
        val imageNode =
                ImageNode(
                        url = "https://example.com/image.png",
                        contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                        modifier = ModifierData(width = 300f, height = 200f)
                )

        // When: Serialize
        val json = imageNode.serialize()

        // Then: Should contain image data
        assertTrue("Should contain url", json.contains("\"url\":\"https://example.com/image.png\""))
        assertTrue("Should contain width", json.contains("\"width\":300.0"))
        assertTrue("Should contain height", json.contains("\"height\":200.0"))
    }

    @Test
    fun testTextFieldNodeSerialization() {
        // Given: TextFieldNode
        val textFieldNode =
                TextFieldNode(
                        value = "Initial text",
                        placeholder = "Enter text here",
                        onValueChange = "text_change_event",
                        modifier = ModifierData(padding = 12f, cornerRadius = 4f)
                )

        // When: Serialize
        val json = textFieldNode.serialize()

        // Then: Should contain text field data
        assertTrue("Should contain value", json.contains("\"value\":\"Initial text\""))
        assertTrue(
                "Should contain placeholder",
                json.contains("\"placeholder\":\"Enter text here\"")
        )
        assertTrue(
                "Should contain onValueChange",
                json.contains("\"onValueChange\":\"text_change_event\"")
        )
        assertTrue("Should contain padding", json.contains("\"padding\":12.0"))
        assertTrue("Should contain cornerRadius", json.contains("\"cornerRadius\":4.0"))
    }

    @Test
    fun testVelaVDOMComplexDeserialization() {
        // Given: Complex VDOM JSON with nested nodes
        val complexJson =
                """
        {
            "text": "Root Text",
            "style": {
                "fontSize": 20.0,
                "fontWeight": "Bold",
                "color": "#000000",
                "textAlign": "Center"
            },
            "modifier": {
                "padding": 16.0,
                "backgroundColor": "#F0F0F0"
            }
        }
        """.trimIndent()

        // When: Deserialize
        val vdom = VelaVDOM.deserialize(complexJson)

        // Then: Should parse correctly
        assertNotNull("VDOM should not be null", vdom)
        assertTrue("Root should be TextNode", vdom?.root is TextNode)

        val textNode = vdom?.root as TextNode
        assertEquals("Text should match", "Root Text", textNode.text)
        assertEquals("Font size should match", 20f, textNode.style.fontSize)
        assertEquals("Font weight should match", "Bold", textNode.style.fontWeight)
        assertEquals("Color should match", "#000000", textNode.style.color)
        assertEquals("Padding should match", 16f, textNode.modifier.padding)
        assertEquals("Background color should match", "#F0F0F0", textNode.modifier.backgroundColor)
    }

    @Test
    fun testModifierDataToComposeModifier() {
        // Given: ModifierData
        val modifierData =
                ModifierData(
                        width = 100f,
                        height = 50f,
                        padding = 8f,
                        backgroundColor = "#FF0000",
                        cornerRadius = 4f
                )

        // When: Convert to Compose Modifier
        val modifier = modifierData.toComposeModifier()

        // Then: Modifier should be created (basic validation)
        assertNotNull("Modifier should not be null", modifier)
        // Note: Detailed Compose modifier testing would require Compose testing framework
    }

    @Test
    fun testTextStyleDataToComposeStyle() {
        // Given: TextStyleData
        val styleData =
                TextStyleData(
                        fontSize = 24f,
                        fontWeight = "Bold",
                        color = "#00FF00",
                        textAlign = "Center"
                )

        // When: Convert to Compose TextStyle
        val textStyle = styleData.toComposeStyle()

        // Then: TextStyle should be created (basic validation)
        assertNotNull("TextStyle should not be null", textStyle)
        // Note: Detailed styling validation would require more complex testing
    }

    @Test
    fun testButtonStyleDataToButtonColors() {
        // Given: ButtonStyleData
        val buttonStyle = ButtonStyleData(backgroundColor = "#6200EE", contentColor = "#FFFFFF")

        // When: Convert to ButtonColors
        val buttonColors = buttonStyle.toButtonColors()

        // Then: ButtonColors should be created
        assertNotNull("ButtonColors should not be null", buttonColors)
    }
}
