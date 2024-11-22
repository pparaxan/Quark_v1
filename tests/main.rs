use crowsa::prelude::*;

#[cfg(test)]
mod crowsa_lib {
    use super::*;
    use std::{path::Path, thread::sleep};

    const BASIC_EXAMPLE: &str = "examples/frontend/basic";

    fn assert_html_exists() {
        let html_path = Path::new(BASIC_EXAMPLE).join("index.html");
        assert!(
            html_path.exists(),
            "index.html should exist in {}",
            BASIC_EXAMPLE
        );
    }

    // #[test]
    // fn test_valid_initialization() {
    //     assert_html_exists();

    //     let config = CrowsaConfig::new().frontend(BASIC_EXAMPLE);

    //     let result = Crowsa::new(config);
    //     assert!(result.is_ok());
    // }

    // #[test]
    // fn test_invalid_path() {
    //     let config = CrowsaConfig::new().frontend("invalid");

    //     let result = Crowsa::new(config);
    //     assert!(matches!(result, Err(CrowsaError::PathError)));
    // }

    // #[test]
    // fn test_binding() {
    //     assert_html_exists();

    //     let config = CrowsaConfig::new().frontend(BASIC_EXAMPLE);

    //     let mut crowsa = Crowsa::new(config).expect("Failed to create an instance");
    //     crowsa.bind("test_function", |_, _| {});
    // }

    // #[test]
    // fn eval() {
    //     assert_html_exists();

    //     let config = CrowsaConfig::new().frontend(BASIC_EXAMPLE);

    //     let mut crowsa = Crowsa::new(config).expect("Failed to create Crowsa instance");
    //     crowsa.eval("console.log('test');");
    // }

    #[test]
    fn custom_config() {
        assert_html_exists();

        let config = CrowsaConfig::new()
            .frontend(BASIC_EXAMPLE)
            .title("CrowsaTestWindowConfig")
            .width(1024)
            .height(768)
            .debug(false)
            .resizable(SizeHint::FIXED);

        let result = Crowsa::new(config);
        assert!(result.is_ok());
    }

    // #[test]
    // fn path() {
    //     assert_html_exists();

    //     let current_dir = std::env::current_dir().unwrap();
    //     let full_path = current_dir.join(BASIC_EXAMPLE).join("index.html");
    //     assert!(full_path.exists());
    // }

    #[test]
    fn errors() {
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

    // #[test]
    // fn test_multiple_bindings() {
    //     assert_html_exists();

    //     let config = CrowsaConfig::new().frontend(BASIC_EXAMPLE);

    //     let mut crowsa = Crowsa::new(config).expect("Failed to create Crowsa instance");

    //     crowsa.bind("function1", |_, _| {});
    //     crowsa.bind("function2", |_, _| {});
    //     crowsa.bind("function3", |_, _| {});
    // }

    // #[test]
    // fn test_multiple_evals() {
    //     assert_html_exists();

    //     let config = CrowsaConfig::new().frontend(BASIC_EXAMPLE);

    //     let mut crowsa = Crowsa::new(config).expect("Failed to create Crowsa instance");

    //     // Test that multiple evals don't panic
    //     crowsa.eval("console.log('test1');");
    //     crowsa.eval("console.log('test2');");
    //     crowsa.eval("console.log('test3');");
    // }
}
