use regex::Regex;
use std::sync::LazyLock;

/// snake_case
/// Compile-time constant regex patterns. These are guaranteed to be valid regexes.
macro_rules! static_regex {
    ($name:ident, $pattern:expr) => {
        pub static $name: LazyLock<Regex> =
            LazyLock::new(|| Regex::new($pattern).expect(concat!($pattern, " is a valid regex")));
    };
}

static_regex!(SNAKE_CASE, r"^[a-z][a-z0-9_]*$");
static_regex!(PRIVATE_SNAKE_CASE, r"^_[a-z][a-z0-9_]*$");
static_regex!(PASCAL_CASE, r"^[A-Z][a-zA-Z0-9]*$");
static_regex!(CONSTANT_CASE, r"^[A-Z][A-Z0-9_]*$");
static_regex!(PRIVATE_CONSTANT_CASE, r"^_[A-Z][A-Z0-9_]*$");
