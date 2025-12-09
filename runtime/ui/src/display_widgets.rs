//! Display Widgets Implementation
//!
//! This module contains the implementation of display widgets:
//! - Text: Text display with basic formatting
//! - Image: Image display with sizing and fit options
//! - Icon: Unicode icon display with styling

use crate::vdom::VDomNode;
use crate::context::BuildContext;

/// Display mode for text widgets
#[derive(Debug, Clone, PartialEq)]
pub enum TextDisplay {
    Inline,  // <span>
    Block,   // <p>
}

/// Text widget for displaying text content
#[derive(Clone)]
pub struct Text {
    pub content: String,
    pub font_size: Option<f32>,
    pub color: Option<String>,
    pub font_weight: Option<String>,
    pub text_align: Option<String>,
    pub display: TextDisplay,
}

impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("content", &self.content)
            .field("font_size", &self.font_size)
            .field("color", &self.color)
            .field("font_weight", &self.font_weight)
            .field("text_align", &self.text_align)
            .field("display", &self.display)
            .finish()
    }
}

impl Text {
    /// Create a new text widget
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font_size: None,
            color: None,
            font_weight: None,
            text_align: None,
            display: TextDisplay::Inline,
        }
    }

    /// Set font size in pixels
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Set text color (hex color)
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set font weight
    pub fn font_weight(mut self, weight: impl Into<String>) -> Self {
        self.font_weight = Some(weight.into());
        self
    }

    /// Set text alignment
    pub fn text_align(mut self, align: impl Into<String>) -> Self {
        self.text_align = Some(align.into());
        self
    }

    /// Make text bold
    pub fn bold(mut self) -> Self {
        self.font_weight = Some("bold".to_string());
        self
    }

    /// Center align text
    pub fn align_center(mut self) -> Self {
        self.text_align = Some("center".to_string());
        self
    }

    /// Display as block element
    pub fn block(mut self) -> Self {
        self.display = TextDisplay::Block;
        self
    }

    /// Generate CSS styles for the text
    pub fn generate_css(&self) -> String {
        let mut styles = Vec::new();

        if let Some(size) = self.font_size {
            styles.push(format!("font-size: {}px", size));
        }

        if let Some(ref color) = self.color {
            styles.push(format!("color: {}", color));
        }

        if let Some(ref weight) = self.font_weight {
            styles.push(format!("font-weight: {}", weight));
        }

        if let Some(ref align) = self.text_align {
            styles.push(format!("text-align: {}", align));
        }

        if styles.is_empty() {
            String::new()
        } else {
            styles.join("; ")
        }
    }
}

impl crate::widget::Widget for Text {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("class".to_string(), "text".to_string());

        let css = self.generate_css();
        if !css.is_empty() {
            attributes.insert("style".to_string(), css);
        }

        let tag_name = match self.display {
            TextDisplay::Inline => "span",
            TextDisplay::Block => "p",
        };

        VDomNode {
            node_type: crate::vdom::NodeType::Element,
            tag_name: Some(tag_name.to_string()),
            attributes,
            children: Vec::new(),
            text_content: Some(self.content.clone()),
            event_listeners: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            key: None,
        }
    }
}

/// Image fit modes for object-fit CSS property
#[derive(Debug, Clone, PartialEq)]
pub enum ImageFit {
    Contain,   // object-fit: contain
    Cover,     // object-fit: cover
    Fill,      // object-fit: fill
    None,      // object-fit: none
    ScaleDown, // object-fit: scale-down
}

impl ImageFit {
    fn as_css(&self) -> &'static str {
        match self {
            ImageFit::Contain => "contain",
            ImageFit::Cover => "cover",
            ImageFit::Fill => "fill",
            ImageFit::None => "none",
            ImageFit::ScaleDown => "scale-down",
        }
    }
}

/// Image widget for displaying images
#[derive(Clone)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub fit: ImageFit,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("src", &self.src)
            .field("alt", &self.alt)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("fit", &self.fit)
            .finish()
    }
}

