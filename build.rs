use std::env;
use std::path::PathBuf;
use std::fs;
use cmake::Config;
use reqwest::blocking::Client;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    download_tessdata(&project_dir);

    let leptonica_dir = project_dir.join("third_party/leptonica");
    let leptonica_install_dir = out_dir.join("leptonica");
    let leptonica = Config::new(&leptonica_dir)
        .define("BUILD_PROG", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("ENABLE_ZLIB", "OFF")
        .define("ENABLE_PNG", "OFF")
        .define("ENABLE_JPEG", "OFF")
        .define("ENABLE_TIFF", "OFF")
        .define("ENABLE_WEBP", "OFF")
        .define("ENABLE_OPENJPEG", "OFF")
        .define("ENABLE_GIF", "OFF")
        .define("CMAKE_INSTALL_PREFIX", &leptonica_install_dir)
        .build();

    let leptonica_include_dir = leptonica_install_dir.join("include");
    let leptonica_lib_dir = leptonica_install_dir.join("lib");

    let tesseract_dir = project_dir.join("third_party/tesseract");
    let tesseract_install_dir = out_dir.join("tesseract");
    let tessdata_prefix = project_dir.join("tessdata");
    let tesseract = Config::new(&tesseract_dir)
        .define("BUILD_TRAINING_TOOLS", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("DISABLE_ARCHIVE", "ON")
        .define("DISABLE_CURL", "ON")
        .define("DISABLE_OPENCL", "ON")
        .define("Leptonica_DIR", &leptonica_install_dir)
        .define("LEPTONICA_INCLUDE_DIR", &leptonica_include_dir)
        .define("LEPTONICA_LIBRARY", &leptonica_lib_dir)
        .define("CMAKE_PREFIX_PATH", &leptonica_install_dir)
        .define("CMAKE_INSTALL_PREFIX", &tesseract_install_dir)
        .define("TESSDATA_PREFIX", &tessdata_prefix)
        .build();

    println!("cargo:rustc-link-search=native={}", leptonica_lib_dir.display());
    println!("cargo:rustc-link-search=native={}", tesseract_install_dir.join("lib").display());
    println!("cargo:rustc-link-lib=static=leptonica");
    println!("cargo:rustc-link-lib=static=tesseract");

    println!("cargo:rustc-link-lib=stdc++");

    println!("cargo:warning=Leptonica include dir: {:?}", leptonica_include_dir);
    println!("cargo:warning=Leptonica lib dir: {:?}", leptonica_lib_dir);
    println!("cargo:warning=Tesseract install dir: {:?}", tesseract_install_dir);
    println!("cargo:warning=Tessdata dir: {:?}", tessdata_prefix);
}

fn download_tessdata(project_dir: &PathBuf) {
    let tessdata_dir = project_dir.join("tessdata");
    fs::create_dir_all(&tessdata_dir).expect("Failed to create Tessdata directory");

    let languages = ["eng", "tur"];
    let base_url = "https://github.com/tesseract-ocr/tessdata_best/raw/refs/heads/main/";
    let client = Client::new();

    for lang in &languages {
        let filename = format!("{}.traineddata", lang);
        let file_path = tessdata_dir.join(&filename);
        
        if !file_path.exists() {
            let url = format!("{}{}", base_url, filename);
            let response = client.get(&url).send().expect("Failed to download Tessdata");
            let mut dest = fs::File::create(&file_path).expect("Failed to create file");
            std::io::copy(&mut response.bytes().expect("Failed to get response bytes").as_ref(), &mut dest)
                .expect("Failed to write Tessdata");
            println!("cargo:warning={} downloaded", filename);
        } else {
            println!("cargo:warning={} already exists, skipping download", filename);
        }
    }
}