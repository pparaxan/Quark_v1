use crowsa::{Crowsa, CrowsaConfig, CrowsaError};

#[cfg(test)]
mod crowsa_lib {
    use super::*;
    use std::path::Path;

    const TEST_PATH: &str = "examples/global_html";

    fn assert_html_exists() {
        let html_path = Path::new(TEST_PATH).join("index.html");
        assert!(html_path.exists(), "index.html should exist in {}", TEST_PATH);
    }

    #[test]
    fn test_valid_initialization() {
        assert_html_exists();

        let config = CrowsaConfig {
            content_path: TEST_PATH.to_string(),
            ..Default::default()
        };

        let result = Crowsa::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_path() {
        let config = CrowsaConfig {
            content_path: "invalid".to_string(),
            ..Default::default()
        };

        let result = Crowsa::new(config);
        assert!(matches!(result, Err(CrowsaError::PathError)));
    }

    #[test]
    fn test_binding() {
        assert_html_exists();

        let config = CrowsaConfig {
            content_path: TEST_PATH.to_string(),
            ..Default::default()
        };

        let mut crowsa = Crowsa::new(config).expect("Failed to create an instance");
        crowsa.bind("test_function", |_, _| {});
    }

    #[test]
    fn test_eval() {
        assert_html_exists();

        let config = CrowsaConfig {
            content_path: TEST_PATH.to_string(),
            ..Default::default()
        };

        let mut crowsa = Crowsa::new(config).expect("Failed to create Crowsa instance");
        crowsa.eval("console.log('test');");
    }

    #[test]
    fn test_custom_window_config() {
        assert_html_exists();

        let config = CrowsaConfig {
            content_path: TEST_PATH.to_string(),
            window_title: "Test Window".to_string(),
            width: 1024,
            height: 768,
            debug: false,
            resizable: false,
        };

        let result = Crowsa::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_resolution() {
        assert_html_exists();

        let current_dir = std::env::current_dir().unwrap();
        let full_path = current_dir.join(TEST_PATH).join("index.html");
        assert!(full_path.exists());
    }

    #[test]
    fn test_error_debug() {
        let errors = [
            CrowsaError::InitializationFailed,
            CrowsaError::PathError,
            CrowsaError::WebviewError,
        ];

        for error in &errors {
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_multiple_bindings() {
        assert_html_exists();

        let config = CrowsaConfig {
            content_path: TEST_PATH.to_string(),
            ..Default::default()
        };

        let mut crowsa = Crowsa::new(config).expect("Failed to create Crowsa instance");

        crowsa.bind("function1", |_, _| {});
        crowsa.bind("function2", |_, _| {});
        crowsa.bind("function3", |_, _| {});
    }

    #[test]
    fn test_multiple_evals() {
        assert_html_exists();

        let config = CrowsaConfig {
            content_path: TEST_PATH.to_string(),
            ..Default::default()
        };

        let mut crowsa = Crowsa::new(config).expect("Failed to create Crowsa instance");

        // Test that multiple evals don't panic
        crowsa.eval("console.log('test1');");
        crowsa.eval("console.log('test2');");
        crowsa.eval("console.log('test3');");
    }
}
