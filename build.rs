#![allow(clippy::uninlined_format_args)]
// When both `build-tesseract` and `use-system-tesseract` are enabled, the
// bundled-build module is compiled (it hosts shared helpers) but its build
// entry point is not called — silence the resulting dead-code warnings.
#![cfg_attr(
    all(feature = "build-tesseract", feature = "use-system-tesseract"),
    allow(dead_code)
)]

#[cfg(feature = "build-tesseract")]
mod build_tesseract {
    use cmake::Config;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};

    // Use specific release versions for stability.
    // NOTE: when bumping these, also update the Windows library name variants
    // in build_or_use_cached() (leptonica-<ver>.lib and tesseract<MAJ><MIN>.lib).
    const LEPTONICA_URL: &str =
        "https://github.com/DanBloomberg/leptonica/archive/refs/tags/1.87.0.zip";
    const TESSERACT_URL: &str =
        "https://github.com/tesseract-ocr/tesseract/archive/refs/tags/5.5.2.zip";
    const TESSDATA_BASE_URL: &str = "https://github.com/tesseract-ocr/tessdata_best/raw/main/";

    /// Languages every bundled build downloads into the shared tessdata
    /// directory. Also the default set for the `embed-tessdata` feature.
    pub(crate) const DEFAULT_LANGUAGES: &[&str] = &["eng", "tur"];

    pub fn get_custom_out_dir() -> PathBuf {
        if cfg!(target_os = "macos") {
            let home_dir = env::var("HOME").unwrap_or_else(|_| {
                env::var("USER")
                    .map(|user| format!("/Users/{}", user))
                    .expect("Neither HOME nor USER environment variable set")
            });
            PathBuf::from(home_dir)
                .join("Library")
                .join("Application Support")
                .join("tesseract-rs")
        } else if cfg!(target_os = "linux") {
            let home_dir = env::var("HOME").unwrap_or_else(|_| {
                env::var("USER")
                    .map(|user| format!("/home/{}", user))
                    .expect("Neither HOME nor USER environment variable set")
            });
            PathBuf::from(home_dir).join(".tesseract-rs")
        } else if cfg!(target_os = "freebsd") {
            let home_dir = env::var("HOME").unwrap_or_else(|_| {
                env::var("USER")
                    .map(|user| format!("/home/{}", user))
                    .expect("Neither HOME nor USER environment variable set")
            });
            PathBuf::from(home_dir).join(".tesseract-rs")
        } else if cfg!(target_os = "windows") {
            env::var("APPDATA")
                .or_else(|_| env::var("USERPROFILE").map(|p| format!("{}\\AppData\\Roaming", p)))
                .map(PathBuf::from)
                .expect("Neither APPDATA nor USERPROFILE environment variable set")
                .join("tesseract-rs")
        } else {
            panic!("Unsupported operating system");
        }
    }

    pub fn build() {
        let custom_out_dir = get_custom_out_dir();
        std::fs::create_dir_all(&custom_out_dir).expect("Failed to create custom out directory");

        println!("cargo:warning=custom_out_dir: {:?}", custom_out_dir);

        // Everything compiled here depends on the crate version (bundled
        // library versions, source patches, build flags), on the target and,
        // for MSVC, on the C runtime. Each combination gets its own directory
        // so that it is not handed libraries built for another one. Compiler
        // settings taken from the environment (CC, CFLAGS, ...) are not part
        // of the key; CARGO_CLEAN forces a rebuild after changing those.
        // Only tessdata stays at the top level, where it is shared.
        let version_dir = custom_out_dir.join("build").join(env!("CARGO_PKG_VERSION"));
        let out_dir = version_dir.join(target_dir_name());

        // Without this cargo would not rerun the build script when the
        // variable is set, and the clean request would be ignored.
        println!("cargo:rerun-if-env-changed=CARGO_CLEAN");
        if env::var("CARGO_CLEAN").is_ok() {
            clean_build_dir(&out_dir);
        }

        let cache_dir = out_dir.join("cache");
        std::fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");

        let project_dir = custom_out_dir.clone();
        let third_party_dir = version_dir.join("third_party");

        let leptonica_dir = if third_party_dir.join("leptonica").exists() {
            println!("cargo:warning=Using existing leptonica source");
            third_party_dir.join("leptonica")
        } else {
            fs::create_dir_all(&third_party_dir).expect("Failed to create third_party directory");
            download_and_extract(&third_party_dir, LEPTONICA_URL, "leptonica")
        };

        let tesseract_dir = if third_party_dir.join("tesseract").exists() {
            println!("cargo:warning=Using existing tesseract source");
            third_party_dir.join("tesseract")
        } else {
            fs::create_dir_all(&third_party_dir).expect("Failed to create third_party directory");
            download_and_extract(&third_party_dir, TESSERACT_URL, "tesseract")
        };

        let (cmake_cxx_flags, additional_defines) = get_os_specific_config();

        let leptonica_install_dir = out_dir.join("leptonica");
        let leptonica_cache_dir = cache_dir.join("leptonica");

        build_or_use_cached(
            "leptonica",
            &leptonica_cache_dir,
            &leptonica_install_dir,
            || {
                let mut leptonica_config = Config::new(&leptonica_dir);

                let leptonica_src_dir = leptonica_dir.join("src");
                let environ_h_path = leptonica_src_dir.join("environ.h");

                // Only modify environ.h if it exists
                if environ_h_path.exists() {
                    patch_source(&environ_h_path, |source| {
                        let patched = source.replace(
                            "#define  HAVE_LIBZ          1",
                            "#define  HAVE_LIBZ          0",
                        );
                        // The replacement contains the text it replaces, so
                        // only apply it once.
                        let force_no_console_io = "#define NO_CONSOLE_IO\n#ifdef  NO_CONSOLE_IO";
                        if patched.contains(force_no_console_io) {
                            patched
                        } else {
                            patched.replace("#ifdef  NO_CONSOLE_IO", force_no_console_io)
                        }
                    });
                }

                let makefile_static_path = leptonica_dir.join("prog").join("makefile.static");

                // Only modify makefile.static if it exists
                if makefile_static_path.exists() {
                    patch_source(&makefile_static_path, |source| {
                        source.replace(
                            "ALL_LIBS =	$(LEPTLIB) -ltiff -ljpeg -lpng -lz -lm",
                            "ALL_LIBS =	$(LEPTLIB) -lm",
                        )
                    });
                }

                // Configure build tools
                if cfg!(target_os = "windows") {
                    // Use NMake on Windows for better compatibility
                    if let Ok(_vs_install_dir) = env::var("VSINSTALLDIR") {
                        leptonica_config.generator("NMake Makefiles");
                    }
                }

                // Only use sccache if not in CI
                if env::var("CI").is_err()
                    && env::var("RUSTC_WRAPPER").unwrap_or_default() == "sccache"
                {
                    leptonica_config
                        .env("CC", "sccache cc")
                        .env("CXX", "sccache c++");
                }
                leptonica_config
                    // `profile` also selects the configuration built by
                    // multi-config generators (Visual Studio), which would
                    // otherwise follow the cargo profile.
                    .profile("Release")
                    .define("CMAKE_POLICY_VERSION_MINIMUM", "3.5")
                    .define("CMAKE_BUILD_TYPE", "Release")
                    .define("BUILD_PROG", "OFF")
                    .define("BUILD_SHARED_LIBS", "OFF")
                    .define("ENABLE_ZLIB", "OFF")
                    .define("ENABLE_PNG", "OFF")
                    .define("ENABLE_JPEG", "OFF")
                    .define("ENABLE_TIFF", "OFF")
                    .define("ENABLE_WEBP", "OFF")
                    .define("ENABLE_OPENJPEG", "OFF")
                    .define("ENABLE_GIF", "OFF")
                    .define("NO_CONSOLE_IO", "ON")
                    .define("CMAKE_CXX_FLAGS", &cmake_cxx_flags)
                    .define("MINIMUM_SEVERITY", "L_SEVERITY_NONE")
                    .define("SW_BUILD", "OFF")
                    .define("HAVE_LIBZ", "0")
                    .define("ENABLE_LTO", "OFF")
                    .define("CMAKE_INSTALL_PREFIX", &leptonica_install_dir);

                for (key, value) in &additional_defines {
                    leptonica_config.define(key, value);
                }

                leptonica_config.build();
            },
        );

        let leptonica_include_dir = leptonica_install_dir.join("include");
        let leptonica_lib_dir = leptonica_install_dir.join("lib");
        let tesseract_install_dir = out_dir.join("tesseract");
        let tesseract_cache_dir = cache_dir.join("tesseract");
        let tessdata_prefix = project_dir.join("tessdata");

        build_or_use_cached(
            "tesseract",
            &tesseract_cache_dir,
            &tesseract_install_dir,
            || {
                let cmakelists_path = tesseract_dir.join("CMakeLists.txt");
                patch_source(&cmakelists_path, |source| {
                    source.replace("set(HAVE_TIFFIO_H ON)", "")
                });

                let mut tesseract_config = Config::new(&tesseract_dir);
                // Configure build tools
                if cfg!(target_os = "windows") {
                    // Use NMake on Windows for better compatibility
                    if let Ok(_vs_install_dir) = env::var("VSINSTALLDIR") {
                        tesseract_config.generator("NMake Makefiles");
                    }
                }

                // Only use sccache if not in CI
                if env::var("CI").is_err()
                    && env::var("RUSTC_WRAPPER").unwrap_or_default() == "sccache"
                {
                    tesseract_config
                        .env("CC", "sccache cc")
                        .env("CXX", "sccache c++");
                }
                tesseract_config
                    .profile("Release")
                    .define("CMAKE_POLICY_VERSION_MINIMUM", "3.5")
                    .define("CMAKE_BUILD_TYPE", "Release")
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
                    .define("DISABLE_TIFF", "ON")
                    .define("DISABLE_PNG", "ON")
                    .define("DISABLE_JPEG", "ON")
                    .define("DISABLE_WEBP", "ON")
                    .define("DISABLE_OPENJPEG", "ON")
                    .define("DISABLE_ZLIB", "ON")
                    .define("DISABLE_LIBXML2", "ON")
                    .define("DISABLE_LIBICU", "ON")
                    .define("DISABLE_LZMA", "ON")
                    .define("DISABLE_GIF", "ON")
                    .define("DISABLE_DEBUG_MESSAGES", "ON")
                    .define("debug_file", "/dev/null")
                    .define("HAVE_LIBARCHIVE", "OFF")
                    .define("HAVE_LIBCURL", "OFF")
                    .define("HAVE_TIFFIO_H", "OFF")
                    .define("GRAPHICS_DISABLED", "ON")
                    .define("DISABLED_LEGACY_ENGINE", "OFF")
                    .define("USE_OPENCL", "OFF")
                    .define("OPENMP_BUILD", "OFF")
                    .define("BUILD_TESTS", "OFF")
                    .define("ENABLE_LTO", "OFF")
                    .define("BUILD_PROG", "OFF")
                    .define("SW_BUILD", "OFF")
                    .define("LEPT_TIFF_RESULT", "FALSE")
                    .define("INSTALL_CONFIGS", "ON")
                    .define("USE_SYSTEM_ICU", "ON")
                    .define("CMAKE_CXX_FLAGS", &cmake_cxx_flags);

                for (key, value) in &additional_defines {
                    tesseract_config.define(key, value);
                }

                tesseract_config.build();
            },
        );

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed={}", third_party_dir.display());
        println!("cargo:rerun-if-changed={}", leptonica_dir.display());
        println!("cargo:rerun-if-changed={}", tesseract_dir.display());

        println!(
            "cargo:rustc-link-search=native={}",
            leptonica_lib_dir.display()
        );
        println!(
            "cargo:rustc-link-search=native={}",
            tesseract_install_dir.join("lib").display()
        );
        // Don't emit link directives here - let build_or_use_cached handle it

        set_os_specific_link_flags();

        println!(
            "cargo:warning=Leptonica include dir: {:?}",
            leptonica_include_dir
        );
        println!("cargo:warning=Leptonica lib dir: {:?}", leptonica_lib_dir);
        println!(
            "cargo:warning=Tesseract install dir: {:?}",
            tesseract_install_dir
        );
        println!("cargo:warning=Tessdata dir: {:?}", tessdata_prefix);

        download_tessdata(&project_dir, DEFAULT_LANGUAGES);
    }

    fn get_os_specific_config() -> (String, Vec<(String, String)>) {
        let mut cmake_cxx_flags = String::new();
        let mut additional_defines = Vec::new();

        if cfg!(target_os = "macos") {
            cmake_cxx_flags.push_str("-stdlib=libc++ ");
            cmake_cxx_flags.push_str("-std=c++17 ");
            // Pin the deployment target to 10.15, the minimum macOS version
            // that provides std::filesystem (used by Tesseract 5.x
            // baseapi.cpp). Because CMAKE_CXX_FLAGS is set explicitly below,
            // the cmake crate skips injecting the cc crate's
            // -mmacosx-version-min into the C++ flags (it only does so for
            // C/ASM). Without an explicit deployment target, Xcode 26+
            // runners derive a default below 10.15 and the build fails
            // (issue #32). CMAKE_OSX_DEPLOYMENT_TARGET is the authoritative
            // cmake-level setting; the flag in CMAKE_CXX_FLAGS is redundant
            // but keeps the two consistent.
            cmake_cxx_flags.push_str("-mmacosx-version-min=10.15 ");
            additional_defines.push((
                "CMAKE_OSX_DEPLOYMENT_TARGET".to_string(),
                "10.15".to_string(),
            ));
        } else if cfg!(target_os = "linux") {
            cmake_cxx_flags.push_str("-std=c++17 ");
            // Check if we're on a system using clang
            if cfg!(target_env = "musl")
                || env::var("CC")
                    .map(|cc| cc.contains("clang"))
                    .unwrap_or(false)
            {
                cmake_cxx_flags.push_str("-stdlib=libc++ ");
                additional_defines.push(("CMAKE_CXX_COMPILER".to_string(), "clang++".to_string()));
            } else {
                // Assume GCC
                additional_defines.push(("CMAKE_CXX_COMPILER".to_string(), "g++".to_string()));
            }
        } else if cfg!(target_os = "freebsd") {
            cmake_cxx_flags.push_str("-std=c++17 ");
            // FreeBSD typically uses clang by default
            cmake_cxx_flags.push_str("-stdlib=libc++ ");
            additional_defines.push(("CMAKE_CXX_COMPILER".to_string(), "clang++".to_string()));
        } else if cfg!(target_os = "windows") {
            // Windows-specific MSVC flags
            cmake_cxx_flags.push_str("/EHsc /MP /std:c++17 ");
            // No /MD or /MT here: the runtime is selected through
            // CMAKE_MSVC_RUNTIME_LIBRARY below, and a flag in the per-config
            // variables would override it.
            additional_defines.push(("CMAKE_C_FLAGS_RELEASE".to_string(), "/O2".to_string()));
            additional_defines.push(("CMAKE_CXX_FLAGS_RELEASE".to_string(), "/O2".to_string()));
            additional_defines.push((
                "CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS".to_string(),
                "ON".to_string(),
            ));
            // Link the same C runtime as the Rust code these libraries end up
            // in: static with `-C target-feature=+crt-static`, dynamic
            // otherwise. rustc always uses the release runtime, so there is
            // no debug variant to select.
            let runtime_library = if crt_static() {
                "MultiThreaded"
            } else {
                "MultiThreadedDLL"
            };
            additional_defines.push((
                "CMAKE_MSVC_RUNTIME_LIBRARY".to_string(),
                runtime_library.to_string(),
            ));
            // CMAKE_MSVC_RUNTIME_LIBRARY is ignored unless policy CMP0091 is
            // NEW. Tesseract enables it itself, but Leptonica's minimum CMake
            // version predates the policy, so opt in explicitly.
            additional_defines.push((
                "CMAKE_POLICY_DEFAULT_CMP0091".to_string(),
                "NEW".to_string(),
            ));
        }

        // Common flags and defines for all platforms
        cmake_cxx_flags.push_str("-DUSE_STD_NAMESPACE ");
        // Leptonica and Tesseract install through GNUInstallDirs, which picks
        // lib64 on Fedora, RHEL and similar distributions. Everything below
        // looks for the libraries in <prefix>/lib, so pin the directory.
        additional_defines.push(("CMAKE_INSTALL_LIBDIR".to_string(), "lib".to_string()));
        additional_defines.push((
            "CMAKE_POSITION_INDEPENDENT_CODE".to_string(),
            "ON".to_string(),
        ));

        (cmake_cxx_flags, additional_defines)
    }

    fn set_os_specific_link_flags() {
        if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-lib=c++");
        } else if cfg!(target_os = "linux") {
            if cfg!(target_env = "musl")
                || env::var("CC")
                    .map(|cc| cc.contains("clang"))
                    .unwrap_or(false)
            {
                println!("cargo:rustc-link-lib=c++");
            } else {
                println!("cargo:rustc-link-lib=stdc++");
            }
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=m");
            println!("cargo:rustc-link-lib=dl");
        } else if cfg!(target_os = "freebsd") {
            println!("cargo:rustc-link-lib=c++");
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=m");
        } else if cfg!(target_os = "windows") {
            // Additional linker flags are generally not required for Windows,
            // as MSVC automatically links the necessary libraries.
            // However, for some special cases, additions can be made as follows:
            // println!("cargo:rustc-link-lib=user32");
            // println!("cargo:rustc-link-lib=gdi32");
        }

        println!(
            "cargo:rustc-link-search=native={}",
            env::var("OUT_DIR").unwrap()
        );
    }

    fn download_and_extract(target_dir: &Path, url: &str, name: &str) -> PathBuf {
        use reqwest::blocking::Client;
        use zip::ZipArchive;

        fs::create_dir_all(target_dir).expect("Failed to create target directory");

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        println!("cargo:warning=Downloading {} from {}", name, url);
        let mut response = client.get(url).send().expect("Failed to download archive");

        if !response.status().is_success() {
            panic!("Failed to download {}: HTTP {}", name, response.status());
        }

        let mut content = Vec::new();
        response
            .copy_to(&mut content)
            .expect("Failed to read archive content");

        println!(
            "cargo:warning=Downloaded {} bytes for {}",
            content.len(),
            name
        );

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
            // enclosed_name() returns None for absolute paths and for entries
            // that would resolve outside the extraction directory.
            let file_path = file.enclosed_name().unwrap_or_else(|| {
                panic!(
                    "Refusing to extract unsafe path {:?} from the {} archive",
                    String::from_utf8_lossy(file.name_raw()),
                    name
                )
            });

            // Skip the top-level directory
            let mut components = file_path.components();
            components.next();
            let path = components.as_path();

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

    /// Fetch `<lang>.traineddata` from tessdata_best into the shared tessdata
    /// directory for every language that is not already there.
    pub(crate) fn download_tessdata<S: AsRef<str>>(project_dir: &Path, languages: &[S]) {
        let tessdata_dir = project_dir.join("tessdata");
        fs::create_dir_all(&tessdata_dir).expect("Failed to create Tessdata directory");

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        for lang in languages {
            let lang = lang.as_ref();
            let filename = format!("{}.traineddata", lang);
            let file_path = tessdata_dir.join(&filename);

            if file_path.exists() {
                println!(
                    "cargo:warning={} already exists, skipping download",
                    filename
                );
                continue;
            }

            let url = format!("{}{}", TESSDATA_BASE_URL, filename);
            let mut response = client
                .get(&url)
                .send()
                .unwrap_or_else(|e| panic!("Failed to download {}: {}", filename, e));

            if response.status() == reqwest::StatusCode::NOT_FOUND {
                panic!(
                    "No traineddata found for language '{}' ({}). \
                     See https://github.com/tesseract-ocr/tessdata_best for the available codes.",
                    lang, url
                );
            }
            if !response.status().is_success() {
                panic!(
                    "Failed to download {}: HTTP {}",
                    filename,
                    response.status()
                );
            }

            // Download under a temporary name and rename once complete, so an
            // interrupted build never leaves a truncated file that later
            // builds would treat as already present. The name is unique per
            // process because the directory is shared: another build (a
            // second project, rust-analyzer) may be fetching the same file.
            let partial_path =
                tessdata_dir.join(format!("{}.{}.part", filename, std::process::id()));
            let mut dest = fs::File::create(&partial_path).expect("Failed to create file");
            if let Err(e) = response.copy_to(&mut dest) {
                drop(dest);
                let _ = fs::remove_file(&partial_path);
                panic!("Failed to write {}: {}", filename, e);
            }
            drop(dest);

            if let Err(e) = fs::rename(&partial_path, &file_path) {
                let _ = fs::remove_file(&partial_path);
                // Losing the race to a concurrent build is fine as long as
                // the file is there now.
                if !file_path.exists() {
                    panic!("Failed to move {} into place: {}", filename, e);
                }
            }
            println!("cargo:warning={} downloaded", filename);
        }
    }

    /// Apply `patch` to a bundled source file, writing it back only when that
    /// changes its contents. The source tree is watched with
    /// `rerun-if-changed`, so rewriting a file that is already patched would
    /// make cargo run the build script again on the next build.
    fn patch_source(path: &Path, patch: impl FnOnce(&str) -> String) {
        let original = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
        let patched = patch(&original);
        if patched != original {
            fs::write(path, patched)
                .unwrap_or_else(|e| panic!("Failed to write {}: {}", path.display(), e));
        }
    }

    /// Whether rustc links the static C runtime for this target
    /// (`-C target-feature=+crt-static`).
    fn crt_static() -> bool {
        env::var("CARGO_CFG_TARGET_FEATURE")
            .map(|features| features.split(',').any(|f| f == "crt-static"))
            .unwrap_or(false)
    }

    /// Directory name for the artifacts of the current target, e.g.
    /// `aarch64-apple-darwin` or `x86_64-pc-windows-msvc-crt-static`.
    fn target_dir_name() -> String {
        let mut name = env::var("TARGET").expect("TARGET not set");
        // The C runtime only changes the compiled libraries on MSVC.
        let msvc = env::var("CARGO_CFG_TARGET_ENV").is_ok_and(|env| env == "msvc");
        if msvc && crt_static() {
            name.push_str("-crt-static");
        }
        name
    }

    /// Remove the cached libraries and install trees of the current target so
    /// they are rebuilt from scratch.
    fn clean_build_dir(build_dir: &Path) {
        println!("Cleaning build directory: {:?}", build_dir);
        if build_dir.exists() {
            fs::remove_dir_all(build_dir).expect("Failed to remove build directory");
        }
    }

    fn build_or_use_cached<F>(name: &str, cache_dir: &Path, install_dir: &Path, build_fn: F)
    where
        F: FnOnce(),
    {
        // Expected library name for caching
        let lib_name = if cfg!(target_os = "windows") {
            format!("{}.lib", name)
        } else {
            format!("lib{}.a", name)
        };

        let cached_path = cache_dir.join(&lib_name);
        let out_path = install_dir.join("lib").join(&lib_name);

        // For Windows, check multiple possible library names
        let possible_lib_names: Vec<String> = if cfg!(target_os = "windows") {
            match name {
                // MSVC debug builds append a "d" suffix to the library name, so
                // list both release and debug variants (see issue #17).
                "leptonica" => vec![
                    "leptonica.lib".to_string(),
                    "libleptonica.lib".to_string(),
                    "leptonica-static.lib".to_string(),
                    "leptonica-1.87.0.lib".to_string(),
                    "leptonica-1.87.0d.lib".to_string(),
                    "leptonicad.lib".to_string(),
                ],
                "tesseract" => vec![
                    "tesseract.lib".to_string(),
                    "libtesseract.lib".to_string(),
                    "tesseract-static.lib".to_string(),
                    "tesseract55.lib".to_string(),
                    "tesseract54.lib".to_string(),
                    "tesseract53.lib".to_string(),
                    "tesseract55d.lib".to_string(),
                    "tesseract54d.lib".to_string(),
                    "tesseract53d.lib".to_string(),
                ],
                _ => vec![format!("{}.lib", name)],
            }
        } else {
            vec![format!("lib{}.a", name)]
        };

        fs::create_dir_all(cache_dir).expect("Failed to create cache directory");
        fs::create_dir_all(out_path.parent().unwrap()).expect("Failed to create output directory");

        if cached_path.exists() {
            println!("Using cached {} library", name);
            if let Err(e) = fs::copy(&cached_path, &out_path) {
                println!("cargo:warning=Failed to copy cached library: {}", e);
                // If cache copy fails, rebuild
                build_fn();
            }
        } else {
            println!("Building {} library", name);
            build_fn();

            // Look for the library with various possible names
            let mut found_lib_path = None;
            for lib_name in &possible_lib_names {
                let lib_path = install_dir.join("lib").join(lib_name);
                if lib_path.exists() {
                    println!(
                        "cargo:warning=Found {} library at: {}",
                        name,
                        lib_path.display()
                    );
                    found_lib_path = Some(lib_path);
                    break;
                }
            }

            if let Some(lib_path) = found_lib_path {
                // Copy to expected location for caching
                if !out_path.exists() {
                    if let Err(e) = fs::copy(&lib_path, &out_path) {
                        println!(
                            "cargo:warning=Failed to copy library to standard location: {}",
                            e
                        );
                    }
                }
                // Cache the library
                if let Err(e) = fs::copy(&lib_path, &cached_path) {
                    println!("cargo:warning=Failed to cache library: {}", e);
                }
            } else {
                println!(
                    "cargo:warning=Library {} not found! Searched for: {:?}",
                    name, possible_lib_names
                );
                println!(
                    "cargo:warning=In directory: {}",
                    install_dir.join("lib").display()
                );
                // List files in lib directory for debugging
                if let Ok(entries) = fs::read_dir(install_dir.join("lib")) {
                    println!("cargo:warning=Files in lib directory:");
                    for entry in entries.flatten() {
                        println!("cargo:warning=  - {}", entry.file_name().to_string_lossy());
                    }
                }
            }
        }

        println!(
            "cargo:rustc-link-search=native={}",
            install_dir.join("lib").display()
        );

        // The actual built library (any of possible_lib_names, including the
        // MSVC debug "d"-suffixed variant) is copied to the generic
        // "{name}.lib" / "lib{name}.a" above, so a single generic link works
        // for both debug and release without hardcoding a versioned name.
        println!("cargo:rustc-link-lib=static={}", name);
    }
}

