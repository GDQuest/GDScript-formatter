/// This module pairs tree-sitter node kind strings with our own enum so we can
/// classify AST nodes in constant time.
///
/// We use our own enum instead of tree-sitter strings for two reasons:
///
/// 1. type safety. You cannot accidentally compare a node kind to the
///    wrong string or misspell a name.
/// 2. It separates our code from tree-sitter a little. We use this project to
///    test and refine the tree-sitter GDScript parser. That helps improve GDScript
///    support in code editors. But we may replace the parser one day, and the
///    enum makes it easier to change.
use std::sync::OnceLock;
use tree_sitter::Language;

/// This is how we represent GDScript AST node kinds.
///
/// We list every tree-sitter node kind that any part of the formatter branches
/// on. Unknown or unused kinds fall through to the Other enum member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GDScriptNodeKind {
    // Structural
    Body,
    ClassBody,
    MatchBody,
    GetBody,
    SetBody,

    // Declarations
    ClassName,
    Extends,
    ClassDefinition,
    InnerClass,
    Variable,
    Const,
    Enum,
    Signal,
    Function,
    Constructor,
    ExportVariable,
    OnReadyVariable,

    // Annotations and comments
    Annotation,
    Annotations,
    Comment,

    // Containers
    Array,
    Dictionary,
    EnumeratorList,
    Parameters,
    Arguments,
    SubscriptArguments,
    Condition,

    // Expressions
    Lambda,
    SetGet,
    ParenthesizedExpression,
    BinaryOperator,
    UnaryOperator,
    Call,
    Assignment,
    AugmentedAssignment,
    ExpressionStatement,

    // Control flow
    IfStatement,
    ElifStatement,
    ElseStatement,
    ForStatement,
    ReturnStatement,
    PassStatement,

    // Attribute access and method calls
    Attribute,
    AttributeCall,

    // Leaf nodes with special handling
    String,
    StringName,
    NodePath,
    GetNode,
    LineContinuation,
    Enumerator,
    Identifier,
    /// true, false, null, integer, float
    Literal,
    /// typed_parameter, default_parameter, typed_default_parameter
    Parameter,

    // Punctuation. Tree-sitter treats these as named nodes in its concrete
    // syntax tree.
    /// ( )
    TokenParen,
    /// { }
    TokenBrace,
    /// [ ]
    TokenBracket,
    TokenDot,
    TokenComma,
    TokenColon,
    SemiColon,
    /// + - etc.
    Operator,

    // Keyword nodes
    KeywordFunc,
    KeywordStatic,

    // Type annotations
    InferredType,

    // Meta
    RegionStart,
    RegionEnd,
    Error,

    // Special identifiers used in emit_inter_child_separator to decide when to
    // skip a space.
    NameInit,
    NameSet,
    NameGet,

    /// Catch-all for tree-sitter node kinds we do not need to handle explicitly.
    Other,
}

const LOOKUP_TABLE_SIZE: usize = 256;

/// The lookup table. It maps tree-sitter kind IDs (integers starting from 0) to
/// our enum. We only build it once, on first use.
///
/// Tree-sitter kind IDs are u16 values. In practice right now the GDScript
/// grammar does not have so many node kinds, so this table size should be
/// enough. The assert in the function will catch any out-of-bounds values when
/// running tests.
static LOOKUP_TABLE: OnceLock<[GDScriptNodeKind; LOOKUP_TABLE_SIZE]> = OnceLock::new();

