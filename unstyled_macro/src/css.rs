use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::Chars;

const SELECTOR_END: &str = ".#[{:";

#[derive(Debug, Default)]
struct AtRuleWithSelectors {
    at_rule: String,
    blocks: Vec<NormalBlock>,
}

#[derive(Debug)]
struct NormalBlock {
    selector: Vec<Combinator>,
    content: String,
}

#[derive(Debug)]
enum StyleBlock {
    Normal(NormalBlock),
    /// Used for @rules like @media or @supports, which again contains selectors
    AtRuleWithSelectors(AtRuleWithSelectors),
    /// Used for generic/other @rules like @keyframes, @page, @import etc.
    GenericAtRule(String),
}

impl StyleBlock {
    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal(_))
    }
}

#[derive(Default, Debug)]
pub(crate) struct Stylesheet {
    blocks: Vec<StyleBlock>,
}

impl AtRuleWithSelectors {
    pub fn compile(&self, scope_class: &str) -> String {
        let mut output = String::new();

        output.push_str(&self.at_rule);
        output.push('{');

        let block = self
            .blocks
            .iter()
            .map(|block| block.compile(scope_class))
            .collect::<String>();

        output.push_str(&block);
        output.push('}');

        output
    }
}

impl NormalBlock {
    pub fn compile(&self, scope_class: &str) -> String {
        let mut output = String::new();

        for combinator in &self.selector {
            output.push_str(&combinator.compile(scope_class));
        }

        output.push_str(&self.content);

        output
    }
}

impl StyleBlock {
    pub fn compile(&self, scope_class: &str) -> String {
        let mut output = String::new();

        match self {
            StyleBlock::AtRuleWithSelectors(at_rule) => {
                output.push_str(&at_rule.compile(scope_class));
            }
            StyleBlock::Normal(block) => {
                output.push_str(&block.compile(scope_class));
            }
            StyleBlock::GenericAtRule(content) => {
                output.push_str(content);
            }
        }

        output
    }
}

impl Stylesheet {
    pub fn compile(&self, scope_class: &str) -> String {
        let mut output = String::new();

        for block in &self.blocks {
            output.push_str(&block.compile(scope_class));
        }

        output
    }
}

#[derive(Clone, Debug)]
enum Combinator {
    Sibling(Selector),
    Child(Selector),
    General(Selector),
    Namespace(Selector),
    Descendant(Selector),
    Combine(Selector),
}

const PSEUDO_ELEMENTS: &[&str] = &[
    "after",
    "backdrop",
    "before",
    "cue",
    "cue-region",
    "first-letter",
    "first-line",
    "file-selector-button",
    "grammar-error",
    "marker",
    "part",
    "placeholder",
    "selection",
    "slotted",
    "spelling-error",
    "target-text",
];

fn apply_scope_class(scope_class: &str, combinator: &str, selector: &Selector) -> String {
    match selector {
        Selector::Pseudo(pseudo) => {
            if pseudo.starts_with("deep") {
                let chars = pseudo.chars().skip_while(|char| *char != '(').skip(1);
                let mut in_braces = 1;
                let mut selector = String::new();
                for char in chars {
                    if char == '(' {
                        in_braces += 1;
                    }

                    if char == ')' {
                        in_braces -= 1;

                        if in_braces == 0 {
                            return format!("{selector}{combinator}");
                        }
                    }

                    selector.push(char);
                }

            }


            format!(".{scope_class}{selector}{combinator}")
        },
        selector => format!("{selector}.{scope_class}{combinator}"),
    }
}

impl Combinator {
    fn compile(&self, scope_class: &str) -> String {
        match self {
            Combinator::Sibling(selector) => apply_scope_class(scope_class, "+", selector),
            Combinator::Child(selector) => apply_scope_class(scope_class, ">", selector),
            Combinator::General(selector) => apply_scope_class(scope_class, "~", selector),
            Combinator::Namespace(selector) => apply_scope_class(scope_class, "|", selector),
            Combinator::Descendant(selector) => apply_scope_class(scope_class, " ", selector),
            Combinator::Combine(selector) => format!("{selector}"),
        }
    }
}

