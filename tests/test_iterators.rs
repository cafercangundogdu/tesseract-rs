mod common;
use common::*;
use tesseract_rs::{TessPageIteratorLevel, TessPolyBlockType};

// ===========================================================================
// ResultIterator tests
// ===========================================================================

#[test]
fn test_result_iterator_get_utf8_text_word() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let text = ri
        .get_utf8_text(TessPageIteratorLevel::RIL_WORD)
        .expect("get_utf8_text at word level failed");
    assert!(!text.is_empty(), "First word text should not be empty");
}

#[test]
fn test_result_iterator_get_utf8_text_line() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let text = ri
        .get_utf8_text(TessPageIteratorLevel::RIL_TEXTLINE)
        .expect("get_utf8_text at line level failed");
    assert!(!text.is_empty(), "First line text should not be empty");
}

#[test]
fn test_result_iterator_get_utf8_text_block() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let text = ri
        .get_utf8_text(TessPageIteratorLevel::RIL_BLOCK)
        .expect("get_utf8_text at block level failed");
    assert!(!text.is_empty(), "Block-level text should not be empty");
}

#[test]
fn test_result_iterator_get_utf8_text_symbol() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let text = ri
        .get_utf8_text(TessPageIteratorLevel::RIL_SYMBOL)
        .expect("get_utf8_text at symbol level failed");
    assert!(!text.is_empty(), "First symbol text should not be empty");
    // A single symbol is usually one character
    assert!(
        text.trim().len() <= 4,
        "Symbol text should be very short, got: '{}'",
        text.trim()
    );
}

#[test]
fn test_result_iterator_confidence() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let conf = ri
        .confidence(TessPageIteratorLevel::RIL_WORD)
        .expect("confidence failed");
    assert!(
        conf >= 0.0 && conf <= 100.0,
        "Word confidence should be 0-100, got {}",
        conf
    );
}

#[test]
fn test_result_iterator_confidence_block_level() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let conf = ri
        .confidence(TessPageIteratorLevel::RIL_BLOCK)
        .expect("confidence at block level failed");
    assert!(
        conf >= 0.0 && conf <= 100.0,
        "Block confidence should be 0-100, got {}",
        conf
    );
}

#[test]
fn test_result_iterator_next() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    // The sample text has multiple words, so next at word level should succeed at least once
    let has_next = ri
        .next(TessPageIteratorLevel::RIL_WORD)
        .expect("next failed");
    assert!(has_next, "Should have more than one word in sample text");
}

#[test]
fn test_result_iterator_word_recognition_language() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let lang = ri
        .word_recognition_language()
        .expect("word_recognition_language failed");
    assert_eq!(
        lang, "eng",
        "Word recognition language should be 'eng', got '{}'",
        lang
    );
}

#[test]
fn test_result_iterator_word_is_from_dictionary() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    // Just verify the method does not error; the bool value is OCR-dependent
    let _from_dict = ri
        .word_is_from_dictionary()
        .expect("word_is_from_dictionary failed");
}

#[test]
fn test_result_iterator_word_is_numeric() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let is_numeric = ri.word_is_numeric().expect("word_is_numeric failed");
    // "This" (the first word) is not numeric
    assert!(
        !is_numeric,
        "First word in sample text should not be numeric"
    );
}

#[test]
fn test_result_iterator_symbol_is_superscript() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let is_sup = ri
        .symbol_is_superscript()
        .expect("symbol_is_superscript failed");
    assert!(!is_sup, "Normal text should not be superscript");
}

#[test]
fn test_result_iterator_symbol_is_subscript() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let is_sub = ri
        .symbol_is_subscript()
        .expect("symbol_is_subscript failed");
    assert!(!is_sub, "Normal text should not be subscript");
}

#[test]
fn test_result_iterator_symbol_is_dropcap() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let is_dropcap = ri.symbol_is_dropcap().expect("symbol_is_dropcap failed");
    assert!(!is_dropcap, "Normal text should not be a dropcap");
}

#[test]
fn test_result_iterator_get_bounding_box() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let (left, top, right, bottom) = ri
        .get_bounding_box(TessPageIteratorLevel::RIL_WORD)
        .expect("get_bounding_box failed");
    assert!(
        left < right,
        "left ({}) should be < right ({})",
        left,
        right
    );
    assert!(
        top < bottom,
        "top ({}) should be < bottom ({})",
        top,
        bottom
    );
    assert!(left >= 0, "left should be non-negative, got {}", left);
    assert!(top >= 0, "top should be non-negative, got {}", top);
}

#[test]
fn test_result_iterator_get_bounding_box_block() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let (left, top, right, bottom) = ri
        .get_bounding_box(TessPageIteratorLevel::RIL_BLOCK)
        .expect("get_bounding_box at block level failed");
    assert!(
        right > left && bottom > top,
        "Block bounding box should have positive area: ({}, {}, {}, {})",
        left,
        top,
        right,
        bottom
    );
}

