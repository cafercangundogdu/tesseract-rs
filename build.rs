#[cfg(feature = "build-tesseract")]
mod build_tesseract {
    use std::env;
    use std::path::{Path, PathBuf};
    use std::fs;
    use cmake::Config;
    use reqwest::blocking::Client;
    use std::process::Command;
    
    pub fn build() {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

        if should_init_submodules(&project_dir, &out_dir) {
            init_submodules(&project_dir);
            mark_submodules_initialized(&out_dir);
        }

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

    fn should_init_submodules(project_dir: &Path, out_dir: &Path) -> bool {
        let submodule_marker = out_dir.join("submodules_initialized");
        if submodule_marker.exists() {
            return false;
        }

        let leptonica_dir = project_dir.join("third_party/leptonica");
        let tesseract_dir = project_dir.join("third_party/tesseract");

        !leptonica_dir.exists() || !tesseract_dir.exists()
    }

    fn init_submodules(project_dir: &Path) {
        // Remove all submodules
        run_git_command(project_dir, &["submodule", "deinit", "-f", "--all"]);

        // Remove .git/modules directory
        let git_modules_path = project_dir.join(".git/modules");
        if git_modules_path.exists() {
            fs::remove_dir_all(git_modules_path).expect("Failed to remove .git/modules");
        }

        // Add submodules
        run_git_command(project_dir, &["submodule", "add", "-f", "https://github.com/DanBloomberg/leptonica.git", "third_party/leptonica"]);
        run_git_command(project_dir, &["submodule", "add", "-f", "https://github.com/tesseract-ocr/tesseract.git", "third_party/tesseract"]);

        // Update submodules
        run_git_command(project_dir, &["submodule", "update", "--init", "--recursive", "--force"]);

        // Run git gc
        run_git_command(project_dir, &["gc", "--aggressive", "--prune=now"]);

        // Run git fsck
        run_git_command(project_dir, &["fsck", "--full"]);
    }

    fn run_git_command(project_dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(project_dir)
            .args(args)
            .status()
            .expect(&format!("Failed to run git command: {:?}", args));

        if !status.success() {
            panic!("Git command failed: {:?}", args);
        }
    }

    fn mark_submodules_initialized(out_dir: &Path) {
        let submodule_marker = out_dir.join("submodules_initialized");
        fs::write(submodule_marker, "").expect("Failed to create submodule marker file");
    }
}



fn main() {
    #[cfg(feature = "build-tesseract")]
    build_tesseract::build();
}