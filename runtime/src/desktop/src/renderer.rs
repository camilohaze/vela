/*!
Skia-based renderer for desktop applications

This module provides hardware-accelerated 2D rendering using Skia,
integrating with the Vela widget system for desktop applications.
*/

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use skia_safe::{
    surfaces, images,
    Canvas, Color as SkiaColor, Font, FontMgr, Paint as SkiaPaint, Point, Rect as SkiaRect,
    Surface, TextBlob, Typeface,
};

/// Main renderer interface
pub struct DesktopRenderer {
    surface: Option<Surface>,
    font_mgr: FontMgr,
}

impl DesktopRenderer {
    /// Create a new desktop renderer
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let mut surface = surfaces::raster_n32_premul((width as i32, height as i32))
            .ok_or_else(|| anyhow::anyhow!("Failed to create Skia surface"))?;

        let font_mgr = FontMgr::default();

        Ok(Self {
            surface: Some(surface),
            font_mgr,
        })
    }

    /// Resize the render surface
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.surface = Some(
            surfaces::raster_n32_premul((width as i32, height as i32))
                .ok_or_else(|| anyhow::anyhow!("Failed to create Skia surface"))?
        );
        Ok(())
    }

    /// Begin a new frame
    pub fn begin_frame(&mut self) -> Result<()> {
        if let Some(surface) = &mut self.surface {
            let canvas = surface.canvas();
            canvas.clear(SkiaColor::WHITE);
        }
        Ok(())
    }

    /// Render a widget tree
    pub fn render_widget(&mut self, widget: &WidgetNode) -> Result<()> {
        if let Some(surface) = &mut self.surface {
            let canvas = surface.canvas();
            Self::render_widget_recursive_static(canvas, widget, &self.font_mgr, 0.0, 0.0)?;
        }
        Ok(())
    }

    /// Static version of render_widget_recursive to avoid borrowing issues
    fn render_widget_recursive_static(
        canvas: &Canvas,
        widget: &WidgetNode,
        font_mgr: &FontMgr,
        x: f32,
        y: f32,
    ) -> Result<()> {
        let x = x + widget.layout.x;
        let y = y + widget.layout.y;

        match &widget.widget_type {
            WidgetType::Container => {
                // Render container background if specified
                if let Some(bg_color) = widget.style.background_color {
                    let rect = SkiaRect::from_xywh(x, y, widget.layout.width, widget.layout.height);
                    let mut paint = SkiaPaint::default();
                    paint.set_color(bg_color.to_skia());
                    paint.set_anti_alias(true);
                    canvas.draw_rect(rect, &paint);
                }

                // Render children
                for child in &widget.children {
                    Self::render_widget_recursive_static(canvas, child, font_mgr, x, y)?;
                }
            }

            WidgetType::Text(text_props) => {
                let typeface = font_mgr
                    .match_family_style(&text_props.font_family, Default::default())
                    .ok_or_else(|| anyhow::anyhow!("Failed to load font family"))?;

                let font = Font::from_typeface(typeface, text_props.size);

                let mut paint = SkiaPaint::default();
                paint.set_color(text_props.color.to_skia());
                paint.set_anti_alias(true);

                let text_blob = TextBlob::from_str(&text_props.content, &font)
                    .ok_or_else(|| anyhow::anyhow!("Failed to create text blob"))?;

                canvas.draw_text_blob(&text_blob, (x, y), &paint);
            }

            WidgetType::Button(button_props) => {
                // Draw button background
                let rect = SkiaRect::from_xywh(x, y, widget.layout.width, widget.layout.height);
                let bg_color = if button_props.pressed {
                    button_props.pressed_color
                } else if button_props.hovered {
                    button_props.hover_color
                } else {
                    button_props.normal_color
                };

                let mut paint = SkiaPaint::default();
                paint.set_color(bg_color.to_skia());
                paint.set_anti_alias(true);
                canvas.draw_rect(rect, &paint);

                // Draw button border
                let mut border_paint = SkiaPaint::default();
                border_paint.set_color(button_props.border_color.to_skia());
                border_paint.set_stroke_width(1.0);
                border_paint.set_style(skia_safe::PaintStyle::Stroke);
                border_paint.set_anti_alias(true);
                canvas.draw_rect(rect, &border_paint);

                // Draw button text
                let typeface = font_mgr
                    .match_family_style(&button_props.font_family, Default::default())
                    .ok_or_else(|| anyhow::anyhow!("Failed to load font family"))?;

                let font = Font::from_typeface(typeface, button_props.text_size);

                let mut text_paint = SkiaPaint::default();
                text_paint.set_color(button_props.text_color.to_skia());
                text_paint.set_anti_alias(true);

                let text_blob = TextBlob::from_str(&button_props.text, &font)
                    .ok_or_else(|| anyhow::anyhow!("Failed to create text blob"))?;

                let text_bounds = text_blob.bounds();
                let text_x = x + (widget.layout.width - text_bounds.width()) / 2.0;
                let text_y = y + (widget.layout.height + button_props.text_size) / 2.0;

                canvas.draw_text_blob(&text_blob, (text_x, text_y), &text_paint);
            }

            WidgetType::Image(image_props) => {
                if let Some(image_data) = &image_props.image_data {
                    // Create Skia image from pixel data
                    let image_info = skia_safe::ImageInfo::new(
                        (image_data.width as i32, image_data.height as i32),
                        skia_safe::ColorType::RGBA8888,
                        skia_safe::AlphaType::Premul,
                        None,
                    );

                    if let Some(mut skia_image) = images::raster_from_data(
                        &image_info,
                        skia_safe::Data::new_copy(&image_data.pixels),
                        (image_data.width * 4) as usize,
                    ) {
                        let rect = SkiaRect::from_xywh(x, y, widget.layout.width, widget.layout.height);
                        canvas.draw_image_rect(&skia_image, None, rect, &SkiaPaint::default());
                    }
                }
            }

            WidgetType::Custom(_) => {
                // Custom widgets handle their own rendering
                // This would be implemented by the widget itself
            }
        }

        Ok(())
    }
    /// End frame and flush
    pub fn end_frame(&mut self) -> Result<()> {
        // Surface is automatically flushed when dropped or when we access pixels
        Ok(())
    }

    /// Get the rendered frame buffer
    pub fn get_framebuffer(&mut self) -> Option<Vec<u8>> {
        if let Some(surface) = &mut self.surface {
            let image = surface.image_snapshot();
            image.peek_pixels().map(|pixmap| pixmap.pixels().unwrap_or_default().to_vec())
        } else {
            None
        }
    }
}