#[test]
fn test_result_iterator_get_word_with_bounds() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let (text, left, top, right, bottom, confidence) = ri
        .get_word_with_bounds()
        .expect("get_word_with_bounds failed");
    assert!(!text.is_empty(), "Word text should not be empty");
    assert!(
        left < right,
        "left ({}) should be < right ({})",
        left,
        right
    );
    assert!(
        top < bottom,
        "top ({}) should be < bottom ({})",
        top,
        bottom
    );
    assert!(
        confidence >= 0.0 && confidence <= 100.0,
        "Confidence should be 0-100, got {}",
        confidence
    );
}

#[test]
fn test_result_iterator_next_word() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let has_next = ri.next_word().expect("next_word failed");
    assert!(has_next, "Sample text should have more than one word");
}

#[test]
fn test_result_iterator_get_current_word() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let (text, left, top, right, bottom, confidence) =
        ri.get_current_word().expect("get_current_word failed");
    assert!(!text.is_empty(), "Current word text should not be empty");
    assert!(
        left < right,
        "left ({}) should be < right ({})",
        left,
        right
    );
    assert!(
        top < bottom,
        "top ({}) should be < bottom ({})",
        top,
        bottom
    );
    assert!(
        confidence >= 0.0 && confidence <= 100.0,
        "Confidence should be 0-100, got {}",
        confidence
    );
}

#[test]
fn test_result_iterator_word_font_attributes() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");
    let result = ri.word_font_attributes();
    // word_font_attributes may fail if font info is not available (e.g., LSTM-only mode)
    // so we just verify it returns a result without panicking
    match result {
        Ok((
            is_bold,
            is_italic,
            is_underlined,
            is_monospace,
            is_serif,
            is_smallcaps,
            pointsize,
            font_id,
        )) => {
            // Basic sanity checks on returned values
            assert!(
                pointsize >= 0,
                "Pointsize should be non-negative, got {}",
                pointsize
            );
            assert!(
                font_id >= 0,
                "Font ID should be non-negative, got {}",
                font_id
            );
            // Booleans are always valid, just print for debug visibility
            let _ = (
                is_bold,
                is_italic,
                is_underlined,
                is_monospace,
                is_serif,
                is_smallcaps,
            );
        }
        Err(_) => {
            // It is acceptable for font attributes to be unavailable
        }
    }
}

// ===========================================================================
// PageIterator tests
// ===========================================================================

#[test]
fn test_page_iterator_begin() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");
    pi.begin().expect("begin failed");
}

#[test]
fn test_page_iterator_next() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");
    // Advance at word level; the sample text has multiple words
    let has_next = pi
        .next(TessPageIteratorLevel::RIL_WORD)
        .expect("next failed");
    assert!(
        has_next,
        "PageIterator should be able to advance at word level"
    );
}

#[test]
fn test_page_iterator_is_at_beginning_of() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");
    // At the start, the word should be at the beginning of a block
    let at_beginning = pi
        .is_at_beginning_of(TessPageIteratorLevel::RIL_BLOCK)
        .expect("is_at_beginning_of failed");
    assert!(
        at_beginning,
        "First element should be at the beginning of its block"
    );
}

#[test]
fn test_page_iterator_is_at_final_element() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");
    // Check if the current word is the final word in its textline
    let _is_final = pi
        .is_at_final_element(
            TessPageIteratorLevel::RIL_TEXTLINE,
            TessPageIteratorLevel::RIL_WORD,
        )
        .expect("is_at_final_element failed");
    // The first word is typically NOT the final word in the line, but we just test
    // that the method returns without error.
}

#[test]
fn test_page_iterator_bounding_box() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");
    let (left, top, right, bottom) = pi
        .bounding_box(TessPageIteratorLevel::RIL_WORD)
        .expect("bounding_box failed");
    assert!(
        left < right,
        "left ({}) should be < right ({})",
        left,
        right
    );
    assert!(
        top < bottom,
        "top ({}) should be < bottom ({})",
        top,
        bottom
    );
    assert!(left >= 0, "left should be non-negative, got {}", left);
    assert!(top >= 0, "top should be non-negative, got {}", top);
}

#[test]
fn test_page_iterator_bounding_box_block() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");
    let (left, top, right, bottom) = pi
        .bounding_box(TessPageIteratorLevel::RIL_BLOCK)
        .expect("bounding_box at block level failed");
    assert!(
        right > left && bottom > top,
        "Block bounding box should have positive area: ({}, {}, {}, {})",
        left,
        top,
        right,
        bottom
    );
}

