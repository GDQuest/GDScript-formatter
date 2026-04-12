use crate::linter::rules::Rule;
use crate::linter::{LintIssue, LintSeverity};
use crate::reorder::{GDScriptTokenKind, GDScriptTokensWithComments, collect_top_level_tokens, sort_gdscript_tokens};
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
        // NOTE: This rule re-parses the source because the Rule trait's check_source
        // only receives the raw string, not the pre-built tree from the linter.
        // TODO: consider extending the Rule trait to pass the tree to avoid the
        // extra parse for source-level rules that need the AST.
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

        // Clone tokens before sorting so the original order is still available for index lookup.
        let sorted = sort_gdscript_tokens(tokens.clone());

        // Build sorted_original_indices: for each token in the expected (sorted) order,
        // find its position in the original list. We match by both start_byte AND end_byte
        // because synthetic tokens (Docstring, Unknown) are assigned start_byte: 0 as a
        // sentinel, making start_byte alone non-unique.
        let mut sorted_original_indices: Vec<usize> = Vec::with_capacity(sorted.len());
        let mut remaining: Vec<(usize, &crate::reorder::GDScriptTokensWithComments)> =
            tokens.iter().enumerate().collect();

        for sorted_token in &sorted {
            // Find this token in the remaining original tokens by matching start_byte AND end_byte
            if let Some(pos) = remaining.iter().position(|(_, t)| {
                t.start_byte == sorted_token.start_byte && t.end_byte == sorted_token.end_byte
            }) {
                let (orig_idx, _) = remaining.remove(pos);
                sorted_original_indices.push(orig_idx);
            }
        }

        if sorted_original_indices.len() != sorted.len() {
            // Couldn't match all tokens; bail out safely
            return vec![];
        }

        let mut issues = Vec::new();

        // For each consecutive pair in the expected (sorted) order, check that
        // the second element did not appear *before* the first one in the original.
        for i in 1..sorted_original_indices.len() {
            let prev_original_idx = sorted_original_indices[i - 1];
            let curr_original_idx = sorted_original_indices[i];

            if curr_original_idx < prev_original_idx {
                // curr appears earlier in the source than prev, but should come after
                let line = Self::byte_to_line(source_code, tokens[curr_original_idx].start_byte);
                issues.push(LintIssue::new(
                    line,
                    1,
                    "code-order".to_string(),
                    LintSeverity::Warning,
                    format!(
                        "{} should appear after {} according to the GDScript style guide",
                        Self::describe(&sorted[i].token_kind),
                        Self::describe(&sorted[i - 1].token_kind),
                    ),
                ));
            }
        }

        issues
    }
}
