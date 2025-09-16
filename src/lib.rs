pub mod formatter;
pub mod reorder;

pub use formatter::format_gdscript;

pub struct FormatterConfig {
    pub indent_size: usize,
    pub use_spaces: bool,
    pub reorder_code: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            use_spaces: false,
            reorder_code: false,
        }
    }
}