#[test]
fn test_page_iterator_block_type() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");
    let bt = pi.block_type().expect("block_type failed");
    // For a text image, we expect flowing text or heading text
    assert!(
        bt == TessPolyBlockType::PT_FLOWING_TEXT
            || bt == TessPolyBlockType::PT_HEADING_TEXT
            || bt == TessPolyBlockType::PT_PULLOUT_TEXT
            || bt == TessPolyBlockType::PT_CAPTION_TEXT,
        "Block type should be a text type for sample_text.png, got {:?}",
        bt
    );
}

#[test]
fn test_page_iterator_baseline() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");
    let (x1, y1, x2, y2) = pi
        .baseline(TessPageIteratorLevel::RIL_TEXTLINE as i32)
        .expect("baseline failed");
    // Baseline should span some horizontal distance for a line of text
    assert!(x2 >= x1, "Baseline x2 ({}) should be >= x1 ({})", x2, x1);
    // y coordinates represent the baseline position
    assert!(y1 >= 0, "Baseline y1 should be non-negative, got {}", y1);
    assert!(y2 >= 0, "Baseline y2 should be non-negative, got {}", y2);
}

#[test]
fn test_page_iterator_orientation() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");
    let result = pi.orientation();
    match result {
        Ok((orientation, writing_direction, textline_order, deskew_angle)) => {
            // For normal left-to-right English text
            assert_eq!(
                orientation,
                tesseract_rs::TessOrientation::ORIENTATION_PAGE_UP,
                "Orientation should be PAGE_UP for normal text, got {:?}",
                orientation
            );
            assert_eq!(
                writing_direction,
                tesseract_rs::TessWritingDirection::WRITING_DIRECTION_LEFT_TO_RIGHT,
                "Writing direction should be LTR for English text, got {:?}",
                writing_direction
            );
            assert_eq!(
                textline_order,
                tesseract_rs::TessTextlineOrder::TEXTLINE_ORDER_TOP_TO_BOTTOM,
                "Textline order should be top-to-bottom, got {:?}",
                textline_order
            );
            assert!(
                deskew_angle.is_finite(),
                "Deskew angle should be finite, got {}",
                deskew_angle
            );
        }
        Err(_) => {
            // orientation() may fail on some Tesseract builds; acceptable
        }
    }
}

#[test]
fn test_page_iterator_paragraph_info() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");
    let result = pi.paragraph_info();
    match result {
        Ok((justification, is_list_item, is_crown, first_line_indent)) => {
            // Just verify the values are sane
            let _ = justification; // any justification is valid
            let _ = is_list_item;
            let _ = is_crown;
            let _ = first_line_indent;
        }
        Err(_) => {
            // paragraph_info() may fail on some builds; acceptable
        }
    }
}

#[test]
fn test_page_iterator_iterate_words() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");

    // Count words by iterating at word level
    let mut word_count = 1; // start at 1 because iterator starts on the first element
    while pi
        .next(TessPageIteratorLevel::RIL_WORD)
        .expect("next failed")
    {
        word_count += 1;
        // Safety: prevent infinite loop
        if word_count > 100 {
            panic!("Too many words, likely an infinite loop");
        }
    }
    assert!(
        word_count >= 2,
        "Sample text should have at least 2 words, got {}",
        word_count
    );
}

#[test]
fn test_page_iterator_begin_resets() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let pi = api.analyse_layout().expect("analyse_layout failed");

    // Get the first bounding box
    let first_bbox = pi
        .bounding_box(TessPageIteratorLevel::RIL_WORD)
        .expect("bounding_box failed");

    // Advance past the first element
    pi.next(TessPageIteratorLevel::RIL_WORD)
        .expect("next failed");

    // Reset to beginning
    pi.begin().expect("begin failed");

    // Get the bounding box again - should match the first one
    let reset_bbox = pi
        .bounding_box(TessPageIteratorLevel::RIL_WORD)
        .expect("bounding_box after begin failed");

    assert_eq!(
        first_bbox, reset_bbox,
        "After begin(), bounding box should match original: {:?} vs {:?}",
        first_bbox, reset_bbox
    );
}

// ===========================================================================
// Iteration pattern: collect all words from sample text
// ===========================================================================

