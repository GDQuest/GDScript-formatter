use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::{fs, io::IsTerminal};
use tree_sitter::{Node, Parser};

pub mod ignore_patterns;
pub mod lib;
pub mod regex_patterns;
pub mod rule_config;
pub mod rules;

#[cfg(test)]
mod tests;

use ignore_patterns::{parse_ignore_patterns, should_ignore_rule};
use rules::{ALL_RULES, Rule};

#[derive(Debug, Clone, PartialEq)]
pub enum LintSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct LintIssue {
    pub line: usize,
    pub column: usize,
    pub rule: String,
    pub severity: LintSeverity,
    pub message: String,
}

impl LintIssue {
    pub fn new(
        line: usize,
        column: usize,
        rule: String,
        severity: LintSeverity,
        message: String,
    ) -> Self {
        Self {
            line,
            column,
            rule,
            severity,
            message,
        }
    }

    pub fn format(&self, file_path: &str) -> String {
        let severity_str = match self.severity {
            LintSeverity::Error => "error",
            LintSeverity::Warning => "warning",
        };
        format!(
            "{}:{}:{}:{}: {}",
            file_path, self.line, self.rule, severity_str, self.message
        )
    }
}

#[derive(Debug, Clone)]
pub struct LinterConfig {
    pub disabled_rules: HashSet<String>,
    pub max_line_length: usize,
}

impl Default for LinterConfig {
    fn default() -> Self {
        Self {
            disabled_rules: HashSet::new(),
            max_line_length: 100,
        }
    }
}

pub struct GDScriptLinter {
    config: LinterConfig,
    parser: Parser,
}

