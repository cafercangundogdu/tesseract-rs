//! Example demonstrating embedded tessdata functionality
//! 
//! This example shows how to use the embed-tessdata feature to create
//! a single binary with tessdata embedded directly into it.
//! 
//! To run this example:
//! ```bash
//! cargo run --example embedded_example --features embed-tessdata
//! ```

use tesseract_rs::TesseractAPI;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Tesseract-rs Embedded Tessdata Example");
    println!("======================================");
    
    let api = TesseractAPI::new();
    
    // Show available embedded languages
    let languages = api.embedded_languages();
    println!("Available embedded languages: {:?}", languages);
    
    // Initialize with embedded English tessdata
    println!("\nInitializing with embedded English tessdata...");
    api.init_embedded("eng")?;
    println!("✓ Successfully initialized with embedded tessdata!");
    
    // Create a simple test image with the word "HELLO"
    let width = 200;
    let height = 50;
    let mut image_data = vec![255u8; width * height]; // White background
    
    // Draw "HELLO" in a simple bitmap font
    draw_hello(&mut image_data, width, height);
    
    // Set the image for OCR
    api.set_image(
        &image_data,
        width as i32,
        height as i32,
        1, // bytes per pixel (grayscale)
        width as i32, // bytes per line
    )?;
    
    // Configure for better text recognition
    api.set_variable("tessedit_pageseg_mode", "8")?; // Single word
    
    // Perform OCR
    println!("\nPerforming OCR on test image...");
    let text = api.get_utf8_text()?;
    println!("Recognized text: '{}'", text.trim());
    
    // Test with Turkish if available
    if languages.contains(&"tur") {
        println!("\nTesting Turkish tessdata...");
        let api_tur = TesseractAPI::new();
        api_tur.init_embedded("tur")?;
        println!("✓ Successfully initialized with embedded Turkish tessdata!");
    }
    
    println!("\n🎉 Embedded tessdata example completed successfully!");
    println!("This binary contains all necessary tessdata files embedded within it.");
    
    Ok(())
}

/// Draw a simple "HELLO" text on the image
fn draw_hello(image_data: &mut [u8], width: usize, height: usize) {
    // Simple bitmap patterns for each letter
    let patterns = [
        // H
        vec![
            "##    ##",
            "##    ##", 
            "##    ##",
            "########",
            "##    ##",
            "##    ##",
            "##    ##",
        ],
        // E
        vec![
            "########",
            "##      ",
            "##      ",
            "######  ",
            "##      ",
            "##      ",
            "########",
        ],
        // L
        vec![
            "##      ",
            "##      ",
            "##      ",
            "##      ",
            "##      ",
            "##      ",
            "########",
        ],
        // L
        vec![
            "##      ",
            "##      ",
            "##      ",
            "##      ",
            "##      ",
            "##      ",
            "########",
        ],
        // O
        vec![
            " ###### ",
            "##    ##",
            "##    ##",
            "##    ##",
            "##    ##",
            "##    ##",
            " ###### ",
        ],
    ];
    
    let start_x = 20;
    let start_y = 15;
    let letter_width = 10;
    let letter_spacing = 12;
    
    for (letter_idx, pattern) in patterns.iter().enumerate() {
        let letter_x = start_x + letter_idx * letter_spacing;
        
        for (row_idx, row) in pattern.iter().enumerate() {
            let y = start_y + row_idx;
            if y >= height { break; }
            
            for (col_idx, ch) in row.chars().enumerate() {
                let x = letter_x + col_idx;
                if x >= width { break; }
                
                if ch == '#' {
                    image_data[y * width + x] = 0; // Black pixel
                }
            }
        }
    }
}
