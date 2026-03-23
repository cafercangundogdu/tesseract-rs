mod common;
use common::*;
use tesseract_rs::{TessPageIteratorLevel, TessPageSegMode, TessResultRenderer, TesseractAPI};

/// E2E: Basic OCR workflow -- init, load image, recognize, get text
#[test]
fn test_e2e_basic_ocr_workflow() {
    let api = TesseractAPI::new();
    let tessdata_dir = get_tessdata_dir();

    // Init
    api.init(tessdata_dir.to_str().unwrap(), "eng").unwrap();

    // Load image
    let (image_data, width, height) = load_test_image("sample_text.png").unwrap();
    api.set_image(
        &image_data,
        width as i32,
        height as i32,
        3,
        3 * width as i32,
    )
    .unwrap();

    // Get text
    let text = api.get_utf8_text().unwrap();
    assert!(text.contains("This is a sample text for OCR testing."));

    // Verify confidence
    let conf = api.mean_text_conf().unwrap();
    assert!(conf > 50, "Expected high confidence, got {}", conf);
}

/// E2E: Multi-format output -- same image, multiple output formats
#[test]
fn test_e2e_multi_format_output() {
    let api = create_api_with_image();

    let plain = api.get_utf8_text().unwrap();
    let hocr = api.get_hocr_text(0).unwrap();
    let tsv = api.get_tsv_text(0).unwrap();
    let box_text = api.get_box_text(0).unwrap();

    // All should contain some form of the recognized text
    assert!(!plain.is_empty());
    assert!(hocr.contains("ocr_page"));
    assert!(tsv.contains('\t'));
    assert!(!box_text.is_empty());
}

/// E2E: Word-by-word iteration with bounding boxes
#[test]
fn test_e2e_word_iteration_with_bounds() {
    let api = create_api_with_image();
    api.recognize().unwrap();
    let iter = api.get_iterator().unwrap();

    let mut words = Vec::new();
    loop {
        if let Ok((text, left, top, right, bottom, confidence)) = iter.get_word_with_bounds() {
            words.push((text, left, top, right, bottom, confidence));
        }
        if !iter.next(TessPageIteratorLevel::RIL_WORD).unwrap_or(false) {
            break;
        }
    }

    assert!(!words.is_empty(), "Should have found words");

    // Verify bounding boxes are reasonable
    for (text, left, top, right, bottom, conf) in &words {
        assert!(right > left, "Word '{}' has invalid width", text);
        assert!(bottom > top, "Word '{}' has invalid height", text);
        assert!(*conf >= 0.0, "Word '{}' has negative confidence", text);
    }
}

/// E2E: Digit-only recognition with whitelist
#[test]
fn test_e2e_digit_whitelist() {
    let api = TesseractAPI::new();
    let tessdata_dir = get_tessdata_dir();
    api.init(tessdata_dir.to_str().unwrap(), "eng").unwrap();

    // Configure for digits only
    api.set_variable("tessedit_char_whitelist", "0123456789")
        .unwrap();
    api.set_page_seg_mode(TessPageSegMode::PSM_SINGLE_BLOCK)
        .unwrap();

    let (image_data, width, height) = load_test_image("digits.png").unwrap();
    api.set_image(
        &image_data,
        width as i32,
        height as i32,
        3,
        3 * width as i32,
    )
    .unwrap();

    let text = api.get_utf8_text().unwrap();
    assert!(
        text.chars()
            .all(|c| c.is_ascii_digit() || c.is_whitespace()),
        "Expected only digits, got: {}",
        text
    );
}

/// E2E: Multiple images in sequence (reuse API)
#[test]
fn test_e2e_sequential_images() {
    let api = create_initialized_api();

    let images = ["sample_text.png", "digits.png", "multilang_sample.png"];

    for filename in &images {
        let (image_data, width, height) = load_test_image(filename).unwrap();
        api.set_image(
            &image_data,
            width as i32,
            height as i32,
            3,
            3 * width as i32,
        )
        .unwrap();
        let text = api.get_utf8_text().unwrap();
        assert!(!text.is_empty(), "OCR on {} returned empty", filename);
    }
}

