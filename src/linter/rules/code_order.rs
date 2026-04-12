use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use crate::reorder::{GDScriptTokenKind, collect_top_level_tokens, sort_gdscript_tokens};
use tree_sitter::Parser;

pub struct CodeOrderRule;

impl CodeOrderRule {
    /// Returns a human-readable description of a token kind for use in messages.
    fn describe(kind: &GDScriptTokenKind) -> String {
        match kind {
            GDScriptTokenKind::ClassAnnotation(text) => {
                format!("class annotation `{}`", text.trim())
            }
            GDScriptTokenKind::ClassName(name) => format!("`class_name {}`", name),
            GDScriptTokenKind::Extends(name) => format!("`extends {}`", name),
            GDScriptTokenKind::Docstring(_) => "class docstring".to_string(),
            GDScriptTokenKind::Signal(name, _) => format!("signal `{}`", name),
            GDScriptTokenKind::Enum(name, _) => format!("enum `{}`", name),
            GDScriptTokenKind::Constant(name, _) => format!("constant `{}`", name),
            GDScriptTokenKind::StaticVariable(name, _) => {
                format!("static variable `{}`", name)
            }
            GDScriptTokenKind::ExportVariable(name, _) => {
                format!("@export variable `{}`", name)
            }
            GDScriptTokenKind::RegularVariable(name, _) => format!("variable `{}`", name),
            GDScriptTokenKind::OnReadyVariable(name, _) => {
                format!("@onready variable `{}`", name)
            }
            GDScriptTokenKind::Method(name, _, _) => format!("method `{}`", name),
            GDScriptTokenKind::InnerClass(name, _) => format!("inner class `{}`", name),
            GDScriptTokenKind::Unknown(_) => "unknown element".to_string(),
        }
    }

    /// Converts a byte offset in `source` to a 1-based line number.
    fn byte_to_line(source: &str, byte_offset: usize) -> usize {
        source[..byte_offset].chars().filter(|&c| c == '\n').count() + 1
    }
}

impl Rule for CodeOrderRule {
    fn check_source(&mut self, source_code: &str) -> Vec<LintIssue> {
        let mut parser = Parser::new();
        if parser
            .set_language(&tree_sitter_gdscript::LANGUAGE.into())
            .is_err()
        {
            return vec![];
        }

        let tree = match parser.parse(source_code, None) {
            Some(t) => t,
            None => return vec![],
        };

        let tokens = match collect_top_level_tokens(&tree, source_code) {
            Ok(t) => t,
            Err(_) => return vec![],
        };

        if tokens.len() < 2 {
            return vec![];
        }

        // Build a lookup: start_byte -> index in the original (unsorted) list
        let original_index: std::collections::HashMap<usize, usize> = tokens
            .iter()
            .enumerate()
            .map(|(i, t)| (t.start_byte, i))
            .collect();

        let sorted = sort_gdscript_tokens(tokens);

        let mut issues = Vec::new();

        // For each consecutive pair in the expected (sorted) order, check that
        // the second element did not appear *before* the first one in the original.
        for i in 1..sorted.len() {
            let prev = &sorted[i - 1];
            let curr = &sorted[i];

            let prev_original_idx = original_index[&prev.start_byte];
            let curr_original_idx = original_index[&curr.start_byte];

            if curr_original_idx < prev_original_idx {
                // curr comes earlier in the source than prev, but should come after
                let line = Self::byte_to_line(source_code, curr.start_byte);
                issues.push(LintIssue::new(
                    line,
                    1,
                    "code-order".to_string(),
                    LintSeverity::Warning,
                    format!(
                        "{} should appear after {} according to the GDScript style guide",
                        Self::describe(&curr.token_kind),
                        Self::describe(&prev.token_kind),
                    ),
                ));
            }
        }

        issues
    }
}