#[derive(Clone, Debug)]
enum Selector {
    Tag(String),
    Class(String),
    Id(String),
    Attribute(String),
    Pseudo(String),
}

impl Selector {
    fn push(&mut self, char: char) {
        match self {
            Selector::Tag(content) => {
                content.push(char);
            }
            Selector::Class(content) => {
                content.push(char);
            }
            Selector::Id(content) => {
                content.push(char);
            }
            Selector::Attribute(content) => {
                content.push(char);
            }
            Selector::Pseudo(content) => {
                content.push(char);
            }
        }
    }

    fn is_attribute(&self) -> bool {
        matches!(self, Self::Attribute(_))
    }
}

impl Display for Selector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Selector::Tag(selector) => f.write_str(selector),
            Selector::Class(selector) => f.write_fmt(format_args!(".{selector}")),
            Selector::Id(selector) => f.write_fmt(format_args!("#{selector}")),
            Selector::Attribute(selector) => f.write_fmt(format_args!("[{selector}]")),
            Selector::Pseudo(selector) => {
                let mut pseudo_colon = ":";
                for element in PSEUDO_ELEMENTS {
                    if selector.starts_with(element) {
                        pseudo_colon = "::";
                        break;
                    }
                }

                f.write_fmt(format_args!("{pseudo_colon}{selector}"))
            },
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct StylesheetParser {
    pub stylesheet: Stylesheet,
    current_selector: Vec<Combinator>,
}

impl StylesheetParser {
    pub fn compress_combinator(&self, css: String, combinator: &str) -> String {
        css.replace(&[" ", combinator, " "].join(""), combinator)
            .replace(&[combinator, " "].join(""), combinator)
            .replace(&[" ", combinator].join(""), combinator)
    }

    pub fn parse_stylesheet(&mut self, css: String) {
        let css = css
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace("::", ":");
        let css = self.compress_combinator(css, ">");
        let css = self.compress_combinator(css, "+");
        let css = self.compress_combinator(css, "|");
        let css = self.compress_combinator(css, "~");
        let mut char_iter = css.chars().peekable();
        let mut in_block = 0;
        let mut block_tokens = String::new();

        loop {
            let Some(char) = char_iter.peek().copied() else {
                break;
            };

            if char == '{' {
                in_block += 1;
            }

            if char == '}' {
                in_block -= 1;

                if in_block == 0 {
                    block_tokens.push(char);

                    let selector = std::mem::take(&mut self.current_selector);

                    let block = NormalBlock {
                        selector,
                        content: block_tokens,
                    };

                    self.stylesheet.blocks.push(StyleBlock::Normal(block));

                    block_tokens = String::new();
                }
            }

            if in_block > 0 {
                block_tokens.push(char);
                char_iter.next();

                continue;
            }

            if ".#:[]_".contains(char) || char.is_alphabetic() {
                self.parse_selector(&mut char_iter);

                continue;
            }

            if char == '@' {
                self.parse_at_rule(&mut char_iter);

                continue;
            }

            char_iter.next();
        }
    }

