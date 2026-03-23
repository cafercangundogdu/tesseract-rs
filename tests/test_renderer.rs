mod common;
use common::*;
use tesseract_rs::TessResultRenderer;

#[test]
fn test_text_renderer_create() {
    let tmp = std::env::temp_dir().join("tesseract_test_renderer_text");
    let renderer = TessResultRenderer::new_text_renderer(tmp.to_str().unwrap());
    assert!(renderer.is_ok());
    // cleanup
    let _ = std::fs::remove_file(format!("{}.txt", tmp.display()));
}

#[test]
fn test_hocr_renderer_create() {
    let tmp = std::env::temp_dir().join("tesseract_test_renderer_hocr");
    let renderer = TessResultRenderer::new_hocr_renderer(tmp.to_str().unwrap());
    assert!(renderer.is_ok());
}

#[test]
fn test_pdf_renderer_create() {
    let tessdata_dir = get_tessdata_dir();
    let tmp = std::env::temp_dir().join("tesseract_test_renderer_pdf");
    let renderer = TessResultRenderer::new_pdf_renderer(
        tmp.to_str().unwrap(),
        tessdata_dir.to_str().unwrap(),
        false,
    );
    assert!(renderer.is_ok());
}

#[test]
fn test_text_renderer_workflow() {
    let tmp = std::env::temp_dir().join("tesseract_test_renderer_wf");
    let renderer = TessResultRenderer::new_text_renderer(tmp.to_str().unwrap()).unwrap();

    let began = renderer.begin_document("Test Document").unwrap();
    assert!(began);

    let api = create_api_with_image();
    let added = renderer.add_image(&api).unwrap();
    assert!(added);

    let ended = renderer.end_document().unwrap();
    assert!(ended);

    // cleanup
    let _ = std::fs::remove_file(format!("{}.txt", tmp.display()));
}

#[test]
fn test_renderer_get_extension() {
    let tmp = std::env::temp_dir().join("tesseract_test_renderer_ext");
    let renderer = TessResultRenderer::new_text_renderer(tmp.to_str().unwrap()).unwrap();
    let ext = renderer.get_extension().unwrap();
    assert_eq!(ext, "txt");
}

#[test]
fn test_renderer_get_title() {
    let tmp = std::env::temp_dir().join("tesseract_test_renderer_title");
    let renderer = TessResultRenderer::new_text_renderer(tmp.to_str().unwrap()).unwrap();
    renderer.begin_document("My Title").unwrap();
    let title = renderer.get_title().unwrap();
    assert_eq!(title, "My Title");
}

#[test]
fn test_renderer_get_image_num_initial() {
    let tmp = std::env::temp_dir().join("tesseract_test_renderer_imgnum");
    let renderer = TessResultRenderer::new_text_renderer(tmp.to_str().unwrap()).unwrap();
    let num = renderer.get_image_num().unwrap();
    assert_eq!(num, -1);
}

#[test]
fn test_renderer_get_image_num_after_add() {
    let tmp = std::env::temp_dir().join("tesseract_test_renderer_imgnum2");
    let renderer = TessResultRenderer::new_text_renderer(tmp.to_str().unwrap()).unwrap();
    renderer.begin_document("Test").unwrap();
    let api = create_api_with_image();
    renderer.add_image(&api).unwrap();
    let num = renderer.get_image_num().unwrap();
    assert_eq!(num, 0);
    renderer.end_document().unwrap();
    let _ = std::fs::remove_file(format!("{}.txt", tmp.display()));
}