#[cfg(feature = "use-system-tesseract")]
mod system_tesseract {
    use std::env;

    /// Link against a system-installed Tesseract (Homebrew, libtesseract-dev,
    /// etc.) instead of compiling the bundled sources.
    ///
    /// Uses pkg-config to locate `tesseract.pc`; leptonica is pulled in
    /// transitively via the `Requires:` entry. libtesseract is C++, so the
    /// C++ standard library runtime is linked explicitly.
    pub fn link_system_tesseract() {
        pkg_config::Config::new()
            .probe("tesseract")
            .expect("use-system-tesseract requires the Tesseract development library (e.g. `brew install tesseract` or `apt install libtesseract-dev`)");

        if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-lib=c++");
        } else if cfg!(target_os = "freebsd") {
            println!("cargo:rustc-link-lib=c++");
            println!("cargo:rustc-link-lib=pthread");
        } else if cfg!(target_os = "linux") {
            let uses_clang = env::var("CC")
                .map(|cc| cc.contains("clang"))
                .unwrap_or(false);
            if uses_clang || cfg!(target_env = "musl") {
                println!("cargo:rustc-link-lib=c++");
            } else {
                println!("cargo:rustc-link-lib=stdc++");
            }
            println!("cargo:rustc-link-lib=pthread");
        }
    }
}

