use image::{DynamicImage, ImageBuffer, Luma};
use imageproc::contrast::adaptive_threshold;
use imageproc::filter::filter3x3;
use std::path::PathBuf;
use tesseract_rs::TesseractAPI;

fn get_default_tessdata_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
        PathBuf::from(home_dir)
            .join("Library")
            .join("Application Support")
            .join("tesseract-rs")
            .join("tessdata")
    } else if cfg!(target_os = "linux") {
        let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
        PathBuf::from(home_dir)
            .join(".tesseract-rs")
            .join("tessdata")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var("APPDATA").expect("APPDATA environment variable not set"))
            .join("tesseract-rs")
            .join("tessdata")
    } else {
        panic!("Unsupported operating system");
    }
}

fn get_tessdata_dir() -> PathBuf {
    match std::env::var("TESSDATA_PREFIX") {
        Ok(dir) => {
            let path = PathBuf::from(dir);
            println!("Using TESSDATA_PREFIX directory: {:?}", path);
            path
        }
        Err(_) => {
            let default_dir = get_default_tessdata_dir();
            println!(
                "TESSDATA_PREFIX not set, using default directory: {:?}",
                default_dir
            );
            default_dir
        }
    }
}

fn preprocess_image(img: &DynamicImage) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let luma_img = img.to_luma8();

    let contrast_adjusted = adaptive_threshold(&luma_img, 2);

    filter3x3(&contrast_adjusted, &[-1, -1, -1, -1, 9, -1, -1, -1, -1])
}

fn load_test_image(filename: &str) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("test_images");
    path.push(filename);

    let img = image::open(&path)
        .map_err(|e| format!("Failed to open test image {}: {}", path.display(), e))?
        .to_rgb8();
    let (width, height) = img.dimensions();
    Ok((img.into_raw(), width, height))
}

#[test]
fn test_multiple_languages_with_lstm() {
    let tessdata_dir = get_tessdata_dir();

    let eng_traineddata = tessdata_dir.join("eng.traineddata");
    let tur_traineddata = tessdata_dir.join("tur.traineddata");
    assert!(eng_traineddata.exists(), "eng.traineddata not found");
    assert!(tur_traineddata.exists(), "tur.traineddata not found");

    let api = TesseractAPI::new();
    let res = api.set_variable("debug_file", "/dev/null");
    assert!(res.is_ok());

    api.init(tessdata_dir.to_str().unwrap(), "tur")
        .expect("Failed to initialize Tesseract with multiple languages");

    api.set_variable("tessedit_ocr_engine_mode", "4")
        .expect("Failed to set LSTM mode");

    api.set_variable("tessedit_pageseg_mode", "1")
        .expect("Failed to set PSM");
    //api.set_variable("tessedit_char_blacklist", "!?@#$%&*()_+-=[]{}|\\")
    //    .expect("Failed to set char blacklist");

    let img = image::open("tests/test_images/multilang_sample.png").expect("Failed to open image");
    let preprocessed = preprocess_image(&img);
    let (width, height) = preprocessed.dimensions();

    let res = api.set_image(
        preprocessed.as_raw(),
        width as i32,
        height as i32,
        1,
        width as i32,
    );
    assert!(res.is_ok());
    let text = api.get_utf8_text().expect("Failed to perform OCR");
    println!("Recognized text: {}", text);

    assert!(!text.is_empty());
    assert!(
        text.to_lowercase().contains("hello") && text.to_lowercase().contains("dünya"),
        "Text does not contain expected words. Found: {}",
        text
    );

    let confidences = api.get_word_confidences();
    println!("Word confidences: {:?}", confidences);
    assert!(confidences.is_ok(), "No word confidences returned");
    assert!(
        confidences.unwrap().iter().any(|&c| c > 80),
        "No high confidence words found"
    );
}

#[test]
fn test_ocr_on_real_image() {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");

    let (image_data, width, height) =
        load_test_image("sample_text.png").expect("Failed to load test image");
    let res = api.set_image(
        &image_data,
        width as i32,
        height as i32,
        3,
        3 * width as i32,
    );
    assert!(res.is_ok());
    let text = api.get_utf8_text().expect("Failed to perform OCR");
    assert!(!text.is_empty());
    assert!(text.contains("This is a sample text for OCR testing."));

    let confidences = api.get_word_confidences();
    assert!(confidences.is_ok());
    assert!(confidences.unwrap().iter().all(|&c| c > 0));
}