impl Image {
    /// Create a new image widget
    pub fn new(src: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            alt: None,
            width: None,
            height: None,
            fit: ImageFit::Contain,
        }
    }

    /// Set alt text for accessibility
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    /// Set image dimensions
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set image width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set image height
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set image fit mode
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    /// Generate CSS styles for the image
    pub fn generate_css(&self) -> String {
        let mut styles = Vec::new();

        if let Some(width) = self.width {
            styles.push(format!("width: {}px", width));
        }

        if let Some(height) = self.height {
            styles.push(format!("height: {}px", height));
        }

        styles.push(format!("object-fit: {}", self.fit.as_css()));

        styles.join("; ")
    }
}

impl crate::widget::Widget for Image {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("src".to_string(), self.src.clone());
        attributes.insert("class".to_string(), "image".to_string());

        if let Some(ref alt) = self.alt {
            attributes.insert("alt".to_string(), alt.clone());
        }

        let css = self.generate_css();
        if !css.is_empty() {
            attributes.insert("style".to_string(), css);
        }

        VDomNode {
            node_type: crate::vdom::NodeType::Element,
            tag_name: Some("img".to_string()),
            attributes,
            children: Vec::new(),
            text_content: None,
            event_listeners: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            key: None,
        }
    }
}

/// Icon widget for displaying Unicode icons
#[derive(Clone)]
pub struct Icon {
    pub code: char,
    pub size: Option<f32>,
    pub color: Option<String>,
    pub weight: Option<String>,
}

impl std::fmt::Debug for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Icon")
            .field("code", &self.code)
            .field("size", &self.size)
            .field("color", &self.color)
            .field("weight", &self.weight)
            .finish()
    }
}

impl Icon {
    /// Create a new icon widget
    pub fn new(code: char) -> Self {
        Self {
            code,
            size: None,
            color: None,
            weight: None,
        }
    }

    /// Set icon size in pixels
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    /// Set icon color (hex color)
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set font weight
    pub fn weight(mut self, weight: impl Into<String>) -> Self {
        self.weight = Some(weight.into());
        self
    }

    /// Make icon bold
    pub fn bold(mut self) -> Self {
        self.weight = Some("bold".to_string());
        self
    }

    /// Generate CSS styles for the icon
    pub fn generate_css(&self) -> String {
        let mut styles = Vec::new();

        if let Some(size) = self.size {
            styles.push(format!("font-size: {}px", size));
        }

        if let Some(ref color) = self.color {
            styles.push(format!("color: {}", color));
        }

        if let Some(ref weight) = self.weight {
            styles.push(format!("font-weight: {}", weight));
        }

        if styles.is_empty() {
            String::new()
        } else {
            styles.join("; ")
        }
    }
}

impl crate::widget::Widget for Icon {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("class".to_string(), "icon".to_string());

        let css = self.generate_css();
        if !css.is_empty() {
            attributes.insert("style".to_string(), css);
        }