impl GDScriptLinter {
    pub fn new(config: LinterConfig) -> Result<Self, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_gdscript::LANGUAGE.into())
            .map_err(|e| format!("Failed to set language: {}", e))?;

        Ok(Self { config, parser })
    }

    pub fn lint(&mut self, source_code: &str, _file_path: &str) -> Result<Vec<LintIssue>, String> {
        let tree = self
            .parser
            .parse(source_code, None)
            .ok_or("Failed to parse GDScript code")?;

        let root_node = tree.root_node();
        let mut issues = Vec::new();

        let ignore_map = parse_ignore_patterns(source_code);

        let mut checkers: Vec<Box<dyn Rule>> = Vec::new();
        for current_rule in ALL_RULES {
            if !self.config.disabled_rules.contains(current_rule.name) {
                checkers.push((current_rule.create)(&self.config));
            }
        }

        // Here we build a map from node kinds to the rules that care about
        // them. That allows us to use the visitor pattern to go through the
        // tree only once. We call each rule only when we encounter an AST node
        // it cares about.
        let mut node_kind_map: HashMap<String, Vec<usize>> = HashMap::new();
        let mut source_only_rules = Vec::new();
        for (current_index, checker) in checkers.iter().enumerate() {
            let kinds = checker.get_target_ast_nodes();
            if kinds.is_empty() {
                source_only_rules.push(current_index);
            } else {
                for &kind in kinds {
                    node_kind_map
                        .entry(kind.to_string())
                        .or_default()
                        .push(current_index);
                }
            }
        }

        // First we run the rules that only care about the source code. Then we
        // visit each node in the AST, calling the relevant rules. Finally we
        // call the finalize method on each rule in case a rule needs to collect
        // state while visiting nodes and report issues at the end.
        for &current_index in &source_only_rules {
            let rule_issues = checkers[current_index].check_source(source_code);
            for issue in rule_issues {
                if !should_ignore_rule(&ignore_map, issue.line, &issue.rule) {
                    issues.push(issue);
                }
            }
        }
        visit_each_node(
            &root_node,
            source_code,
            &mut checkers,
            &node_kind_map,
            &mut issues,
            &ignore_map,
        );
        for checker in checkers.iter_mut() {
            let rule_issues = checker.finalize(source_code);
            for issue in rule_issues {
                if !should_ignore_rule(&ignore_map, issue.line, &issue.rule) {
                    issues.push(issue);
                }
            }
        }

        // Sort issues by line number. Rules that run on the source code like
        // line length check will otherwise appear at the end.
        issues.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));

        Ok(issues)
    }

    pub fn lint_files(
        &mut self,
        input_files: Vec<PathBuf>,
        pretty: bool,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let gdscript_files: Vec<&PathBuf> = input_files
            .iter()
            .filter(|path| path.extension().map_or(false, |ext| ext == "gd"))
            .collect();

        if gdscript_files.is_empty() {
            eprintln!(
                "Error: No GDScript files found in the arguments provided. Please provide at least one .gd file."
            );
            std::process::exit(1);
        }

        let with_colors = std::io::stdout().is_terminal();

        if pretty {
            self.lint_files_pretty(&gdscript_files, with_colors)
        } else {
            self.lint_files_standard(&gdscript_files)
        }
    }

    fn lint_files_pretty(
        &mut self,
        gdscript_files: &[&PathBuf],
        with_colors: bool,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use std::collections::HashMap;
        let mut file_issues: HashMap<String, Vec<_>> = HashMap::new();
        let mut has_issues = false;

        for file_path in gdscript_files {
            let source_code = fs::read_to_string(file_path).map_err(|error| {
                format!("Failed to read file {}: {}", file_path.display(), error)
            })?;

            let issues = self.lint(&source_code, &file_path.to_string_lossy())?;

            if !issues.is_empty() {
                has_issues = true;
                file_issues.insert(file_path.to_string_lossy().to_string(), issues);
            }
        }

        // Print pretty output grouped by file and line
        let mut file_iter = file_issues.iter().peekable();
        while let Some((file_path, issues)) = file_iter.next() {
            let bold = if with_colors { "\x1b[1m" } else { "" };
            let reset = if with_colors { "\x1b[0m" } else { "" };

            println!("{}{}{}", bold, file_path, reset);

            // Group issues by line number
            let mut line_issues: HashMap<usize, Vec<_>> = HashMap::new();
            for issue in issues {
                line_issues.entry(issue.line).or_default().push(issue);
            }

            // Sort by line number and print
            let mut sorted_lines: Vec<_> = line_issues.keys().collect();
            sorted_lines.sort();

            for (i, &line_num) in sorted_lines.iter().enumerate() {
                if let Some(line_issues) = line_issues.get(&line_num) {
                    println!("    {}:{}", file_path, line_num);
                    for issue in line_issues {
                        let (severity_str, severity_color) = match issue.severity {
                            LintSeverity::Error => ("ERROR", "\x1b[31m"),  // Red
                            LintSeverity::Warning => ("WARN", "\x1b[33m"), // Yellow
                        };

                        if with_colors {
                            println!(
                                "        {}{}\x1b[0m: `{}`",
                                severity_color, severity_str, issue.rule
                            );
                        } else {
                            println!("        {}: `{}`", severity_str, issue.rule);
                        }
                        println!("        {}", issue.message);
                    }

                    // Add newline between line groups (except for the last line group)
                    if i < sorted_lines.len() - 1 {
                        println!();
                    }
                }
            }

            // Add separator between files (except for the last file)
            if file_iter.peek().is_some() {
                println!("\n{}", "-".repeat(60));
                println!();
            }
        }

        Ok(has_issues)
    }

    fn lint_files_standard(
        &mut self,
        gdscript_files: &[&PathBuf],
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut has_issues = false;

        for file_path in gdscript_files {
            let source_code = fs::read_to_string(file_path).map_err(|error| {
                format!("Failed to read file {}: {}", file_path.display(), error)
            })?;

            let issues = self.lint(&source_code, &file_path.to_string_lossy())?;

            for issue in issues {
                has_issues = true;
                println!("{}", issue.format(&file_path.to_string_lossy()));
            }
        }

        Ok(has_issues)
    }
}

/// This uses the visitor pattern to walk the parsed tree sitter AST only once.
/// We call each rule only when we encounter an AST node it cares about.
fn visit_each_node(
    node: &Node,
    source_code: &str,
    checkers: &mut [Box<dyn Rule>],
    node_kind_map: &HashMap<String, Vec<usize>>,
    issues: &mut Vec<LintIssue>,
    ignore_map: &HashMap<usize, HashSet<String>>,
) {
    if let Some(matching_rules) = node_kind_map.get(node.kind()) {
        for &rule_idx in matching_rules {
            let rule_issues = checkers[rule_idx].check_node(node, source_code);
            for issue in rule_issues {
                if !should_ignore_rule(ignore_map, issue.line, &issue.rule) {
                    issues.push(issue);
                }
            }
        }
    }

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            visit_each_node(
                &cursor.node(),
                source_code,
                checkers,
                node_kind_map,
                issues,
                ignore_map,
            );
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

pub fn lint_gdscript_with_config(
    source_code: &str,
    file_path: &str,
    config: &LinterConfig,
) -> Result<Vec<LintIssue>, String> {
    let mut linter = GDScriptLinter::new(config.clone())?;
    linter.lint(source_code, file_path)
}

pub fn lint_gdscript(source_code: &str, file_path: &str) -> Result<Vec<LintIssue>, String> {
    let config = LinterConfig::default();
    lint_gdscript_with_config(source_code, file_path, &config)
}
