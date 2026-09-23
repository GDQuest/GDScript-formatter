/// Returns `true` if the annotation should be inline, `false` otherwise. All
/// export annotations except those that create inspector groups or categories
/// and the onready annotation should be written inline.
pub fn should_annotation_be_inline(annotation_name: &str) -> bool {
    (annotation_name.starts_with("export")
        && !["export_group", "export_subgroup", "export_category"].contains(&annotation_name))
        || annotation_name == "onready"
}