#[test]
fn test_multiple_languages() {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();
    api.init(tessdata_dir.to_str().unwrap(), "tur+eng")
        .expect("Failed to initialize Tesseract with multiple languages");
    api.set_variable("tessedit_pageseg_mode", "1")
        .expect("Failed to set PSM");
    //api.set_variable("tessedit_char_blacklist", "!?@#$%&*()_+-=[]{}").expect("Failed to set char blacklist");
    api.set_variable("tessedit_enable_dict_correction", "1")
        .expect("Failed to enable dictionary correction");

    api.set_variable("preserve_interword_spaces", "1")
        .expect("Failed to set preserve_interword_spaces");

    api.set_variable(
        "tessedit_char_whitelist",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZçğıiöşüÇĞİIÖŞÜ.,! ",
    )
    .expect("Failed to set char whitelist");

    let (image_data, width, height) =
        load_test_image("multilang_sample.png").expect("Failed to load test image");
    let res = api.set_image(
        &image_data,
        width as i32,
        height as i32,
        3,
        3 * width as i32,
    );
    assert!(res.is_ok());
    let text = api.get_utf8_text().expect("Failed to perform OCR");
    assert!(!text.is_empty());
    assert!(text.contains("Hello") && text.contains("Dünya"));
}

#[test]
fn test_digit_recognition() {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");
    api.set_variable("tessedit_char_whitelist", "0123456789")
        .expect("Failed to set whitelist");

    let (image_data, width, height) =
        load_test_image("digits.png").expect("Failed to load test image");
    let res = api.set_image(
        &image_data,
        width as i32,
        height as i32,
        3,
        3 * width as i32,
    );
    assert!(res.is_ok());

    let text = api.get_utf8_text().expect("Failed to perform OCR");
    assert!(!text.is_empty());
    assert!(text
        .chars()
        .all(|c| c.is_ascii_digit() || c.is_whitespace()));
}

#[test]
fn test_error_handling() {
    let api = TesseractAPI::new();

    let init_result = api.init("/invalid/path", "eng");
    assert!(init_result.is_err());

    if init_result.is_err() {}
}

#[test]
fn test_image_operation_errors() {
    let api = TesseractAPI::new();
    let tessdata_dir = get_tessdata_dir();

    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");

    let (image_data, width, height) =
        load_test_image("sample_text.png").expect("Failed to load test image");

    let res = api.set_image(
        &image_data,
        0, // Invalid width
        height as i32,
        3,
        3 * width as i32,
    );
    assert!(res.is_err());
}

#[test]
fn test_invalid_language_code() {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();

    // Test invalid language code
    let result = api.init(tessdata_dir.to_str().unwrap(), "invalid_lang");
    assert!(result.is_err());
}

#[test]
fn test_empty_image_data() {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");

    // Test with empty image data
    let empty_data: Vec<u8> = Vec::new();
    let res = api.set_image(&empty_data, 100, 100, 3, 300);
    assert!(res.is_err());
}

#[test]
fn test_invalid_image_parameters() {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");

    let (image_data, width, height) =
        load_test_image("sample_text.png").expect("Failed to load test image");

    // Test negative dimensions
    let res = api.set_image(&image_data, -1, height as i32, 3, 3 * width as i32);
    assert!(res.is_err());

    // Test zero height
    let res = api.set_image(&image_data, width as i32, 0, 3, 3 * width as i32);
    assert!(res.is_err());

    // Test invalid bytes_per_pixel
    let res = api.set_image(
        &image_data,
        width as i32,
        height as i32,
        0,
        3 * width as i32,
    );
    assert!(res.is_err());

    // Test mismatched bytes_per_line
    let res = api.set_image(&image_data, width as i32, height as i32, 3, width as i32); // Should be 3 * width
    assert!(res.is_err());
}

#[test]
fn test_variable_setting() {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");

    // Test invalid variable name
    let res = api.set_variable("invalid_variable_name", "1");
    assert!(res.is_err());

    // Test empty variable value
    let res = api.set_variable("tessedit_char_whitelist", "");
    assert!(res.is_ok()); // Empty whitelist is actually valid

    // Test valid variable settings
    assert!(api.set_variable("tessedit_pageseg_mode", "1").is_ok());
    assert!(api.set_variable("tessedit_ocr_engine_mode", "1").is_ok());
}

#[test]
fn test_multiple_operations() {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");

    let (image_data, width, height) =
        load_test_image("sample_text.png").expect("Failed to load test image");

    // Set image multiple times
    for _ in 0..3 {
        let res = api.set_image(
            &image_data,
            width as i32,
            height as i32,
            3,
            3 * width as i32,
        );
        assert!(res.is_ok());
        let text = api.get_utf8_text().expect("Failed to perform OCR");
        assert!(!text.is_empty());
    }
}

#[test]
fn test_preprocessing_effects() {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");

    let img = image::open("tests/test_images/sample_text.png").expect("Failed to open image");

    // Test with preprocessed image
    let preprocessed = preprocess_image(&img);
    let (width, height) = preprocessed.dimensions();

    let res = api.set_image(
        preprocessed.as_raw(),
        width as i32,
        height as i32,
        1,
        width as i32,
    );
    assert!(res.is_ok());

    let text = api.get_utf8_text().expect("Failed to perform OCR");
    assert!(!text.is_empty());
}

