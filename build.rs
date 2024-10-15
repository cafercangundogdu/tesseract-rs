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

        let (cmake_cxx_flags, additional_defines) = get_os_specific_config();

        // Build Leptonica
        let leptonica_install_dir = out_dir.join("leptonica");
        let mut leptonica_config = Config::new(&leptonica_dir);
        if env::var("RUSTC_WRAPPER").unwrap_or_default() == "sccache" {
            leptonica_config.env("CC", "sccache cc")
                  .env("CXX", "sccache c++");
        }
        leptonica_config
            .define("BUILD_PROG", "OFF")
            .define("BUILD_SHARED_LIBS", "OFF")
            .define("ENABLE_ZLIB", "OFF")
            .define("ENABLE_PNG", "OFF")
            .define("ENABLE_JPEG", "OFF")
            .define("ENABLE_TIFF", "OFF")
            .define("ENABLE_WEBP", "OFF")
            .define("ENABLE_OPENJPEG", "OFF")
            .define("ENABLE_GIF", "OFF")
            .define("CMAKE_CXX_FLAGS", &cmake_cxx_flags)
            .define("CMAKE_INSTALL_PREFIX", &leptonica_install_dir);
            

        for (key, value) in &additional_defines {
            leptonica_config.define(key, value);
        }

        let leptonica = leptonica_config.build();

        let leptonica_include_dir = leptonica_install_dir.join("include");
        let leptonica_lib_dir = leptonica_install_dir.join("lib");

        // Build Tesseract
        let tesseract_install_dir = out_dir.join("tesseract");
        let tessdata_prefix = project_dir.join("tessdata");
        let mut tesseract_config = Config::new(&tesseract_dir);
        if env::var("RUSTC_WRAPPER").unwrap_or_default() == "sccache" {
            tesseract_config.env("CC", "sccache cc")
                  .env("CXX", "sccache c++");
        }
        tesseract_config
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
            .define("CMAKE_CXX_FLAGS", &cmake_cxx_flags);

        for (key, value) in &additional_defines {
            tesseract_config.define(key, value);
        }

        let tesseract = tesseract_config.build();

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed={}", third_party_dir.display());
        println!("cargo:rerun-if-changed={}", leptonica_dir.display());
        println!("cargo:rerun-if-changed={}", tesseract_dir.display());

        println!("cargo:rustc-link-search=native={}", leptonica_lib_dir.display());
        println!("cargo:rustc-link-search=native={}", tesseract_install_dir.join("lib").display());
        println!("cargo:rustc-link-lib=static=leptonica");
        println!("cargo:rustc-link-lib=static=tesseract");

        set_os_specific_link_flags();

        println!("cargo:warning=Leptonica include dir: {:?}", leptonica_include_dir);
        println!("cargo:warning=Leptonica lib dir: {:?}", leptonica_lib_dir);
        println!("cargo:warning=Tesseract install dir: {:?}", tesseract_install_dir);
        println!("cargo:warning=Tessdata dir: {:?}", tessdata_prefix);

        download_tessdata(&project_dir);
    }

    fn get_os_specific_config() -> (String, Vec<(String, String)>) {
        let mut cmake_cxx_flags = String::new();
        let mut additional_defines = Vec::new();
    
        if cfg!(target_os = "macos") {
            cmake_cxx_flags.push_str("-stdlib=libc++ ");
            cmake_cxx_flags.push_str("-std=c++11 ");
        } else if cfg!(target_os = "linux") {
            cmake_cxx_flags.push_str("-std=c++11 ");
            // Check if we're on a system using clang
            if cfg!(target_env = "musl") || env::var("CC").map(|cc| cc.contains("clang")).unwrap_or(false) {
                cmake_cxx_flags.push_str("-stdlib=libc++ ");
                additional_defines.push(("CMAKE_CXX_COMPILER".to_string(), "clang++".to_string()));
            } else {
                // Assume GCC
                additional_defines.push(("CMAKE_CXX_COMPILER".to_string(), "g++".to_string()));
            }
        } else if cfg!(target_os = "windows") {
            // Windows-specific MSVC flags
            cmake_cxx_flags.push_str("/EHsc /MP ");
            additional_defines.push(("CMAKE_CXX_FLAGS_RELEASE".to_string(), "/MD".to_string()));
            additional_defines.push(("CMAKE_CXX_FLAGS_DEBUG".to_string(), "/MDd".to_string()));
        }
    
        // Common flags and defines for all platforms
        cmake_cxx_flags.push_str("-DUSE_STD_NAMESPACE ");
        additional_defines.push(("CMAKE_POSITION_INDEPENDENT_CODE".to_string(), "ON".to_string()));
    
        (cmake_cxx_flags, additional_defines)
    }

    fn set_os_specific_link_flags() {
        if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-lib=c++");
        } else if cfg!(target_os = "linux") {
            if cfg!(target_env = "musl") || env::var("CC").map(|cc| cc.contains("clang")).unwrap_or(false) {
                println!("cargo:rustc-link-lib=c++");
            } else {
                println!("cargo:rustc-link-lib=stdc++");
            }
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=m");
            println!("cargo:rustc-link-lib=dl");
        } else if cfg!(target_os = "windows") {
            // Additional linker flags are generally not required for Windows,
            // as MSVC automatically links the necessary libraries.
            // However, for some special cases, additions can be made as follows:
            // println!("cargo:rustc-link-lib=user32");
            // println!("cargo:rustc-link-lib=gdi32");
        }
    
        println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").unwrap());
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