/// Deserialize VelaVDOM JSON into widget tree
pub fn deserialize_vdom(json: &str) -> Result<WidgetNode> {
    serde_json::from_str(json).map_err(|e| anyhow::anyhow!("Failed to deserialize VDOM: {}", e))
}

/// Serialize widget tree to VelaVDOM JSON
pub fn serialize_vdom(widget: &WidgetNode) -> Result<String> {
    serde_json::to_string_pretty(widget).map_err(|e| anyhow::anyhow!("Failed to serialize VDOM: {}", e))
}

/// Widget node for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetNode {
    pub widget_type: WidgetType,
    pub layout: Layout,
    pub style: Style,
    pub children: Vec<WidgetNode>,
}

/// Widget types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    Container,
    Text(TextProperties),
    Button(ButtonProperties),
    Image(ImageProperties),
    Custom(String), // Custom widget type identifier
}

/// Layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Style information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<f32>,
    pub border_radius: Option<f32>,
}

/// Text widget properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProperties {
    pub content: String,
    pub color: Color,
    pub size: f32,
    pub font_family: String,
}

/// Button widget properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonProperties {
    pub text: String,
    pub text_color: Color,
    pub text_size: f32,
    pub font_family: String,
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
    pub border_color: Color,
    pub hovered: bool,
    pub pressed: bool,
}

/// Image widget properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageProperties {
    pub image_data: Option<ImageData>,
    pub fit: ImageFit,
}

/// Image data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA
}

/// Image fit modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFit {
    Fill,
    Contain,
    Cover,
    ScaleDown,
    None,
}

