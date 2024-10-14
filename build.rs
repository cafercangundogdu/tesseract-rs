#[cfg(feature = "build-tesseract")]
mod build_tesseract {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use cmake::Config;

    const LEPTONICA_URL: &str = "https://github.com/DanBloomberg/leptonica/archive/refs/heads/master.zip";
    const TESSERACT_URL: &str = "https://github.com/tesseract-ocr/tesseract/archive/refs/heads/main.zip";

    pub fn build() {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let third_party_dir = project_dir.join("third_party");

        let (leptonica_dir, tesseract_dir) = if third_party_dir.exists() {
            // Submodule scenario
            (
                third_party_dir.join("leptonica"),
                third_party_dir.join("tesseract"),
            )
        } else {
            // Download scenario
            fs::create_dir_all(&third_party_dir).expect("Failed to create third_party directory");
            (
                download_and_extract(&third_party_dir, LEPTONICA_URL, "leptonica"),
                download_and_extract(&third_party_dir, TESSERACT_URL, "tesseract"),
            )
        };

        // Build Leptonica
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

        // Build Tesseract
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

        download_tessdata(&project_dir);
    }

    fn download_and_extract(target_dir: &Path, url: &str, name: &str) -> PathBuf {
        use reqwest::blocking::Client;
        use zip::ZipArchive;
        use std::io::Read;
    
        let client = Client::new();
        let mut response = client.get(url).send().expect("Failed to download archive");
        let mut content = Vec::new();
        response.copy_to(&mut content).expect("Failed to read archive content");
    
        let temp_file = target_dir.join(format!("{}.zip", name));
        fs::write(&temp_file, content).expect("Failed to write archive to file");
    
        let extract_dir = target_dir.join(name);
        if extract_dir.exists() {
            fs::remove_dir_all(&extract_dir).expect("Failed to remove existing directory");
        }
        fs::create_dir_all(&extract_dir).expect("Failed to create extraction directory");
    
        let mut archive = ZipArchive::new(fs::File::open(&temp_file).unwrap()).unwrap();
    
        // Extract files, ignoring the top-level directory
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let file_path = file.sanitized_name();
            let file_path = file_path.to_str().unwrap();
            
            // Skip the top-level directory
            let path = Path::new(file_path);
            let path = path.strip_prefix(path.components().next().unwrap()).unwrap();
            
            if path.as_os_str().is_empty() {
                continue;
            }
    
            let target_path = extract_dir.join(path);
    
            if file.is_dir() {
                fs::create_dir_all(target_path).unwrap();
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                let mut outfile = fs::File::create(target_path).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }
        }
    
        fs::remove_file(temp_file).expect("Failed to remove temporary zip file");
    
        extract_dir
    }

    fn download_tessdata(project_dir: &Path) {
        let tessdata_dir = project_dir.join("tessdata");
        fs::create_dir_all(&tessdata_dir).expect("Failed to create Tessdata directory");

        let languages = ["eng", "tur"];
        let base_url = "https://github.com/tesseract-ocr/tessdata_best/raw/main/";
        let client = reqwest::blocking::Client::new();

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
}

fn main() {
    #[cfg(feature = "build-tesseract")]
    build_tesseract::build();
}