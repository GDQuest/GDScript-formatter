use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity, LinterConfig};

pub struct MaxLineLengthRule {
    config: LinterConfig,
}

impl MaxLineLengthRule {
    pub fn new(config: &LinterConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl Rule for MaxLineLengthRule {
    fn check_source(&mut self, source_code: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_number, line) in source_code.lines().enumerate() {
            let display_width = line
                .chars()
                .fold(0, |acc, ch| if ch == '\t' { acc + 4 } else { acc + 1 });

            if display_width > self.config.max_line_length {
                issues.push(LintIssue::new(
                    line_number + 1,
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

    fn get_target_ast_nodes(&self) -> &[&str] {
        &[]
    }
}
