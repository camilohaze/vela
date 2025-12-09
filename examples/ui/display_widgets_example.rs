//! Display Widgets Example
//!
//! This example demonstrates the usage of display widgets:
//! - Text: Basic text display with formatting
//! - Image: Image display with sizing and fit options
//! - Icon: Unicode icon display

use vela_ui::{Text, Image, ImageFit, Icon};

fn main() {
    println!("=== Vela UI Display Widgets Example ===\n");

    // Text Widget Examples
    println!("📝 TEXT WIDGETS:");

    // Basic inline text
    let basic_text = Text::new("Hello, Vela!");
    println!("Basic text: {:?}", basic_text);

    // Formatted text
    let formatted_text = Text::new("Welcome to Vela UI")
        .font_size(18.0)
        .color("#007bff")
        .bold()
        .align_center();
    println!("Formatted text: {:?}", formatted_text);

    // Block text
    let block_text = Text::new("This is a paragraph of text that demonstrates block display mode.")
        .block()
        .font_size(14.0)
        .text_align("justify");
    println!("Block text: {:?}", block_text);

    println!("\n🖼️  IMAGE WIDGETS:");

    // Basic image
    let basic_image = Image::new("logo.png");
    println!("Basic image: {:?}", basic_image);

    // Image with properties
    let styled_image = Image::new("profile.jpg")
        .alt("User profile picture")
        .size(100.0, 100.0)
        .fit(ImageFit::Cover);
    println!("Styled image: {:?}", styled_image);

    // Different fit modes
    let contain_image = Image::new("photo.jpg").fit(ImageFit::Contain);
    let fill_image = Image::new("background.jpg").fit(ImageFit::Fill);
    let scale_down_image = Image::new("thumbnail.jpg").fit(ImageFit::ScaleDown);
    println!("Contain fit: {:?}", contain_image);
    println!("Fill fit: {:?}", fill_image);
    println!("Scale down fit: {:?}", scale_down_image);

    println!("\n🔥 ICON WIDGETS:");

    // Basic icon
    let fire_icon = Icon::new('🔥');
    println!("Fire icon: {:?}", fire_icon);

    // Styled icons
    let large_heart = Icon::new('❤️').size(32.0).color("#ff0000");
    let bold_star = Icon::new('⭐').bold().size(24.0);
    let blue_check = Icon::new('✅').color("#007bff");
    println!("Large heart: {:?}", large_heart);
    println!("Bold star: {:?}", bold_star);
    println!("Blue check: {:?}", blue_check);

    println!("\n🎨 CSS GENERATION:");

    // Show CSS generation
    println!("Text CSS: {}", formatted_text.generate_css());
    println!("Image CSS: {}", styled_image.generate_css());
    println!("Icon CSS: {}", large_heart.generate_css());

    println!("\n✅ All display widgets created successfully!");
    println!("Run this example with: cargo run --example display_widgets_example");
}