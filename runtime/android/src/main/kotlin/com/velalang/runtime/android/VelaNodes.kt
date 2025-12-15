/*
Implementaciones concretas de VelaNode para Android renderer

Jira: TASK-159
Historia: VELA-1167
Fecha: 2025-12-15
*/

package com.velalang.runtime.android

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.rememberAsyncImagePainter
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/**
 * Nodo de texto
 */
@Serializable
data class TextNode(
    val text: String,
    val style: TextStyleData = TextStyleData(),
    val modifier: ModifierData = ModifierData()
) : VelaNode {

    @Composable
    override fun render() {
        Text(
            text = text,
            style = style.toComposeStyle(),
            modifier = modifier.toComposeModifier()
        )
    }

    override fun serialize(): String = Json.encodeToString(serializer(), this)
}

/**
 * Nodo contenedor (layout)
 */
@Serializable
data class ContainerNode(
    val children: List<VelaNode>,
    val layout: LayoutType = LayoutType.Column,
    val modifier: ModifierData = ModifierData()
) : VelaNode {

    @Composable
    override fun render() {
        when (layout) {
            LayoutType.Column -> Column(modifier = modifier.toComposeModifier()) {
                children.forEach { child -> child.render() }
            }
            LayoutType.Row -> Row(modifier = modifier.toComposeModifier()) {
                children.forEach { child -> child.render() }
            }
            LayoutType.Box -> Box(modifier = modifier.toComposeModifier()) {
                children.forEach { child -> child.render() }
            }
            LayoutType.LazyColumn -> LazyColumn(modifier = modifier.toComposeModifier()) {
                items(children) { child -> child.render() }
            }
        }
    }

    override fun serialize(): String = Json.encodeToString(serializer(), this)
}

/**
 * Nodo botón
 */
@Serializable
data class ButtonNode(
    val text: String,
    val onClick: String, // ID del evento
    val style: ButtonStyleData = ButtonStyleData(),
    val modifier: ModifierData = ModifierData()
) : VelaNode {

    @Composable
    override fun render() {
        Button(
            onClick = { /* TODO: Trigger event */ },
            modifier = modifier.toComposeModifier(),
            colors = style.toButtonColors()
        ) {
            Text(text)
        }
    }

    override fun serialize(): String = Json.encodeToString(serializer(), this)
}

/**
 * Nodo imagen
 */
@Serializable
data class ImageNode(
    val url: String,
    val contentScale: ContentScale = ContentScale.Fit,
    val modifier: ModifierData = ModifierData()
) : VelaNode {

    @Composable
    override fun render() {
        Image(
            painter = rememberAsyncImagePainter(url),
            contentDescription = null,
            contentScale = contentScale,
            modifier = modifier.toComposeModifier()
        )
    }

    override fun serialize(): String = Json.encodeToString(serializer(), this)
}

/**
 * Nodo campo de texto
 */
@Serializable
data class TextFieldNode(
    val value: String,
    val placeholder: String = "",
    val onValueChange: String, // ID del evento
    val modifier: ModifierData = ModifierData()
) : VelaNode {

    @Composable
    override fun render() {
        var text by remember { mutableStateOf(value) }

        BasicTextField(
            value = text,
            onValueChange = {
                text = it
                // TODO: Trigger event
            },
            modifier = modifier.toComposeModifier(),
            decorationBox = { innerTextField ->
                if (text.isEmpty() && placeholder.isNotEmpty()) {
                    Text(text = placeholder, color = Color.Gray)
                }
                innerTextField()
            }
        )
    }

    override fun serialize(): String = Json.encodeToString(serializer(), this)
}

/**
 * Tipos de layout
 */
@Serializable
enum class LayoutType {
    Column, Row, Box, LazyColumn
}

/**
 * Datos de estilo de texto
 */
@Serializable
data class TextStyleData(
    val fontSize: Float = 16f,
    val fontWeight: String = "Normal",
    val color: String = "#000000",
    val textAlign: String = "Start"
) {
    fun toComposeStyle(): TextStyle {
        val color = Color(android.graphics.Color.parseColor(color))
        val weight = when (fontWeight) {
            "Bold" -> FontWeight.Bold
            "Light" -> FontWeight.Light
            else -> FontWeight.Normal
        }
        val align = when (textAlign) {
            "Center" -> TextAlign.Center
            "End" -> TextAlign.End
            else -> TextAlign.Start
        }

        return TextStyle(
            fontSize = fontSize.sp,
            fontWeight = weight,
            color = color,
            textAlign = align
        )
    }
}

/**
 * Datos de estilo de botón
 */
@Serializable
data class ButtonStyleData(
    val backgroundColor: String = "#6200EE",
    val contentColor: String = "#FFFFFF"
) {
    fun toButtonColors(): ButtonColors {
        val bgColor = Color(android.graphics.Color.parseColor(backgroundColor))
        val ctColor = Color(android.graphics.Color.parseColor(contentColor))

        return ButtonDefaults.buttonColors(
            containerColor = bgColor,
            contentColor = ctColor
        )
    }
}

/**
 * Datos de modifier
 */
@Serializable
data class ModifierData(
    val width: Float? = null,
    val height: Float? = null,
    val padding: Float = 0f,
    val backgroundColor: String? = null,
    val cornerRadius: Float = 0f
) {
    fun toComposeModifier(): Modifier {
        var modifier = Modifier

        width?.let { modifier = modifier.width(it.dp) }
        height?.let { modifier = modifier.height(it.dp) }

        if (padding > 0) {
            modifier = modifier.padding(padding.dp)
        }

        backgroundColor?.let {
            val color = Color(android.graphics.Color.parseColor(it))
            modifier = modifier.background(color)
        }

        if (cornerRadius > 0) {
            modifier = modifier.clip(RoundedCornerShape(cornerRadius.dp))
        }

        return modifier
    }
}