use regex::Regex;
use std::sync::LazyLock;

/// snake_case
pub static SNAKE_CASE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9_]*$").unwrap());

/// _private_snake_case
pub static PRIVATE_SNAKE_CASE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^_[a-z][a-z0-9_]*$").unwrap());

/// PascalCase
pub static PASCAL_CASE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap());

/// CONSTANT_CASE
pub static CONSTANT_CASE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap());

/// _PRIVATE_CONSTANT_CASE
pub static PRIVATE_CONSTANT_CASE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^_[A-Z][A-Z0-9_]*$").unwrap());
