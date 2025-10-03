use std::collections::{HashMap, HashSet};

/// Represents an ignore directive for a specific line
#[derive(Debug, Clone)]
pub struct IgnoreDirective {
    /// The line number this directive applies to
    pub target_line: usize,
    /// Set of rule names to ignore on this line
    pub ignored_rules: HashSet<String>,
}

/// Parse ignore comments from source code and return a map of line numbers to ignored rules
pub fn parse_ignore_patterns(source_code: &str) -> HashMap<usize, HashSet<String>> {
    let mut ignore_map: HashMap<usize, HashSet<String>> = HashMap::new();

    for (line_idx, line) in source_code.lines().enumerate() {
        let line_number = line_idx + 1;

        // Look for ignore comments
        if let Some(comment_start) = line.find('#') {
            let comment = &line[comment_start..];

            // Check for gdlint-ignore patterns
            if let Some(rules) = parse_ignore_comment(comment) {
                // Check if this is a "next-line" directive
                if comment.contains("gdlint-ignore-next-line") {
                    // Apply to the next line
                    let target_line = line_number + 1;
                    ignore_map.entry(target_line).or_default().extend(rules);
                } else if comment.contains("gdlint-ignore-line")
                    || comment.contains("gdlint-ignore")
                {
                    // Apply to the current line
                    ignore_map.entry(line_number).or_default().extend(rules);
                }
            }
        }
    }

    ignore_map
}

/// Parse a single ignore comment and extract the rule names
fn parse_ignore_comment(comment: &str) -> Option<HashSet<String>> {
    // Look for gdlint-ignore patterns
    let patterns = [
        "gdlint-ignore-next-line",
        "gdlint-ignore-line",
        "gdlint-ignore",
    ];

    for pattern in &patterns {
        if let Some(start_idx) = comment.find(pattern) {
            let after_pattern = &comment[start_idx + pattern.len()..];

            // Look for rule names after the pattern
            // They can be separated by spaces, commas, or both
            let rules_part = after_pattern.trim();

            if rules_part.is_empty() {
                // No specific rules mentioned, ignore all rules
                return Some(HashSet::new()); // Empty set means ignore all
            }

            // Parse comma and/or space separated rule names
            let rules: HashSet<String> = rules_part
                .split(|c: char| c == ',' || c.is_whitespace())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();

            return Some(rules);
        }
    }

    None
}

/// Check if a specific rule should be ignored for a given line
pub fn should_ignore_rule(
    ignore_map: &HashMap<usize, HashSet<String>>,
    line: usize,
    rule_name: &str,
) -> bool {
    if let Some(ignored_rules) = ignore_map.get(&line) {
        // If the set is empty, it means ignore all rules
        ignored_rules.is_empty() || ignored_rules.contains(rule_name)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ignore_next_line_single_rule() {
        let source = r#"# gdlint-ignore-next-line private-access
obj._private_method()"#;

        let ignore_map = parse_ignore_patterns(source);
        assert_eq!(ignore_map.len(), 1);

        let rules = ignore_map.get(&2).unwrap();
        assert!(rules.contains("private-access"));
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_parse_ignore_next_line_multiple_rules() {
        let source = r#"# gdlint-ignore-next-line private-access,constant-name
obj._private_method()"#;

        let ignore_map = parse_ignore_patterns(source);
        assert_eq!(ignore_map.len(), 1);

        let rules = ignore_map.get(&2).unwrap();
        assert!(rules.contains("private-access"));
        assert!(rules.contains("constant-name"));
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_parse_ignore_current_line() {
        let source = r#"obj._private_method() # gdlint-ignore private-access"#;

        let ignore_map = parse_ignore_patterns(source);
        assert_eq!(ignore_map.len(), 1);

        let rules = ignore_map.get(&1).unwrap();
        assert!(rules.contains("private-access"));
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_parse_ignore_all_rules() {
        let source = r#"# gdlint-ignore-next-line
some_problematic_code()"#;

        let ignore_map = parse_ignore_patterns(source);
        assert_eq!(ignore_map.len(), 1);

        let rules = ignore_map.get(&2).unwrap();
        assert!(rules.is_empty()); // Empty means ignore all
    }

    #[test]
    fn test_should_ignore_rule() {
        let mut ignore_map = HashMap::new();
        let mut rules = HashSet::new();
        rules.insert("private-access".to_string());
        rules.insert("constant-name".to_string());
        ignore_map.insert(5, rules);

        assert!(should_ignore_rule(&ignore_map, 5, "private-access"));
        assert!(should_ignore_rule(&ignore_map, 5, "constant-name"));
        assert!(!should_ignore_rule(&ignore_map, 5, "other-rule"));
        assert!(!should_ignore_rule(&ignore_map, 6, "private-access"));
    }

    #[test]
    fn test_should_ignore_all_rules() {
        let mut ignore_map = HashMap::new();
        ignore_map.insert(5, HashSet::new()); // Empty set means ignore all

        assert!(should_ignore_rule(&ignore_map, 5, "private-access"));
        assert!(should_ignore_rule(&ignore_map, 5, "any-rule"));
        assert!(!should_ignore_rule(&ignore_map, 6, "private-access"));
    }

    #[test]
    fn test_parse_with_spaces_and_commas() {
        let source = r#"# gdlint-ignore-next-line private-access , constant-name  ,  other-rule
some_code_with_issues()"#;

        let ignore_map = parse_ignore_patterns(source);
        let rules = ignore_map.get(&2).unwrap();

        assert!(rules.contains("private-access"));
        assert!(rules.contains("constant-name"));
        assert!(rules.contains("other-rule"));
        assert_eq!(rules.len(), 3);
    }
}