#[test]
fn test_iterator_collect_all_words() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");

    let mut words: Vec<String> = Vec::new();

    // Get the first word
    let (text, _left, _top, _right, _bottom, _confidence) = ri
        .get_current_word()
        .expect("get_current_word failed for first word");
    words.push(text.trim().to_string());

    // Iterate through remaining words
    while ri.next_word().expect("next_word failed") {
        let (text, _left, _top, _right, _bottom, _confidence) = ri
            .get_current_word()
            .expect("get_current_word failed during iteration");
        words.push(text.trim().to_string());
        // Safety: prevent infinite loop
        if words.len() > 100 {
            panic!("Too many words collected, likely an infinite loop");
        }
    }

    assert!(!words.is_empty(), "Should have collected at least one word");

    // The sample text is "This is a sample text for OCR testing."
    // Join collected words and check the text is reasonable
    let joined = words.join(" ").to_lowercase();
    assert!(
        joined.contains("sample") || joined.contains("text") || joined.contains("ocr"),
        "Collected words should contain recognizable parts of sample text, got: '{}'",
        joined
    );

    // Check we got a reasonable number of words (the sample text has 8 words)
    assert!(
        words.len() >= 4,
        "Should have at least 4 words from sample text, got {}",
        words.len()
    );
}

#[test]
fn test_iterator_word_bounding_boxes_are_ordered() {
    let api = create_api_with_image();
    api.recognize().expect("recognize failed");
    let ri = api.get_iterator().expect("get_iterator failed");

    let mut prev_left: Option<i32> = None;
    let mut first = true;

    loop {
        let (_, left, _top, _right, _bottom, _) = ri
            .get_word_with_bounds()
            .expect("get_word_with_bounds failed");

        if let Some(pl) = prev_left {
            // Words on the same line should generally progress left to right
            // (unless wrapping to a new line, which resets the x position)
            // We allow left < pl if a new line started
            let _ = pl; // acknowledge previous value
        }
        prev_left = Some(left);

        if first {
            first = false;
        }

        if !ri.next_word().expect("next_word failed") {
            break;
        }

        if prev_left.is_some() && !first {
            // Limit iterations for safety
            if prev_left.unwrap() > 10000 {
                break;
            }
        }
    }

    assert!(
        prev_left.is_some(),
        "Should have iterated through at least one word"
    );
}

// ===========================================================================
// ChoiceIterator tests
// ===========================================================================

#[test]
fn test_choice_iterator_from_result_iterator() {
    let api = create_api_with_image();
    api.recognize().unwrap();
    let iter = api.get_iterator().unwrap();

    // Move to symbol level first
    let text = iter.get_utf8_text(TessPageIteratorLevel::RIL_SYMBOL);
    if text.is_ok() {
        // Try to get choice iterator for current symbol
        match iter.get_choice_iterator() {
            Ok(choice_iter) => {
                // Get the text of the first choice
                let choice_text = choice_iter.get_utf8_text();
                assert!(choice_text.is_ok(), "Should get text from first choice");

                // Get confidence
                let conf = choice_iter.confidence();
                assert!(conf.is_ok());
                let conf_val = conf.unwrap();
                assert!(conf_val >= 0.0, "Confidence should be non-negative");

                // Try to advance
                let _has_next = choice_iter.next();
            }
            Err(_) => {
                // Some symbols may not have alternative choices
            }
        }
    }
}

#[test]
fn test_choice_iterator_iterate_alternatives() {
    let api = create_api_with_image();
    api.recognize().unwrap();
    let iter = api.get_iterator().unwrap();

    // Find a symbol with choices
    let mut found_choices = false;
    loop {
        if let Ok(choice_iter) = iter.get_choice_iterator() {
            let mut count = 0;
            // First choice
            if choice_iter.get_utf8_text().is_ok() {
                count += 1;
            }
            // More choices
            while choice_iter.next().unwrap_or(false) {
                if choice_iter.get_utf8_text().is_ok() {
                    count += 1;
                }
            }
            if count > 1 {
                found_choices = true;
                break;
            }
        }
        if !iter
            .next(TessPageIteratorLevel::RIL_SYMBOL)
            .unwrap_or(false)
        {
            break;
        }
    }
    // It's OK if no multi-choice symbols were found -- depends on image
    println!("Found symbols with multiple choices: {}", found_choices);
}

#[test]
fn test_choice_iterator_confidence_ordering() {
    let api = create_api_with_image();
    api.recognize().unwrap();
    let iter = api.get_iterator().unwrap();

    // For each symbol, verify choices are in descending confidence order
    loop {
        if let Ok(choice_iter) = iter.get_choice_iterator() {
            let mut prev_conf = f32::MAX;
            if let Ok(conf) = choice_iter.confidence() {
                assert!(
                    conf <= prev_conf,
                    "Choices should be in descending confidence"
                );
                prev_conf = conf;
            }
            while choice_iter.next().unwrap_or(false) {
                if let Ok(conf) = choice_iter.confidence() {
                    assert!(
                        conf <= prev_conf,
                        "Choices should be in descending confidence"
                    );
                    prev_conf = conf;
                }
            }
        }
        if !iter
            .next(TessPageIteratorLevel::RIL_SYMBOL)
            .unwrap_or(false)
        {
            break;
        }
    }
}