        VDomNode {
            node_type: crate::vdom::NodeType::Element,
            tag_name: Some("span".to_string()),
            attributes,
            children: Vec::new(),
            text_content: Some(self.code.to_string()),
            event_listeners: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            key: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::BuildContext;
    use crate::widget::Widget;

    #[test]
    fn test_text_new() {
        let text = Text::new("Hello World");
        assert_eq!(text.content, "Hello World");
        assert_eq!(text.display, TextDisplay::Inline);
        assert!(text.font_size.is_none());
    }

    #[test]
    fn test_text_builder_methods() {
        let text = Text::new("Hello")
            .font_size(16.0)
            .color("#ff0000")
            .bold()
            .align_center()
            .block();

        assert_eq!(text.content, "Hello");
        assert_eq!(text.font_size, Some(16.0));
        assert_eq!(text.color, Some("#ff0000".to_string()));
        assert_eq!(text.font_weight, Some("bold".to_string()));
        assert_eq!(text.text_align, Some("center".to_string()));
        assert_eq!(text.display, TextDisplay::Block);
    }

    #[test]
    fn test_text_generate_css() {
        let text = Text::new("Hello")
            .font_size(16.0)
            .color("#ff0000")
            .bold();

        let css = text.generate_css();
        assert!(css.contains("font-size: 16px"));
        assert!(css.contains("color: #ff0000"));
        assert!(css.contains("font-weight: bold"));
    }

    #[test]
    fn test_text_build_inline() {
        let text = Text::new("Hello").font_size(16.0);
        let context = BuildContext::new();
        let node = text.build(&context);

        assert_eq!(node.tag_name, Some("span".to_string()));
        assert_eq!(node.text_content, Some("Hello".to_string()));
        assert!(node.attributes.contains_key("style"));
    }

    #[test]
    fn test_text_build_block() {
        let text = Text::new("Hello").block();
        let context = BuildContext::new();
        let node = text.build(&context);

        assert_eq!(node.tag_name, Some("p".to_string()));
    }

    #[test]
    fn test_image_new() {
        let image = Image::new("test.jpg");
        assert_eq!(image.src, "test.jpg");
        assert_eq!(image.fit, ImageFit::Contain);
        assert!(image.alt.is_none());
    }

    #[test]
    fn test_image_builder_methods() {
        let image = Image::new("test.jpg")
            .alt("Test image")
            .size(100.0, 200.0)
            .fit(ImageFit::Cover);

        assert_eq!(image.src, "test.jpg");
        assert_eq!(image.alt, Some("Test image".to_string()));
        assert_eq!(image.width, Some(100.0));
        assert_eq!(image.height, Some(200.0));
        assert_eq!(image.fit, ImageFit::Cover);
    }

    #[test]
    fn test_image_generate_css() {
        let image = Image::new("test.jpg")
            .size(100.0, 200.0)
            .fit(ImageFit::Cover);

        let css = image.generate_css();
        assert!(css.contains("width: 100px"));
        assert!(css.contains("height: 200px"));
        assert!(css.contains("object-fit: cover"));
    }

    #[test]
    fn test_image_build() {
        let image = Image::new("test.jpg").alt("Test");
        let context = BuildContext::new();
        let node = image.build(&context);

        assert_eq!(node.tag_name, Some("img".to_string()));
        assert_eq!(node.attributes.get("src"), Some(&"test.jpg".to_string()));
        assert_eq!(node.attributes.get("alt"), Some(&"Test".to_string()));
    }

    #[test]
    fn test_icon_new() {
        let icon = Icon::new('🔥');
        assert_eq!(icon.code, '🔥');
        assert!(icon.size.is_none());
    }

    #[test]
    fn test_icon_builder_methods() {
        let icon = Icon::new('🔥')
            .size(24.0)
            .color("#ff0000")
            .bold();

        assert_eq!(icon.code, '🔥');
        assert_eq!(icon.size, Some(24.0));
        assert_eq!(icon.color, Some("#ff0000".to_string()));
        assert_eq!(icon.weight, Some("bold".to_string()));
    }

    #[test]
    fn test_icon_generate_css() {
        let icon = Icon::new('🔥')
            .size(24.0)
            .color("#ff0000");

        let css = icon.generate_css();
        assert!(css.contains("font-size: 24px"));
        assert!(css.contains("color: #ff0000"));
    }

    #[test]
    fn test_icon_build() {
        let icon = Icon::new('🔥').size(24.0);
        let context = BuildContext::new();
        let node = icon.build(&context);

        assert_eq!(node.tag_name, Some("span".to_string()));
        assert_eq!(node.text_content, Some("🔥".to_string()));
        assert!(node.attributes.contains_key("style"));
    }

    #[test]
    fn test_image_fit_as_css() {
        assert_eq!(ImageFit::Contain.as_css(), "contain");
        assert_eq!(ImageFit::Cover.as_css(), "cover");
        assert_eq!(ImageFit::Fill.as_css(), "fill");
        assert_eq!(ImageFit::None.as_css(), "none");
        assert_eq!(ImageFit::ScaleDown.as_css(), "scale-down");
    }
}