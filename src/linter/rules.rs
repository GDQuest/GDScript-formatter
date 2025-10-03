pub mod class_name;
pub mod comparison_with_itself;
pub mod constant_name;
pub mod duplicated_load;
pub mod enum_member_name;
pub mod enum_name;
pub mod function_argument_name;
pub mod function_name;
pub mod loop_variable_name;
pub mod max_line_length;
pub mod no_else_return;
pub mod private_access;
pub mod signal_name;
pub mod standalone_expression;
pub mod unnecessary_pass;
pub mod unused_argument;
pub mod variable_name;

use crate::linter::{LintIssue, LinterConfig};
use tree_sitter::Node;

pub trait Rule {
    fn check(&mut self, source_code: &str, root_node: &Node) -> Result<Vec<LintIssue>, String>;
}

use class_name::ClassNameRule;
use comparison_with_itself::ComparisonWithItselfRule;
use constant_name::ConstantNameRule;
use duplicated_load::DuplicatedLoadRule;
use enum_member_name::EnumMemberNameRule;
use enum_name::EnumNameRule;
use function_argument_name::FunctionArgumentNameRule;
use function_name::FunctionNameRule;
use loop_variable_name::LoopVariableNameRule;
use max_line_length::MaxLineLengthRule;
use no_else_return::NoElseReturnRule;
use private_access::PrivateAccessRule;
use signal_name::SignalNameRule;
use standalone_expression::StandaloneExpressionRule;
use unnecessary_pass::UnnecessaryPassRule;
use unused_argument::UnusedArgumentRule;
use variable_name::VariableNameRule;

pub struct RuleDefinition {
    pub name: &'static str,
    pub create: fn(&LinterConfig) -> Box<dyn Rule>,
}

/// List of all available rules
pub const ALL_RULES: &[RuleDefinition] = &[
    RuleDefinition {
        name: "duplicated-load",
        create: |_config| Box::new(DuplicatedLoadRule::new()),
    },
    RuleDefinition {
        name: "standalone-expression",
        create: |_config| Box::new(StandaloneExpressionRule),
    },
    RuleDefinition {
        name: "unnecessary-pass",
        create: |_config| Box::new(UnnecessaryPassRule),
    },
    RuleDefinition {
        name: "unused-argument",
        create: |_config| Box::new(UnusedArgumentRule),
    },
    RuleDefinition {
        name: "comparison-with-itself",
        create: |_config| Box::new(ComparisonWithItselfRule),
    },
    RuleDefinition {
        name: "private-access",
        create: |_config| Box::new(PrivateAccessRule),
    },
    RuleDefinition {
        name: "max-line-length",
        create: |config| Box::new(MaxLineLengthRule::new(config)),
    },
    RuleDefinition {
        name: "no-else-return",
        create: |_config| Box::new(NoElseReturnRule),
    },
    RuleDefinition {
        name: "function-name",
        create: |_config| Box::new(FunctionNameRule),
    },
    RuleDefinition {
        name: "class-name",
        create: |_config| Box::new(ClassNameRule),
    },
    RuleDefinition {
        name: "signal-name",
        create: |_config| Box::new(SignalNameRule),
    },
    RuleDefinition {
        name: "variable-name",
        create: |_config| Box::new(VariableNameRule),
    },
    RuleDefinition {
        name: "function-argument-name",
        create: |_config| Box::new(FunctionArgumentNameRule),
    },
    RuleDefinition {
        name: "loop-variable-name",
        create: |_config| Box::new(LoopVariableNameRule),
    },
    RuleDefinition {
        name: "enum-name",
        create: |_config| Box::new(EnumNameRule),
    },
    RuleDefinition {
        name: "enum-member-name",
        create: |_config| Box::new(EnumMemberNameRule),
    },
    RuleDefinition {
        name: "constant-name",
        create: |_config| Box::new(ConstantNameRule),
    },
];
