use crate::linter::rules::ALL_RULES;
use std::collections::HashSet;

/// Parse disabled rules from command line arguments or configuration
pub fn parse_disabled_rules(rules_string: &str) -> HashSet<String> {
    rules_string
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Get all available rule names
pub fn get_all_rule_names() -> Vec<&'static str> {
    ALL_RULES.iter().map(|rule| rule.name).collect()
}

/// Validate that all provided rule names are valid
pub fn validate_rule_names(rules: &HashSet<String>) -> Result<(), Vec<String>> {
    let valid_rules: HashSet<&str> = get_all_rule_names().into_iter().collect();
    let invalid_rules: Vec<String> = rules
        .iter()
        .filter(|rule| !valid_rules.contains(rule.as_str()))
        .cloned()
        .collect();

    if invalid_rules.is_empty() {
        Ok(())
    } else {
        Err(invalid_rules)
    }
}
