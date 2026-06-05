pub mod formatter;
pub mod linter;
pub mod reorder;

#[derive(Clone)]
pub struct FormatterConfig {
    pub indent_size: usize,
    pub use_spaces: bool,
    pub reorder_code: bool,
    pub safe: bool,
    pub preserve_trailing_whitespace: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            use_spaces: false,
            reorder_code: false,
            safe: false,
            preserve_trailing_whitespace: false,
        }
    }
}