fn main() {
    #[cfg(feature = "use-system-tesseract")]
    system_tesseract::link_system_tesseract();

    #[cfg(all(feature = "build-tesseract", not(feature = "use-system-tesseract")))]
    build_tesseract::build();

    #[cfg(feature = "embed-tessdata")]
    {
        let languages = embed_languages();

        // The bundled build only fetches the default languages, and system
        // mode fetches nothing, so make sure everything that was requested
        // for embedding is present.
        build_tesseract::download_tessdata(&build_tesseract::get_custom_out_dir(), &languages);

        generate_embedded_tessdata(&languages);
    }
}

/// Languages to embed, taken from `TESSERACT_EMBED_LANGUAGES` (comma
/// separated). Falls back to the default set when the variable is unset or
/// empty.
#[cfg(feature = "embed-tessdata")]
fn embed_languages() -> Vec<String> {
    println!("cargo:rerun-if-env-changed=TESSERACT_EMBED_LANGUAGES");

    let requested = std::env::var("TESSERACT_EMBED_LANGUAGES").unwrap_or_default();

    let mut languages: Vec<String> = Vec::new();
    for lang in requested.split(',').map(str::trim) {
        if lang.is_empty() {
            continue;
        }
        // The code ends up in a file name, a download URL and a generated
        // string literal, so rule out path separators, quotes and the like.
        if !lang
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        {
            panic!(
                "Invalid language code {:?} in TESSERACT_EMBED_LANGUAGES \
                 (expected codes such as \"eng\" or \"chi_sim\")",
                lang
            );
        }
        if !languages.iter().any(|l| l == lang) {
            languages.push(lang.to_string());
        }
    }

    if languages.is_empty() {
        languages.extend(
            build_tesseract::DEFAULT_LANGUAGES
                .iter()
                .map(|l| l.to_string()),
        );
    }

    languages
}

