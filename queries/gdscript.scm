; GDScript formatting queries for Topiary
; This is an early work-in-progress!

; Add a space after keywords
[
  "class_name" "extends" "var" "func" "class"
  "if" "elif" "else" "for" "while"
  "const" "return" "match" "signal" "enum"
  "await" "remote" "master" "puppet" "remotesync"
  "mastersync" "puppetsync"
  (static_keyword)]
@append_space

; Preserve comments and strings as they are
(comment) @leaf
(string) @leaf
(string_name) @leaf
(node_path) @leaf
(region) @leaf

; TYPE ANNOTATION SPACING
(typed_parameter ":" @append_space)
(typed_default_parameter ":" @append_space)
(variable_statement ":" @append_space)
(subscript_arguments "," @append_space)

; ARRAY AND DICTIONARY
; If the array is on a single line, only insert spaces between values. If it's
; multi-line, format it with new lines.
(array
  "[" @append_empty_softline @append_indent_start
  "]" @prepend_empty_softline @append_empty_softline @prepend_indent_end)
(array "," @append_spaced_softline . (comment)? @do_nothing)
(array ((_expression) @append_delimiter (#delimiter! ",") . ","? @do_nothing . (comment)? . "]") (#multi_line_only!))

(array "," @delete . "]" (#single_line_only!))
(dictionary "," @delete . "}" (#single_line_only!))

(dictionary
  "{" @append_empty_softline @append_indent_start
  "}" @prepend_empty_softline @prepend_indent_end)
(dictionary "," @append_spaced_softline . (comment)? @do_nothing)
(dictionary "{" @append_space "}" @prepend_space (#single_line_only!))
(pair ":" @append_space)
(dictionary ((pair (_expression)) @append_delimiter (#delimiter! ",") . ","? @do_nothing . (comment)? . "}") (#multi_line_only!))

; FUNCTIONS
(function_definition (name) @append_antispace)
(function_definition (body) @prepend_hardline @append_hardline)
; This forces adding a line above all functions during the topiary formatting
; pass. Without this cases like "func a(): pass" would not get a line above them
; and get merged together, preventing the post-processing step in formatter.rs
; from catching the functions and adding the 2 new lines.
(function_definition) @prepend_hardline
"->" @prepend_space @append_space
(arguments "," @append_space (#single_line_only!))
(arguments "," @delete . ")" (#single_line_only!))
(parameters "," @append_space (#single_line_only!))
(parameters "," @delete . ")" (#single_line_only!))

; MULTI-LINE ARGUMENTS (in function calls)
(arguments "," @append_hardline (#multi_line_only!))
; uncomment for double indentation in multiline function calls
; (arguments (_) @prepend_indent_start @append_indent_end)
(arguments
    "(" @append_hardline @append_indent_start
    ")" @prepend_hardline @prepend_indent_end
    (#multi_line_only!))
(arguments ((_expression) @append_delimiter (#delimiter! ",") . ","? @do_nothing . (comment)? . ")") (#multi_line_only!))

; MULTI-LINE PARAMETERS (in function definitions)
(parameters
    "(" @append_hardline @append_indent_start
    ")" @prepend_hardline @prepend_indent_end
    (#multi_line_only!))
(parameters
    ([(typed_parameter) (typed_default_parameter) (identifier) (default_parameter)]) @prepend_hardline @prepend_indent_start @append_indent_end
    (#multi_line_only!))
(parameters (([(typed_parameter) (typed_default_parameter) (identifier) (default_parameter)]) @append_delimiter (#delimiter! ",") . ","? @do_nothing . (comment)? . ")") (#multi_line_only!))

; CLASS DEFINITIONS
(class_definition (body) @prepend_hardline @append_hardline)
(class_definition (body (extends_statement) @append_hardline ))
(class_name_statement) @append_space
(class_definition) @prepend_hardline
(source
    (extends_statement) @append_delimiter @append_hardline
    (#delimiter! "\n"))
(extends_statement) @prepend_space

; CONST DEFINITIONS
(const_statement ":" @append_space)

; ENUMS
(enumerator_list
  "{" @append_empty_softline @append_indent_start
  "}" @prepend_empty_softline @prepend_indent_end)
(enumerator_list "{" @append_space "}" @prepend_space (#single_line_only!))
(enumerator_list "," @append_spaced_softline . (comment)? @do_nothing)
(enumerator_list ((enumerator) @append_delimiter (#delimiter! ",") . ","? @do_nothing . (comment)? . "}") (#multi_line_only!))
(enumerator_list) @prepend_space
(enumerator_list "," @delete . "}" (#single_line_only!))

; CONSTRUCTORS
(constructor_definition (body) @prepend_hardline)

; OPERATORS
; Allow line breaks around binary operators for long expressions
; This means that if the programmer has a long expression, they can break it up by wrapping something on a line
(binary_operator
  [
    "+" "-" "*" "/" "%" "**"
    "==" "!=" "<" ">" "<=" ">=" "and"
    "or" "in" "is" "&&" "||" "not"]
  @prepend_input_softline @append_input_softline)
; Comparison operators (+ "as" keyword which needs the same spacing)
[
    "==" "!=" "<" ">" "<=" ">="
    "and" "or" "in" "is" "as"]
@prepend_space @append_space
; not can be at the start of an expression, so we handle it separately - needs another query for the case "is not"
"not" @append_space
; Bitwise operators
[
  "&" "|" "^" "<<" ">>"]
@prepend_space @append_space
; ~ is generally right next to the variable it operates on, so we don't add a space before it
"~" @append_space
[
    "=" ":=" "+=" "-=" "*=" "/=" "%=" "**=" "&=" "|=" "^=" "<<=" ">>="]
@prepend_space @append_space

; CONTROL FLOW FORMATTING
; Colons in control structures - remove space before colon
(if_statement ":" @prepend_antispace)
(elif_clause ":" @prepend_antispace)
(else_clause ":" @prepend_antispace)
(for_statement "in" ":" @prepend_antispace)
(while_statement ":" @prepend_antispace)

(if_statement (body) @prepend_hardline)
(elif_clause (body) @prepend_hardline)
(else_clause (body) @prepend_hardline)
(for_statement (body) @prepend_hardline)
(while_statement (body) @prepend_hardline)

((identifier) . ":" @append_space . (type))

; Make sure the body of control structures is indented (the preprended and
; appended indents target the body)
((body) @prepend_indent_start @append_indent_end)

([(return_statement)
  (pass_statement)
  (breakpoint_statement)
  (break_statement)
  (continue_statement)
  (tool_statement)
  (enum_definition)
  (const_statement)
  (signal_statement)
  (variable_statement)
  (expression_statement)
  (if_statement)
  (elif_clause)
  (else_clause)
  (for_statement)
  (while_statement)
  (match_statement)] @append_empty_softline
 . (comment)? @do_nothing)

(comment) @append_empty_softline @prepend_input_softline

; Allow one blank line before following statements
([(return_statement)
  (pass_statement)
  (breakpoint_statement)
  (break_statement)
  (continue_statement)
  (tool_statement)
  (enum_definition)
  (const_statement)
  (signal_statement)
  (variable_statement)
  (expression_statement)
  (if_statement)
  (elif_clause)
  (else_clause)
  (for_statement)
  (while_statement)
  (match_statement)
  (comment)
  (annotation)] @allow_blank_line_before)

(setget) @prepend_indent_start @append_indent_end
(setget ":" @prepend_antispace)
(setget ":" @append_hardline . (comment)? @do_nothing)
(setget "," @append_space)
(set_body ":" @prepend_antispace)
(set_body (body) @prepend_hardline @append_hardline)
(get_body ":" @prepend_antispace)
(get_body (body) @prepend_hardline @append_hardline)

(match_statement ":" @prepend_antispace)
(match_body) @prepend_indent_start @append_indent_end @prepend_hardline
(pattern_section ":" @prepend_antispace)
(pattern_section (body) @prepend_hardline)
(pattern_section "," @prepend_antispace @append_space)
(pattern_guard) @prepend_space
(pattern_guard (_) @prepend_space)

; This is for ternary expressions, e.g. `a if b else c`
(conditional_expression [("if") ("else")] @prepend_space @append_space)
(parenthesized_expression (conditional_expression ("else") @prepend_input_softline))
(conditional_expression (conditional_expression ("else") @prepend_input_softline))

(parenthesized_expression "(" @append_antispace)
(parenthesized_expression
 "(" @append_input_softline @append_indent_start
 ")" @prepend_input_softline @prepend_indent_end
 (#multi_line_only!))

; LAMBDA
(lambda ":" @append_space (#single_line_only!))
(lambda ":" @append_hardline . (comment)? @do_nothing (#multi_line_only!))
(lambda (parameters "(" @prepend_antispace))

; ANNOTATIONS
; we again are using @append_space capture name, but this time we
; need to make sure to not add additional space between identifier and open paren
(annotation) @append_space
((annotation (identifier) @append_space) @append_empty_softline . (comment)? @do_nothing (#not-match? @append_space "^(onready|export)$"))
(annotation (arguments "(" @prepend_antispace))
(function_definition (annotations (annotation) @append_hardline))

; This is used to preserve new lines after semicolons for people who use them on
; all code lines
(";") @append_hardline

; Calls to get_node get parsed as special nodes, we need them to preserve cases like %NodeName or $Path/To/Node
(get_node) @leaf

(line_continuation) @prepend_space @append_antispace @append_hardline
