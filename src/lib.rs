use proc_macro::TokenStream;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crate::css::StylesheetParser;

mod css;

// TODO: GET RID OF THIS!!
static mut GENERATED_STYLES: Option<HashMap<String, String>> = None;

#[cfg_attr(not(test), proc_macro)]
pub fn style(tokens: TokenStream) -> TokenStream {
    let style = tokens.to_string();
    let mut hasher = DefaultHasher::new();
    style.hash(&mut hasher);
    let scope_class = format!("un-{}", hasher.finish());
    let scope_class_lit = format!(r#""{scope_class}""#);
    let mut parser = StylesheetParser::default();
    parser.parse_stylesheet(style);

    let style = parser.stylesheet.compile(&scope_class);
    write_to_file(&style);

    unsafe {
        if GENERATED_STYLES.is_none() {
            GENERATED_STYLES = Some(HashMap::new());
        }

        if let Some(styles) = &mut GENERATED_STYLES {
            styles.insert(scope_class, style);
        }
    };

    TokenStream::from_str(&scope_class_lit).expect("Can return scope_class")
}

fn write_to_file(css: &str) {
    let output = std::env::current_dir().unwrap().join("target").join("unstyled.css");
    std::fs::write(output, css).expect("Could not write data to unstyled.css");
}

#[cfg(test)]
mod test {
    use crate::css::StylesheetParser;

    #[test]
    pub fn test_simple_class() {
        let css = ".test {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(compiled, ".test.random_test_class {display: block;}");
    }

    #[test]
    pub fn test_simple_id() {
        let css = "#test {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(compiled, "#test.random_test_class {display: block;}");
    }

    #[test]
    pub fn test_simple_attr() {
        let css = "[test] {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(compiled, "[test].random_test_class {display: block;}");
    }

    #[test]
    pub fn test_simple_pseudo() {
        let css = ":not(.test) {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(compiled, ".random_test_class:not(.test) {display: block;}");
    }

    #[test]
    pub fn test_simple_tag() {
        let css = "test {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(compiled, "test.random_test_class {display: block;}");
    }

    #[test]
    pub fn test_mixed_selectors() {
        let css = "test.test-class#test-id[data-test] {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            "test.test-class#test-id[data-test].random_test_class {display: block;}"
        );
    }

    #[test]
    pub fn test_combined_descendent() {
        let css = ".test-class .sub-class {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            ".test-class.random_test_class .sub-class.random_test_class {display: block;}"
        );
    }

    #[test]
    pub fn test_combined_child() {
        let css = ".test-class > .sub-class {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            ".test-class.random_test_class>.sub-class.random_test_class {display: block;}"
        );
    }

    #[test]
    pub fn test_combined_namespace() {
        let css = ".test-class | .sub-class {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            ".test-class.random_test_class|.sub-class.random_test_class {display: block;}"
        );
    }

    #[test]
    pub fn test_combined_sibling() {
        let css = ".test-class + .sub-class {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            ".test-class.random_test_class+.sub-class.random_test_class {display: block;}"
        );
    }

    #[test]
    pub fn test_combined_generic() {
        let css = ".test-class ~ .sub-class {display: block;}".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            ".test-class.random_test_class~.sub-class.random_test_class {display: block;}"
        );
    }

    #[test]
    pub fn test_at_keyframes() {
        let css =
            "@keyframes test { 0% { background: red; } 100% { background: blue; } }".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            "@keyframes test { 0% { background: red; } 100% { background: blue; } }"
        );
    }

    #[test]
    pub fn test_at_import() {
        let css = "@import 'test';".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(compiled, "@import 'test';");
    }

    #[test]
    pub fn test_at_page() {
        let css = "@page { margin: 1210000000em; }".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(compiled, "@page { margin: 1210000000em; }");
    }

    #[test]
    pub fn test_at_media() {
        let css = "@media (min-width: 120em) { .my-element { margin: 1210000000em; } }".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            "@media (min-width: 120em) {.my-element.random_test_class { margin: 1210000000em; }}"
        );
    }

    #[test]
    pub fn test_at_layer() {
        let css = "@layer my_fancy_layer { .my-element { margin: 1210000000em; } }".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            "@layer my_fancy_layer {.my-element.random_test_class { margin: 1210000000em; }}"
        );
    }
}