/// E2E: Thread safety -- parallel OCR on different images
#[test]
fn test_e2e_parallel_ocr() {
    use std::thread;

    let images = vec![("sample_text.png", "sample"), ("digits.png", "digits")];

    let mut handles = vec![];

    for (filename, label) in images {
        let handle = thread::spawn(move || {
            let api = create_initialized_api();
            let (image_data, width, height) = load_test_image(filename).unwrap();
            api.set_image(
                &image_data,
                width as i32,
                height as i32,
                3,
                3 * width as i32,
            )
            .unwrap();
            let text = api.get_utf8_text().unwrap();
            assert!(!text.is_empty(), "OCR on {} returned empty", label);
            text
        });
        handles.push(handle);
    }

    for handle in handles {
        let text = handle.join().unwrap();
        assert!(!text.is_empty());
    }
}

/// E2E: try_clone preserves configuration (variables)
#[test]
fn test_e2e_clone_preserves_config() {
    let api = create_initialized_api();
    api.set_variable("tessedit_char_whitelist", "ABC").unwrap();

    let cloned = api.try_clone().unwrap();

    // Verify variable config preserved
    let whitelist = cloned
        .get_string_variable("tessedit_char_whitelist")
        .unwrap();
    assert_eq!(whitelist, "ABC");

    // Verify the cloned API is independently functional
    let init_lang = cloned.get_init_languages_as_string().unwrap();
    assert!(init_lang.contains("eng"));
}

/// E2E: Full pipeline -- init, configure, OCR, iterate, render
#[test]
fn test_e2e_full_pipeline() {
    let api = TesseractAPI::new();
    let tessdata_dir = get_tessdata_dir();
    api.init(tessdata_dir.to_str().unwrap(), "eng").unwrap();

    // Configure
    api.set_variable("tessedit_pageseg_mode", "1").unwrap();

    // Load image
    let (image_data, width, height) = load_test_image("sample_text.png").unwrap();
    api.set_image(
        &image_data,
        width as i32,
        height as i32,
        3,
        3 * width as i32,
    )
    .unwrap();

    // Get text
    let text = api.get_utf8_text().unwrap();
    assert!(!text.is_empty());

    // Get word confidences
    let confidences = api.all_word_confidences().unwrap();
    assert!(!confidences.is_empty());
    assert!(confidences.iter().all(|&c| c >= 0 && c <= 100));

    // Iterate words
    api.recognize().unwrap();
    let iter = api.get_iterator().unwrap();
    let mut word_count = 0;
    loop {
        if iter.get_utf8_text(TessPageIteratorLevel::RIL_WORD).is_ok() {
            word_count += 1;
        }
        if !iter.next(TessPageIteratorLevel::RIL_WORD).unwrap_or(false) {
            break;
        }
    }
    assert!(word_count > 0);
    drop(iter);

    // Render to text file
    let tmp = std::env::temp_dir().join("tesseract_e2e_pipeline");
    let renderer = TessResultRenderer::new_text_renderer(tmp.to_str().unwrap()).unwrap();
    renderer.begin_document("E2E Test").unwrap();
    renderer.add_image(&api).unwrap();
    renderer.end_document().unwrap();

    // Cleanup
    let _ = std::fs::remove_file(format!("{}.txt", tmp.display()));

    // Clear and verify
    api.clear().unwrap();
}

/// E2E: Language query
#[test]
fn test_e2e_language_info() {
    let api = create_initialized_api();

    let init_lang = api.get_init_languages_as_string().unwrap();
    assert!(init_lang.contains("eng"));

    let loaded = api.get_loaded_languages().unwrap();
    assert!(loaded.contains(&"eng".to_string()));

    let available = api.get_available_languages().unwrap();
    assert!(!available.is_empty());
    assert!(available.contains(&"eng".to_string()));
}

/// E2E: Error recovery -- API should be usable after errors
#[test]
fn test_e2e_error_recovery() {
    let api = create_initialized_api();

    // Try OCR without image -- should fail
    let result = api.get_utf8_text();
    assert!(result.is_err());

    // Now set a valid image -- should work
    let (image_data, width, height) = load_test_image("sample_text.png").unwrap();
    api.set_image(
        &image_data,
        width as i32,
        height as i32,
        3,
        3 * width as i32,
    )
    .unwrap();
    let text = api.get_utf8_text().unwrap();
    assert!(!text.is_empty());

    // Clear and set new image -- should still work
    api.clear().unwrap();
    let (digits_data, dw, dh) = load_test_image("digits.png").unwrap();
    api.set_image(&digits_data, dw as i32, dh as i32, 3, 3 * dw as i32)
        .unwrap();
    let text2 = api.get_utf8_text().unwrap();
    assert!(!text2.is_empty());
}