#[cfg(feature = "embed-tessdata")]
fn generate_embedded_tessdata(languages: &[String]) {
    use std::fs;
    use std::path::Path;

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let tessdata_dir = build_tesseract::get_custom_out_dir().join("tessdata");

    let mut embedded_code = String::new();
    embedded_code.push_str("// Auto-generated embedded tessdata\n");
    embedded_code.push_str("use std::collections::HashMap;\n\n");
    embedded_code.push_str("pub struct EmbeddedTessdata {\n");
    embedded_code.push_str("    data: HashMap<&'static str, &'static [u8]>,\n");
    embedded_code.push_str("}\n\n");
    embedded_code.push_str("impl EmbeddedTessdata {\n");
    embedded_code.push_str("    pub fn new() -> Self {\n");
    embedded_code.push_str("        let mut data = HashMap::new();\n");

    for lang in languages {
        let traineddata_file = tessdata_dir.join(format!("{}.traineddata", lang));

        // Copy the file to OUT_DIR so include_bytes! can find it
        let dest = Path::new(&out_dir).join(format!("{}.traineddata", lang));
        fs::copy(&traineddata_file, &dest).unwrap_or_else(|e| {
            panic!(
                "Failed to copy {} to {}: {}",
                traineddata_file.display(),
                dest.display(),
                e
            )
        });

        embedded_code.push_str(&format!(
            "        data.insert(\"{}\", include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{}.traineddata\")) as &'static [u8]);\n",
            lang, lang
        ));
    }

    embedded_code.push_str("        Self { data }\n");
    embedded_code.push_str("    }\n\n");
    embedded_code.push_str("    pub fn get(&self, language: &str) -> Option<&'static [u8]> {\n");
    embedded_code.push_str("        self.data.get(language).copied()\n");
    embedded_code.push_str("    }\n\n");
    embedded_code.push_str("    pub fn available_languages(&self) -> Vec<&'static str> {\n");
    embedded_code.push_str("        self.data.keys().copied().collect()\n");
    embedded_code.push_str("    }\n");
    embedded_code.push_str("}\n\n");
    embedded_code.push_str("pub static EMBEDDED_TESSDATA: std::sync::LazyLock<EmbeddedTessdata> = std::sync::LazyLock::new(|| EmbeddedTessdata::new());\n");

    let embedded_file = Path::new(&out_dir).join("embedded_tessdata.rs");
    fs::write(&embedded_file, embedded_code).expect("Failed to write embedded tessdata file");

    println!("cargo:rerun-if-changed={}", tessdata_dir.display());
}