impl GDScriptNodeKind {
    /// Build the lookup table and store it in the global OnceLock.
    ///
    /// You must call this before calling `get_kind_from_ast_node`. The table is
    /// built only once. Subsequent calls return a reference to the cached table.
    pub fn populate_lookup_table() -> &'static [GDScriptNodeKind; LOOKUP_TABLE_SIZE] {
        LOOKUP_TABLE.get_or_init(|| {
            let language: Language = tree_sitter_gdscript::LANGUAGE.into();
            let mut table = [GDScriptNodeKind::Other; LOOKUP_TABLE_SIZE];

            for (name, variant) in MAP_TREE_SITTER_TO_GDSCRIPT_NODE_KIND {
                // A tree-sitter node kind can be either a named kind or an
                // anonymous kind, and sometimes both.
                //
                // Named kinds come from grammar rules. Examples: "if_statement",
                // "binary_operator". Anonymous kinds are literal tokens in the
                // grammar. Examples: "(", ";", ".".
                //
                // We look up both for each token because we do not know in
                // advance which category a node name string falls into.
                //
                // `id_for_node_kind` returns a valid ID (0 through N) when the
                // name exists in that category. It returns u16::MAX (65535) when
                // it does not.
                let named_id = language.id_for_node_kind(name, true);
                let anonymous_id = language.id_for_node_kind(name, false);

                if (named_id as usize) < LOOKUP_TABLE_SIZE {
                    table[named_id as usize] = *variant;
                }
                if (anonymous_id as usize) < LOOKUP_TABLE_SIZE {
                    table[anonymous_id as usize] = *variant;
                }

                // If both lookups returned u16::MAX, the string does not exist
                // in the grammar at all. That would be an error in our map
                assert!(
                    (named_id as usize) < LOOKUP_TABLE_SIZE
                        || (anonymous_id as usize) < LOOKUP_TABLE_SIZE,
                    "The tree-sitter GDScript grammar has no node kind named '{}' or the lookup table is too small.\nReturned IDs are: named={} anonymous={}",
                    name, named_id, anonymous_id
                );
            }

            table
        })
    }

    /// Read the kind of a tree-sitter AST node from the lookup table.
    ///
    /// This is a constant-time array lookup. It panics if you have not called
    /// `populate_lookup_table` first.
    #[inline]
    pub fn get_kind_from_ast_node(node: tree_sitter::Node) -> Self {
        LOOKUP_TABLE
            .get()
            .expect("populate_lookup_table must be called before get_kind_from_ast_node")
            [node.kind_id() as usize]
    }
}

