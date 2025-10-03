#[cfg(test)]
mod tests {
    use crate::linter::{LintIssue, LintSeverity, LinterConfig, lint_gdscript_with_config};

    #[test]
    fn test_lint_basic_functionality() {
        let test_code = r#"
class_name badClassName
extends Node

signal BadSignal

const badConstant = 10
const GOOD_CONSTANT = 20

var badVariable = 30
var good_variable = 40

func badFunctionName():
    pass

func good_function_name():
    pass
"#;

        let config = LinterConfig::default();
        let issues = lint_gdscript_with_config(test_code, "test.gd", &config).unwrap();

        // Should have at least some issues
        assert!(!issues.is_empty());

        // Check for specific issues
        let rule_names: Vec<&str> = issues.iter().map(|i| i.rule.as_str()).collect();
        assert!(rule_names.contains(&"class-name"));
        assert!(rule_names.contains(&"signal-name"));
        assert!(rule_names.contains(&"constant-name"));
        assert!(rule_names.contains(&"variable-name"));
        assert!(rule_names.contains(&"function-name"));
    }

    #[test]
    fn test_lint_rule_disabling() {
        let test_code = r#"
class_name badClassName
signal BadSignal
"#;

        let mut config = LinterConfig::default();
        config.disabled_rules.insert("class-name".to_string());

        let issues = lint_gdscript_with_config(test_code, "test.gd", &config).unwrap();

        // Should still have signal-name issue but not class-name
        let rule_names: Vec<&str> = issues.iter().map(|i| i.rule.as_str()).collect();
        assert!(!rule_names.contains(&"class-name"));
        assert!(rule_names.contains(&"signal-name"));
    }

    #[test]
    fn test_lint_issue_format() {
        let issue = LintIssue::new(
            10,
            5,
            "test-rule".to_string(),
            LintSeverity::Error,
            "Test message".to_string(),
        );

        let formatted = issue.format("test.gd");
        assert_eq!(formatted, "test.gd:10:test-rule:error: Test message");
    }

    #[test]
    fn test_parse_disabled_rules() {
        let rules = crate::linter::rule_config::parse_disabled_rules(
            "class-name,signal-name,function-name",
        );
        assert_eq!(rules.len(), 3);
        assert!(rules.contains("class-name"));
        assert!(rules.contains("signal-name"));
        assert!(rules.contains("function-name"));
    }

    #[test]
    fn test_validate_rule_names() {
        use std::collections::HashSet;

        let mut valid_rules = HashSet::new();
        valid_rules.insert("class-name".to_string());
        valid_rules.insert("signal-name".to_string());

        assert!(crate::linter::rule_config::validate_rule_names(&valid_rules).is_ok());

        let mut invalid_rules = HashSet::new();
        invalid_rules.insert("invalid-rule".to_string());

        assert!(crate::linter::rule_config::validate_rule_names(&invalid_rules).is_err());
    }
}
