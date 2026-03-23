#![allow(dead_code)]

use image::{DynamicImage, ImageBuffer, Luma};
use imageproc::contrast::adaptive_threshold;
use imageproc::filter::filter3x3;
use std::path::PathBuf;
use tesseract_rs::TesseractAPI;

pub fn get_default_tessdata_dir() -> PathBuf {
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

pub fn get_tessdata_dir() -> PathBuf {
    match std::env::var("TESSDATA_PREFIX") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => get_default_tessdata_dir(),
    }
}

pub fn preprocess_image(img: &DynamicImage) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let luma_img = img.to_luma8();
    let contrast_adjusted = adaptive_threshold(&luma_img, 2);
    filter3x3(&contrast_adjusted, &[-1, -1, -1, -1, 9, -1, -1, -1, -1])
}

pub fn load_test_image(
    filename: &str,
) -> std::result::Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
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

/// Create a minimal initialized TesseractAPI for testing
pub fn create_initialized_api() -> TesseractAPI {
    let tessdata_dir = get_tessdata_dir();
    let api = TesseractAPI::new();
    api.init(tessdata_dir.to_str().unwrap(), "eng")
        .expect("Failed to initialize Tesseract");
    api
}

/// Create an API with the sample_text.png image already loaded
pub fn create_api_with_image() -> TesseractAPI {
    let api = create_initialized_api();
    let (image_data, width, height) =
        load_test_image("sample_text.png").expect("Failed to load test image");
    api.set_image(
        &image_data,
        width as i32,
        height as i32,
        3,
        3 * width as i32,
    )
    .expect("Failed to set image");
    api
}