/// Maps every tree-sitter node kind string that appears in our codebase to a
/// GDScriptNodeKind variant.
///
/// This table is iterated at runtime to build the ID-based lookup table above.
/// If you add a new branch on a node kind somewhere in the formatter, you must
/// add an entry here.
const MAP_TREE_SITTER_TO_GDSCRIPT_NODE_KIND: &[(&str, GDScriptNodeKind)] = &[
    ("body", GDScriptNodeKind::Body),
    ("class_body", GDScriptNodeKind::ClassBody),
    ("match_body", GDScriptNodeKind::MatchBody),
    ("get_body", GDScriptNodeKind::GetBody),
    ("set_body", GDScriptNodeKind::SetBody),
    ("class_name_statement", GDScriptNodeKind::ClassName),
    ("extends_statement", GDScriptNodeKind::Extends),
    ("class_definition", GDScriptNodeKind::ClassDefinition),
    ("inner_class", GDScriptNodeKind::InnerClass),
    ("variable_statement", GDScriptNodeKind::Variable),
    (
        "export_variable_statement",
        GDScriptNodeKind::ExportVariable,
    ),
    (
        "onready_variable_statement",
        GDScriptNodeKind::OnReadyVariable,
    ),
    ("const_statement", GDScriptNodeKind::Const),
    ("enum_definition", GDScriptNodeKind::Enum),
    ("signal_statement", GDScriptNodeKind::Signal),
    ("function_definition", GDScriptNodeKind::Function),
    ("constructor_definition", GDScriptNodeKind::Constructor),
    ("annotation", GDScriptNodeKind::Annotation),
    ("annotations", GDScriptNodeKind::Annotations),
    ("comment", GDScriptNodeKind::Comment),
    ("array", GDScriptNodeKind::Array),
    ("dictionary", GDScriptNodeKind::Dictionary),
    ("enumerator_list", GDScriptNodeKind::EnumeratorList),
    ("parameters", GDScriptNodeKind::Parameters),
    ("arguments", GDScriptNodeKind::Arguments),
    ("subscript_arguments", GDScriptNodeKind::SubscriptArguments),
    ("condition", GDScriptNodeKind::Condition),
    ("conditional_expression", GDScriptNodeKind::Condition),
    ("lambda", GDScriptNodeKind::Lambda),
    ("setget", GDScriptNodeKind::SetGet),
    (
        "parenthesized_expression",
        GDScriptNodeKind::ParenthesizedExpression,
    ),
    ("binary_operator", GDScriptNodeKind::BinaryOperator),
    ("unary_operator", GDScriptNodeKind::UnaryOperator),
    ("call", GDScriptNodeKind::Call),
    ("assignment", GDScriptNodeKind::Assignment),
    (
        "augmented_assignment",
        GDScriptNodeKind::AugmentedAssignment,
    ),
    (
        "expression_statement",
        GDScriptNodeKind::ExpressionStatement,
    ),
    ("if_statement", GDScriptNodeKind::IfStatement),
    ("elif_clause", GDScriptNodeKind::ElifStatement),
    ("else_clause", GDScriptNodeKind::ElseStatement),
    ("for_statement", GDScriptNodeKind::ForStatement),
    ("return_statement", GDScriptNodeKind::ReturnStatement),
    ("pass_statement", GDScriptNodeKind::PassStatement),
    ("attribute", GDScriptNodeKind::Attribute),
    ("attribute_call", GDScriptNodeKind::AttributeCall),
    ("string", GDScriptNodeKind::String),
    ("string_name", GDScriptNodeKind::StringName),
    ("node_path", GDScriptNodeKind::NodePath),
    ("get_node", GDScriptNodeKind::GetNode),
    ("line_continuation", GDScriptNodeKind::LineContinuation),
    ("enumerator", GDScriptNodeKind::Enumerator),
    ("identifier", GDScriptNodeKind::Identifier),
    ("true", GDScriptNodeKind::Literal),
    ("false", GDScriptNodeKind::Literal),
    ("null", GDScriptNodeKind::Literal),
    ("integer", GDScriptNodeKind::Literal),
    ("float", GDScriptNodeKind::Literal),
    ("typed_parameter", GDScriptNodeKind::Parameter),
    ("default_parameter", GDScriptNodeKind::Parameter),
    ("typed_default_parameter", GDScriptNodeKind::Parameter),
    ("(", GDScriptNodeKind::TokenParen),
    (")", GDScriptNodeKind::TokenParen),
    ("[", GDScriptNodeKind::TokenBracket),
    ("]", GDScriptNodeKind::TokenBracket),
    ("{", GDScriptNodeKind::TokenBrace),
    ("}", GDScriptNodeKind::TokenBrace),
    (".", GDScriptNodeKind::TokenDot),
    (",", GDScriptNodeKind::TokenComma),
    (":", GDScriptNodeKind::TokenColon),
    (";", GDScriptNodeKind::SemiColon),
    ("+", GDScriptNodeKind::Operator),
    ("-", GDScriptNodeKind::Operator),
    ("func", GDScriptNodeKind::KeywordFunc),
    ("static_keyword", GDScriptNodeKind::KeywordStatic),
    ("inferred_type", GDScriptNodeKind::InferredType),
    ("region_start", GDScriptNodeKind::RegionStart),
    ("region_end", GDScriptNodeKind::RegionEnd),
    ("ERROR", GDScriptNodeKind::Error),
    ("_init", GDScriptNodeKind::NameInit),
    ("set", GDScriptNodeKind::NameSet),
    ("get", GDScriptNodeKind::NameGet),
    ("name", GDScriptNodeKind::Identifier),
];