#[test]
fn test_concurrent_access() {
    use std::thread;

    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");

    // Multiple threads trying to access the API simultaneously
    let mut handles = vec![];

    for i in 0..3 {
        let api_clone = api.clone();
        let handle = thread::spawn(move || {
            match i % 3 {
                0 => {
                    let res = api_clone.set_variable("tessedit_pageseg_mode", "1");
                    assert!(res.is_ok());
                }
                1 => {
                    let res = api_clone.set_variable("tessedit_char_whitelist", "0123456789");
                    assert!(res.is_ok());
                }
                2 => {
                    let text = api_clone.get_utf8_text();
                    // Text might be empty since we haven't set an image, but it shouldn't panic
                    assert!(text.is_err());
                }
                _ => unreachable!(),
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_thread_safety_with_image() {
    use std::sync::Arc;
    use std::thread;

    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();

    // Ana API'yi configure et
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");
    api.set_variable("tessedit_pageseg_mode", "1")
        .expect("Failed to set PSM");

    let (image_data, width, height) =
        load_test_image("sample_text.png").expect("Failed to load test image");

    // Image'ı ana thread'de set et
    let res = api.set_image(
        &image_data,
        width as i32,
        height as i32,
        3,
        3 * width as i32,
    );
    assert!(res.is_ok());

    let image_data = Arc::new(image_data);
    let mut handles = vec![];

    // Thread'lerde clone'lanmış API'yi kullan
    for _ in 0..3 {
        let api_clone = api.clone(); // Bu artık tüm konfigürasyonu da kopyalayacak
        let image_data = Arc::clone(&image_data);

        let handle = thread::spawn(move || {
            let res = api_clone.set_image(
                &image_data,
                width as i32,
                height as i32,
                3,
                3 * width as i32,
            );
            assert!(res.is_ok());

            let text = api_clone.get_utf8_text().expect("Failed to get text");
            assert!(!text.is_empty());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_thread_safety_init() {
    use std::thread;

    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();

    let mut handles = vec![];

    // Try to initialize from multiple threads
    for i in 0..3 {
        let api_clone = api.clone();
        let tessdata_dir = tessdata_dir.clone();

        let handle = thread::spawn(move || {
            let lang = match i % 3 {
                0 => "eng",
                1 => "tur",
                2 => "eng+tur",
                _ => unreachable!(),
            };

            let res = api_clone.init(tessdata_dir.to_str().unwrap(), lang);
            // Note: Only one initialization should succeed due to mutex
            if res.is_err() {
                println!(
                    "Init failed for lang {}, which is expected in some cases",
                    lang
                );
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_dynamic_image_setting() {
    let api = TesseractAPI::new();

    // Get tessdata directory (uses default location or TESSDATA_PREFIX if set)
    let tessdata_dir = get_tessdata_dir();
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize API");

    // Create a 24x24 pixel test image with the number 9 (black on white background)
    let width = 24;
    let height = 24;
    let bytes_per_pixel = 1;
    let bytes_per_line = width * bytes_per_pixel;

    // Initialize image data with all white pixels
    let mut image_data = vec![255u8; width * height];

    // Draw the number 9 (simplified version)
    for y in 4..19 {
        for x in 7..17 {
            // Top bar
            if y == 4 && (8..=15).contains(&x) {
                image_data[y * width + x] = 0;
            }
            // Top curve left side
            if (4..=10).contains(&y) && x == 7 {
                image_data[y * width + x] = 0;
            }
            // Top curve right side
            if (4..=11).contains(&y) && x == 16 {
                image_data[y * width + x] = 0;
            }
            // Middle bar
            if y == 11 && (8..=15).contains(&x) {
                image_data[y * width + x] = 0;
            }
            // Bottom right vertical line
            if (11..=18).contains(&y) && x == 16 {
                image_data[y * width + x] = 0;
            }
            // Bottom bar
            if y == 18 && (8..=15).contains(&x) {
                image_data[y * width + x] = 0;
            }
        }
    }

    // Set the image data
    api.set_image(
        &image_data,
        width.try_into().unwrap(),
        height.try_into().unwrap(),
        bytes_per_pixel.try_into().unwrap(),
        bytes_per_line.try_into().unwrap(),
    )
    .expect("Failed to set image");

    // Set whitelist for digits only
    api.set_variable("tessedit_char_whitelist", "0123456789")
        .expect("Failed to set whitelist");

    // Get the recognized text
    let text = api.get_utf8_text().expect("Failed to get text");
    println!("Recognized text: {}", text.trim());

    // Check if the result contains the digit 9
    assert!(!text.trim().is_empty(), "OCR result is empty");
    assert!(text.trim().contains("9"), "Expected digit '9' not found");
}