    pub fn parse_selector(&mut self, char_iter: &mut Peekable<Chars>) {
        let mut current_token = None;
        let mut in_braces = 0;

        loop {
            let Some(char) = char_iter.peek().copied() else {
                break;
            };

            if current_token.is_none() {
                if char == '.' {
                    char_iter.next();
                    current_token = Some(Selector::Class(String::new()));

                    continue;
                }

                if char == '#' {
                    char_iter.next();
                    current_token = Some(Selector::Id(String::new()));

                    continue;
                }

                if char == ':' {
                    char_iter.next();
                    current_token = Some(Selector::Pseudo(String::new()));

                    continue;
                }

                if char == '[' {
                    char_iter.next();
                    current_token = Some(Selector::Attribute(String::new()));

                    continue;
                }

                if char.is_alphabetic() {
                    char_iter.next();
                    current_token = Some(Selector::Tag(String::from(char)));

                    continue;
                }
            }

            if char == '(' {
                in_braces += 1;
            }

            if char == ')' {
                in_braces -= 1;
            }

            if SELECTOR_END.contains(char) && in_braces == 0 {
                if let Some(current_token) = current_token.take() {
                    self.current_selector
                        .push(Combinator::Combine(current_token))
                }

                break;
            }

            if char == '+' && in_braces == 0 {
                if let Some(current_token) = current_token.take() {
                    self.current_selector
                        .push(Combinator::Sibling(current_token))
                }

                break;
            }

            if char == '~' && in_braces == 0 {
                if let Some(current_token) = current_token.take() {
                    self.current_selector
                        .push(Combinator::General(current_token))
                }

                break;
            }

            if char == '|' && in_braces == 0 {
                if let Some(current_token) = current_token.take() {
                    self.current_selector
                        .push(Combinator::Namespace(current_token))
                }

                break;
            }

            if char == '>' && in_braces == 0 {
                if let Some(current_token) = current_token.take() {
                    self.current_selector.push(Combinator::Child(current_token))
                }

                break;
            }

            if char == ' ' && in_braces == 0 {
                if let Some(current_token) = current_token.take() {
                    self.current_selector
                        .push(Combinator::Descendant(current_token))
                }

                break;
            }

            if char == ']'
                && current_token
                .as_ref()
                .map(|token| token.is_attribute())
                .unwrap_or_default()
            {
                char_iter.next();

                continue;
            }

            if let Some(current_token) = &mut current_token {
                current_token.push(char);
                char_iter.next();
            }
        }
    }

    pub fn parse_at_rule(&mut self, char_iter: &mut Peekable<Chars>) {
        let mut raw_rule = String::new();
        let mut in_block = 0;
        let rule_name = char_iter
            .clone()
            .take_while(|char| char.is_alphanumeric() || "-@_".contains(*char))
            .collect::<String>();

        if ["@media", "@layer", "@supports", "@container"].contains(&&*rule_name) {
            self.parse_at_rule_with_selectors(char_iter);

            return;
        }

        loop {
            let Some(char) = char_iter.peek().copied() else {
                break;
            };

            if char == '{' {
                in_block += 1;
            }

            if char == '}' {
                in_block -= 1;

                if in_block == 0 {
                    raw_rule.push(char);
                    self.stylesheet
                        .blocks
                        .push(StyleBlock::GenericAtRule(raw_rule));
                    char_iter.next();

                    break;
                }
            }

            if char == ';' && in_block == 0 {
                raw_rule.push(char);
                self.stylesheet
                    .blocks
                    .push(StyleBlock::GenericAtRule(raw_rule));
                char_iter.next();

                break;
            }

            raw_rule.push(char);
            char_iter.next();
        }
    }

    pub fn parse_at_rule_with_selectors(&mut self, char_iter: &mut Peekable<Chars>) {
        let mut at_rule = String::new();
        let mut block_content = String::new();
        let mut in_block = 0;

        loop {
            let Some(char) = char_iter.peek().copied() else {
                break;
            };

            if char == '{' {
                in_block += 1;
                char_iter.next();

                if in_block > 1 {
                    block_content.push(char);
                }

                continue;
            }

            if char == '}' {
                in_block -= 1;

                if in_block == 0 {
                    let mut parser = StylesheetParser::default();
                    parser.parse_stylesheet(block_content);

                    let blocks = parser
                        .stylesheet
                        .blocks
                        .into_iter()
                        .filter(|block| block.is_normal())
                        .map(|block| {
                            match block {
                                StyleBlock::Normal(normal) => Some(normal),
                                StyleBlock::AtRuleWithSelectors(_) => None,
                                StyleBlock::GenericAtRule(_) => None,
                            }
                                .unwrap()
                        })
                        .collect::<Vec<_>>();
                    let at_rule = AtRuleWithSelectors { at_rule, blocks };

                    self.stylesheet
                        .blocks
                        .push(StyleBlock::AtRuleWithSelectors(at_rule));
                    char_iter.next();

                    break;
                }
            }

            if in_block == 0 {
                at_rule.push(char);
                char_iter.next();

                continue;
            }

            block_content.push(char);
            char_iter.next();
        }
    }
}
