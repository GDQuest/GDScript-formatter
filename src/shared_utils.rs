/// Returns `true` if the annotation should be inline, `false` otherwise. All
/// export annotations except those that create inspector groups or categories
/// and the onready annotation should be written inline.
pub fn should_annotation_be_inline(annotation_name: &str) -> bool {
    (annotation_name.starts_with("export")
        && !["export_group", "export_subgroup", "export_category"].contains(&annotation_name))
        || annotation_name == "onready"
}

/// Returns the display width up to the first line return using the formatter's
/// indentation rules. Tabs count as one indentation level, while all other
/// characters count as one column.
pub fn measure_line_length_until_line_return(text: &str, indent_size: usize) -> usize {
    let mut width: usize = 0;
    for character in text.chars() {
        if character == '\n' {
            break;
        }
        if character == '\t' {
            width += indent_size;
        } else {
            width += 1;
        }
    }
    width
}
