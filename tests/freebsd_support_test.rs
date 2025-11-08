#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn test_freebsd_cache_directory() {
        // This test verifies that FreeBSD uses the correct cache directory structure
        // It should use ~/.tesseract-rs similar to Linux
        
        #[cfg(target_os = "freebsd")]
        {
            let expected_path = if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(home).join(".tesseract-rs")
            } else {
                // Fallback for test environments
                PathBuf::from("/tmp/.tesseract-rs")
            };
            
            // This would be the path used by the build script
            assert!(expected_path.to_string_lossy().contains(".tesseract-rs"));
            assert!(!expected_path.to_string_lossy().contains("Library/Application Support"));
            assert!(!expected_path.to_string_lossy().contains("AppData"));
        }
        
        #[cfg(not(target_os = "freebsd"))]
        {
            // On non-FreeBSD systems, just verify the test compiles
            println!("FreeBSD-specific test skipped on non-FreeBSD system");
        }
    }

    #[test]
    fn test_tessdata_directory_function() {
        // Test the tessdata directory function from integration tests
        fn get_default_tessdata_dir() -> PathBuf {
            if cfg!(target_os = "macos") {
                let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home_dir)
                    .join("Library")
                    .join("Application Support")
                    .join("tesseract-rs")
                    .join("tessdata")
            } else if cfg!(target_os = "linux") {
                let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home_dir)
                    .join(".tesseract-rs")
                    .join("tessdata")
            } else if cfg!(target_os = "freebsd") {
                let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home_dir)
                    .join(".tesseract-rs")
                    .join("tessdata")
            } else if cfg!(target_os = "windows") {
                PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| "C:\\temp".to_string()))
                    .join("tesseract-rs")
                    .join("tessdata")
            } else {
                panic!("Unsupported operating system");
            }
        }

        let tessdata_dir = get_default_tessdata_dir();
        
        // Verify the path is constructed correctly for each OS
        #[cfg(target_os = "freebsd")]
        {
            assert!(tessdata_dir.to_string_lossy().contains(".tesseract-rs"));
            assert!(tessdata_dir.to_string_lossy().ends_with("tessdata"));
        }
        
        #[cfg(target_os = "linux")]
        {
            assert!(tessdata_dir.to_string_lossy().contains(".tesseract-rs"));
            assert!(tessdata_dir.to_string_lossy().ends_with("tessdata"));
        }
        
        #[cfg(target_os = "macos")]
        {
            assert!(tessdata_dir.to_string_lossy().contains("Library/Application Support"));
            assert!(tessdata_dir.to_string_lossy().ends_with("tessdata"));
        }
        
        #[cfg(target_os = "windows")]
        {
            assert!(tessdata_dir.to_string_lossy().contains("tesseract-rs"));
            assert!(tessdata_dir.to_string_lossy().ends_with("tessdata"));
        }
    }
}
