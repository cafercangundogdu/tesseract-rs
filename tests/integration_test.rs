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
    assert!(text.chars().all(|c| c.is_digit(10) || c.is_whitespace()));
}

#[test]
fn test_error_handling() {
    let api = TesseractAPI::new();

    let init_result = api.init("/invalid/path", "eng");
    assert!(init_result.is_err());

    if init_result.is_err() {
        return;
    }
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