/// Color representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn white() -> Self {
        Self::rgb(255, 255, 255)
    }

    pub const fn black() -> Self {
        Self::rgb(0, 0, 0)
    }

    pub const fn red() -> Self {
        Self::rgb(255, 0, 0)
    }

    pub const fn green() -> Self {
        Self::rgb(0, 255, 0)
    }

    pub const fn blue() -> Self {
        Self::rgb(0, 0, 255)
    }

    /// Convert to Skia color
    pub fn to_skia(&self) -> SkiaColor {
        SkiaColor::from_argb(self.a, self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_color_creation() {
        let color = Color::rgb(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::red(), Color::rgb(255, 0, 0));
        assert_eq!(Color::green(), Color::rgb(0, 255, 0));
        assert_eq!(Color::blue(), Color::rgb(0, 0, 255));
    }

    #[test]
    fn test_color_to_skia() {
        let color = Color::rgb(255, 128, 64);
        let skia_color = color.to_skia();
        // Skia uses ARGB format
        assert_eq!(skia_color.a(), 255);
        assert_eq!(skia_color.r(), 255);
        assert_eq!(skia_color.g(), 128);
        assert_eq!(skia_color.b(), 64);
    }

    #[test]
    fn test_vdom_serialization() {
        let widget = WidgetNode {
            widget_type: WidgetType::Container(ContainerProps {
                children: vec![WidgetNode {
                    widget_type: WidgetType::Text(TextProps {
                        text: "Hello Vela!".to_string(),
                    }),
                    layout: Layout {
                        x: 10.0,
                        y: 10.0,
                        width: 200.0,
                        height: 50.0,
                    },
                    style: Style {
                        background_color: Some(Color::white()),
                        color: Some(Color::black()),
                        font_size: Some(16.0),
                        font_family: Some("Arial".to_string()),
                    },
                    properties: serde_json::Value::Null,
                }],
            }),
            layout: Layout {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
            },
            style: Style {
                background_color: Some(Color::white()),
                color: None,
                font_size: None,
                font_family: None,
            },
            properties: serde_json::Value::Null,
        };

        let json = serialize_vdom(&widget).unwrap();
        let deserialized = deserialize_vdom(&json).unwrap();

        // Verificar estructura básica
        match deserialized.widget_type {
            WidgetType::Container(container) => {
                assert_eq!(container.children.len(), 1);
                match &container.children[0].widget_type {
                    WidgetType::Text(text) => {
                        assert_eq!(text.text, "Hello Vela!");
                    }
                    _ => panic!("Expected Text widget"),
                }
            }
            _ => panic!("Expected Container widget"),
        }
    }

    #[test]
    fn test_widget_types() {
        // Test Container
        let container = WidgetNode {
            widget_type: WidgetType::Container(ContainerProps { children: vec![] }),
            layout: Layout::default(),
            style: Style::default(),
            properties: serde_json::Value::Null,
        };
        assert!(matches!(container.widget_type, WidgetType::Container(_)));

        // Test Text
        let text = WidgetNode {
            widget_type: WidgetType::Text(TextProps { text: "Test".to_string() }),
            layout: Layout::default(),
            style: Style::default(),
            properties: serde_json::Value::Null,
        };
        assert!(matches!(text.widget_type, WidgetType::Text(_)));

        // Test Button
        let button = WidgetNode {
            widget_type: WidgetType::Button(ButtonProps {
                text: "Click".to_string(),
                enabled: true,
            }),
            layout: Layout::default(),
            style: Style::default(),
            properties: serde_json::Value::Null,
        };
        assert!(matches!(button.widget_type, WidgetType::Button(_)));

        // Test Image
        let image = WidgetNode {
            widget_type: WidgetType::Image(ImageProps {
                image_data: ImageData {
                    width: 100,
                    height: 100,
                    pixels: vec![255; 100 * 100 * 4],
                },
            }),
            layout: Layout::default(),
            style: Style::default(),
            properties: serde_json::Value::Null,
        };
        assert!(matches!(image.widget_type, WidgetType::Image(_)));
    }

    #[test]
    fn test_layout_properties() {
        let layout = Layout {
            x: 10.5,
            y: 20.5,
            width: 300.0,
            height: 200.0,
        };

        assert_eq!(layout.x, 10.5);
        assert_eq!(layout.y, 20.5);
        assert_eq!(layout.width, 300.0);
        assert_eq!(layout.height, 200.0);

        // Dimensiones positivas
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    #[test]
    fn test_nested_widgets() {
        let nested = WidgetNode {
            widget_type: WidgetType::Container(ContainerProps {
                children: vec![
                    WidgetNode {
                        widget_type: WidgetType::Container(ContainerProps {
                            children: vec![
                                WidgetNode {
                                    widget_type: WidgetType::Text(TextProps {
                                        text: "Deep".to_string(),
                                    }),
                                    layout: Layout::default(),
                                    style: Style::default(),
                                    properties: serde_json::Value::Null,
                                }
                            ],
                        }),
                        layout: Layout::default(),
                        style: Style::default(),
                        properties: serde_json::Value::Null,
                    }
                ],
            }),
            layout: Layout::default(),
            style: Style::default(),
            properties: serde_json::Value::Null,
        };

        // Contar profundidad
        fn count_depth(widget: &WidgetNode) -> usize {
            match &widget.widget_type {
                WidgetType::Container(container) => {
                    1 + container.children.iter().map(count_depth).max().unwrap_or(0)
                }
                _ => 1,
            }
        }

        assert_eq!(count_depth(&nested), 3);
    }

    #[test]
    fn test_button_properties() {
        let button_props = ButtonProps {
            text: "Click me".to_string(),
            enabled: true,
        };

        assert_eq!(button_props.text, "Click me");
        assert_eq!(button_props.enabled, true);
    }

    #[test]
    fn test_image_properties() {
        let image_data = ImageData {
            width: 200,
            height: 150,
            pixels: vec![255; 200 * 150 * 4],
        };

        assert_eq!(image_data.width, 200);
        assert_eq!(image_data.height, 150);
        assert_eq!(image_data.pixels.len(), 200 * 150 * 4);
    }

    #[test]
    fn test_color_validation() {
        // Colores válidos
        let valid_color = Color::rgb(255, 128, 64);
        assert!(valid_color.r <= 255 && valid_color.g <= 255 && valid_color.b <= 255 && valid_color.a <= 255);

        // Nota: En Rust no podemos crear colores inválidos fácilmente debido a los tipos u8
    }

    #[test]
    fn test_font_properties() {
        let style = Style {
            background_color: None,
            color: Some(Color::black()),
            font_size: Some(16.0),
            font_family: Some("Arial".to_string()),
        };

        assert_eq!(style.font_size, Some(16.0));
        assert_eq!(style.font_family, Some("Arial".to_string()));
        assert!(style.font_size.unwrap() > 0.0);
    }
}

