use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity, LinterConfig};
use tree_sitter::Node;

pub struct MaxLineLengthRule {
    config: LinterConfig,
}

impl MaxLineLengthRule {
    pub fn new(config: &LinterConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    fn check_max_line_length(&self, _node: &Node, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_number, line) in source_code.lines().enumerate() {
            // Calculate display width accounting for tabs
            let display_width = line.chars().fold(0, |acc, ch| {
                if ch == '\t' {
                    acc + 4 // Assume tab width of 4
                } else {
                    acc + 1
                }
            });

            if display_width > self.config.max_line_length {
                issues.push(LintIssue::new(
                    line_number + 1, // Convert 0-based to 1-based line numbers
                    self.config.max_line_length + 1,
                    "max-line-length".to_string(),
                    LintSeverity::Warning,
                    format!(
                        "Line is too long. Found {} characters, maximum allowed is {}",
                        display_width, self.config.max_line_length
                    ),
                ));
            }
        }

        issues
    }
}

impl Rule for MaxLineLengthRule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String> {
        Ok(self.check_max_line_length(root_node, source_code))
    }
}
