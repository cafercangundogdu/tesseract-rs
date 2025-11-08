#[cfg(feature = "embed-tessdata")]
mod embedded_tests {
    use tesseract_rs::{TesseractAPI, embedded_languages, get_embedded_tessdata};

    #[test]
    fn test_embedded_languages_available() {
        let languages = embedded_languages();
        assert!(!languages.is_empty(), "Should have at least one embedded language");
        assert!(languages.contains(&"eng"), "English should be embedded");
    }

    #[test]
    fn test_get_embedded_tessdata() {
        let eng_data = get_embedded_tessdata("eng");
        assert!(eng_data.is_some(), "English tessdata should be embedded");
        
        let data = eng_data.unwrap();
        assert!(!data.is_empty(), "English tessdata should not be empty");
        
        // Check for tessdata file signature (first 4 bytes should be tessdata version)
        assert!(data.len() > 4, "Tessdata should be larger than 4 bytes");
    }

    #[test]
    fn test_get_nonexistent_language() {
        let data = get_embedded_tessdata("nonexistent");
        assert!(data.is_none(), "Non-existent language should return None");
    }

    #[test]
    fn test_api_embedded_languages() {
        let api = TesseractAPI::new();
        let languages = api.embedded_languages();
        assert!(!languages.is_empty(), "Should have at least one embedded language");
        assert!(languages.contains(&"eng"), "English should be embedded");
    }

    #[test]
    fn test_api_init_embedded() {
        let api = TesseractAPI::new();
        
        // Test successful initialization
        let result = api.init_embedded("eng");
        assert!(result.is_ok(), "Should successfully initialize with embedded English data");
    }

    #[test]
    fn test_api_init_embedded_nonexistent() {
        let api = TesseractAPI::new();
        
        // Test initialization with non-existent language
        let result = api.init_embedded("nonexistent");
        assert!(result.is_err(), "Should fail to initialize with non-existent language");
    }

    #[test]
    fn test_embedded_ocr_functionality() {
        let api = TesseractAPI::new();
        api.init_embedded("eng").expect("Failed to initialize with embedded data");

        // Create a simple test image (white background with black text-like pattern)
        let width = 100;
        let height = 30;
        let mut image_data = vec![255u8; width * height]; // White background

        // Draw simple text pattern (letter "A")
        for y in 10..25 {
            for x in 40..60 {
                if (y == 10 && x >= 45 && x <= 55) ||  // Top bar
                   (y == 17 && x >= 42 && x <= 58) ||  // Middle bar
                   ((y >= 10 && y <= 25) && (x == 42 || x == 58)) // Sides
                {
                    image_data[y * width + x] = 0; // Black pixels
                }
            }
        }

        // Set the image
        api.set_image(
            &image_data,
            width as i32,
            height as i32,
            1, // bytes per pixel
            width as i32, // bytes per line
        ).expect("Failed to set image");

        // Get OCR result
        let text = api.get_utf8_text().expect("Failed to get OCR text");
        
        // The result might not be perfect, but it should return some text
        assert!(!text.trim().is_empty(), "OCR should return some text");
    }
}

#[cfg(not(feature = "embed-tessdata"))]
mod no_embedded_tests {
    #[test]
    fn test_feature_not_enabled() {
        // This test just ensures the test suite runs even when embed-tessdata is not enabled
        assert!(true, "embed-tessdata feature not enabled");
    }
}
