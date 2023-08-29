#![feature(proc_macro_span)]

use proc_macro::{Span, TokenStream};
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
    let write_macro = unsafe {
        if let Some(styles) = &GENERATED_STYLES {
            if styles.is_empty() {
                "unstyled::write_style!();"
            } else { "" }
        } else { "" }
    };
    let scope_class_lit = format!(r#"{{ {write_macro} "{scope_class}"}}"#);
    let mut parser = StylesheetParser::default();
    parser.parse_stylesheet(style);

    let style = parser.stylesheet.compile(&scope_class);

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

#[cfg_attr(not(test), proc_macro)]
pub fn write_style(_: TokenStream) -> TokenStream {
    let file = Span::call_site().source_file().path().to_str().unwrap().replace("/", "_").replace(".rs", ".css");

    unsafe {
        if let Some(styles) = &GENERATED_STYLES {
            let styles = styles.values().cloned().collect::<Vec<_>>().join("\n");

            write_to_partial(&styles, &file);
        }
    }

    TokenStream::new()
}

fn write_to_partial(css: &str, outfile: &str) {
    let output_dir = std::env::current_dir().unwrap().join("target").join("unstyled");
    std::fs::create_dir_all(&output_dir).unwrap();
    std::fs::write(output_dir.join(outfile), css).expect("Could not write partial css output");
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

    #[test]
    pub fn test_pseudo_element() {
        let css = "span::before { content: '$'; display: block; }".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            "span.random_test_class::before { content: '$'; display: block; }"
        );
    }

    /// Unstyled ensures, that pseudo elements (like the `::before`) actually have the double colon
    /// (::) as defined by the [CSS Pseudo-Elements Module Level 4](https://drafts.csswg.org/css-pseudo/)
    #[test]
    pub fn test_pseudo_element_corrected() {
        let css = "span:before { content: '$'; display: block; }".to_string();
        let mut parser = StylesheetParser::default();
        parser.parse_stylesheet(css);
        let compiled = parser.stylesheet.compile("random_test_class");
        assert_eq!(
            compiled,
            "span.random_test_class::before { content: '$'; display: block; }"
        );
    }
}
