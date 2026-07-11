/// Regression tests for the renderer.
///
/// These tests verify that the renderer itself produces the correct output for
/// various kinds of intermediate representations of code, notably around line
/// wrapping.
use gdscript_formatter::renderer::{
    PrinterConfiguration, RangeRenderElement, RangeSourceBytes, RenderElement, render,
};

fn text(start_byte: usize, end_byte: usize) -> RenderElement {
    RenderElement::Text {
        range: RangeSourceBytes {
            start_byte,
            end_byte,
        },
    }
}

fn group(start: usize, end: usize) -> RenderElement {
    RenderElement::Group {
        children: RangeRenderElement { start, end },
    }
}

fn indent(level: u16, start: usize, end: usize) -> RenderElement {
    RenderElement::Indent {
        level,
        child: RangeRenderElement { start, end },
    }
}

fn branch(flat: Option<RangeRenderElement>, break_: Option<RangeRenderElement>) -> RenderElement {
    RenderElement::Branch {
        if_single_line: flat,
        if_multiline: break_,
    }
}

fn get_default_printer_configuration() -> PrinterConfiguration {
    // In this module we test indenting with spaces by default. It's easier to
    // author the tests this way and tests that config does change the output.
    PrinterConfiguration {
        use_spaces: true,
        indent_size: 4,
        ..Default::default()
    }
}

#[test]
fn flat_group_fits_on_one_line() {
    let source = "ab cde";
    let render_elements = vec![group(1, 4), text(0, 2), RenderElement::Space, text(3, 6)];
    let mut out = String::new();
    render(
        &render_elements,
        source,
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "ab cde\n");
}

#[test]
fn flat_group_with_text_static_and_space() {
    let render_elements = vec![
        RenderElement::TextStatic("func"),
        RenderElement::Space,
        RenderElement::TextStatic("foo"),
        RenderElement::TextStatic("()"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "func foo()\n");
}

#[test]
fn break_group_when_too_long() {
    let render_elements = vec![
        group(1, 8),
        RenderElement::TextStatic("aaaa"),
        RenderElement::SoftLine,
        RenderElement::TextStatic("bbbb"),
        RenderElement::SoftLine,
        RenderElement::TextStatic("cccc"),
        RenderElement::SoftLine,
        RenderElement::TextStatic("dddd"),
    ];
    let config = PrinterConfiguration {
        max_line_length: 10,
        use_spaces: true,
        indent_size: 4,
        ..Default::default()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "aaaa\nbbbb\ncccc\ndddd\n");
}

#[test]
fn softline_flat_is_nothing() {
    let source = "a b";
    let render_elements = vec![group(1, 4), text(0, 1), RenderElement::SoftLine, text(2, 3)];
    let mut out = String::new();
    render(
        &render_elements,
        source,
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "ab\n");
}

#[test]
fn hardline_always_breaks_outside_group() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        RenderElement::HardLine,
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a\nb\n");
}

#[test]
fn hardline_inside_group_forces_break_visualization() {
    let render_elements = vec![
        group(1, 4),
        RenderElement::TextStatic("a"),
        RenderElement::HardLine,
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a\nb\n");
}

#[test]
fn indent_applies_after_newline() {
    let render_elements = vec![
        RenderElement::TextStatic("func"),
        RenderElement::TextStatic(":"),
        RenderElement::HardLine,
        indent(1, 4, 7),
        RenderElement::TextStatic("pass"),
        RenderElement::HardLine,
        RenderElement::TextStatic("return"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "func:\n    pass\n    return\n");
}

#[test]
fn nested_indent_stacks() {
    let render_elements = vec![
        RenderElement::TextStatic("if"),
        RenderElement::TextStatic(":"),
        RenderElement::HardLine,
        indent(1, 4, 8),
        RenderElement::TextStatic("a"),
        RenderElement::HardLine,
        indent(1, 7, 8),
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "if:\n    a\n        b\n");
}

#[test]
fn blank_line_emits_one_blank_line() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        RenderElement::BlankLine,
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a\n\nb\n");
}

#[test]
fn blank_line_collapses_consecutive() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        RenderElement::BlankLine,
        RenderElement::BlankLine,
        RenderElement::BlankLine,
        RenderElement::TextStatic("b"),
    ];
    let config = PrinterConfiguration {
        maximum_blank_lines: 1,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "a\n\nb\n");
}

#[test]
fn blank_line_respects_max_cap() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        RenderElement::BlankLine,
        RenderElement::BlankLine,
        RenderElement::TextStatic("b"),
    ];
    let config = PrinterConfiguration {
        maximum_blank_lines: 2,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "a\n\n\nb\n");
}

#[test]
fn force_break_makes_group_break_regardless_of_width() {
    let render_elements = vec![
        group(1, 4),
        RenderElement::TextStatic("a"),
        RenderElement::ForceBreakingParent,
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "ab\n");
}

#[test]
fn force_break_with_softline_emits_newline() {
    let render_elements = vec![
        group(1, 5),
        RenderElement::TextStatic("a"),
        RenderElement::SoftLine,
        RenderElement::ForceBreakingParent,
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a\nb\n");
}

#[test]
fn if_break_flat_mode_emits_no_comma() {
    let render_elements = vec![
        group(1, 4),
        RenderElement::TextStatic("a"),
        branch(None, Some(RangeRenderElement { start: 3, end: 4 })),
        RenderElement::TextStatic(","),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a\n");
}

#[test]
fn if_break_emits_break_when_group_breaks() {
    let render_elements = vec![
        group(1, 4),
        RenderElement::TextStatic("aaaaaaa"),
        branch(None, Some(RangeRenderElement { start: 3, end: 4 })),
        RenderElement::TextStatic(","),
    ];
    let config = PrinterConfiguration {
        max_line_length: 3,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "aaaaaaa,\n");
}

#[test]
fn if_break_flat_branch_rendered_when_flat() {
    let render_elements = vec![
        group(1, 5),
        RenderElement::TextStatic("a"),
        branch(
            Some(RangeRenderElement { start: 3, end: 4 }),
            Some(RangeRenderElement { start: 4, end: 5 }),
        ),
        RenderElement::TextStatic("flat"),
        RenderElement::TextStatic("break"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "aflat\n");
}

#[test]
fn if_break_break_branch_rendered_when_break() {
    let render_elements = vec![
        group(1, 5),
        RenderElement::TextStatic("a"),
        branch(
            Some(RangeRenderElement { start: 3, end: 4 }),
            Some(RangeRenderElement { start: 4, end: 5 }),
        ),
        RenderElement::TextStatic("flat"),
        RenderElement::TextStatic("break"),
    ];
    let config = PrinterConfiguration {
        max_line_length: 3,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "abreak\n");
}

#[test]
fn nested_groups_re_evaluated_independently() {
    let source = "short long_token_here_that_exceeds_limit";
    let render_elements = vec![
        group(1, 5),
        text(0, 5),
        RenderElement::Space,
        group(4, 5),
        text(6, 40),
    ];
    let config = PrinterConfiguration {
        max_line_length: 50,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, source, &config, &mut out);
    assert_eq!(out, "short long_token_here_that_exceeds_limit\n");
}

#[test]
fn nested_inner_group_breaks_when_outer_flat() {
    let render_elements = vec![
        group(1, 7),
        RenderElement::TextStatic("outer"),
        RenderElement::Space,
        group(4, 7),
        RenderElement::TextStatic("inner_long_long_long_long_long"),
        RenderElement::SoftLine,
        RenderElement::TextStatic("x"),
    ];
    let config = PrinterConfiguration {
        max_line_length: 20,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "outer inner_long_long_long_long_long\nx\n");
}

#[test]
fn empty_softline_flat_is_nothing_break_is_newline() {
    let render_elements = vec![
        group(1, 6),
        RenderElement::TextStatic("["),
        RenderElement::SoftLine,
        RenderElement::TextStatic("a"),
        RenderElement::SoftLine,
        RenderElement::TextStatic("]"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "[a]\n");
}

#[test]
fn tab_indent_when_not_use_spaces() {
    let render_elements = vec![
        RenderElement::TextStatic("f:"),
        RenderElement::HardLine,
        indent(1, 3, 6),
        RenderElement::TextStatic("a"),
        RenderElement::HardLine,
        RenderElement::TextStatic("b"),
    ];
    let config = PrinterConfiguration {
        use_spaces: false,
        indent_size: 4,
        ..Default::default()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "f:\n\ta\n\tb\n");
}

#[test]
fn tab_counts_as_indent_size_in_width() {
    let render_elements = vec![
        group(1, 3),
        RenderElement::TextStatic("\ta"),
        RenderElement::TextStatic("b"),
    ];
    let config = PrinterConfiguration {
        max_line_length: 5,
        use_spaces: false,
        indent_size: 4,
        ..Default::default()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "\tab\n");
}

#[test]
fn raw_source_emitted_verbatim() {
    let source = "hello world";
    let render_elements = vec![RenderElement::UnformattedSource {
        range: RangeSourceBytes {
            start_byte: 0,
            end_byte: 11,
        },
    }];
    let mut out = String::new();
    render(
        &render_elements,
        source,
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "hello world\n");
}

#[test]
fn text_range_slices_source() {
    let source = "hello world";
    let render_elements = vec![text(0, 5), RenderElement::Space, text(6, 11)];
    let mut out = String::new();
    render(
        &render_elements,
        source,
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "hello world\n");
}

#[test]
fn trailing_newline_inserted_if_missing() {
    let render_elements = vec![RenderElement::TextStatic("a")];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a\n");
}

#[test]
fn empty_docs_produces_empty_output() {
    let render_elements: Vec<RenderElement> = vec![];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "");
}

#[test]
fn leading_hardlines_do_not_produce_indent() {
    let render_elements = vec![RenderElement::HardLine, RenderElement::TextStatic("a")];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "\na\n");
}

#[test]
fn leading_blank_line_then_content() {
    let render_elements = vec![RenderElement::BlankLine, RenderElement::TextStatic("a")];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "\na\n");
}

#[test]
fn multiline_raw_source_preserves_internal_newlines() {
    let source = "line1\nline2\nline3";
    let render_elements = vec![RenderElement::UnformattedSource {
        range: RangeSourceBytes {
            start_byte: 0,
            end_byte: 17,
        },
    }];
    let mut out = String::new();
    render(
        &render_elements,
        source,
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "line1\nline2\nline3\n");
}

#[test]
fn force_break_in_indent_breaks_enclosing_group() {
    let render_elements = vec![
        group(1, 6),
        RenderElement::TextStatic("a"),
        RenderElement::SoftLine,
        indent(1, 4, 6),
        RenderElement::ForceBreakingParent,
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a\n    b\n");
}

#[test]
fn if_break_outside_group_uses_flat() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        branch(
            Some(RangeRenderElement { start: 2, end: 3 }),
            Some(RangeRenderElement { start: 3, end: 4 }),
        ),
        RenderElement::TextStatic("flat"),
        RenderElement::TextStatic("break"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "aflat\n");
}

#[test]
fn blank_line_after_indent_preserves_indent_on_blank() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        RenderElement::HardLine,
        indent(1, 3, 6),
        RenderElement::TextStatic("b"),
        RenderElement::BlankLine,
        RenderElement::TextStatic("c"),
    ];
    let mut config = get_default_printer_configuration();
    config.indent_blank_lines = true;
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "a\n    b\n    \n    c\n");
}

#[test]
fn softline_in_break_mode_uses_current_indent_stack() {
    let render_elements = vec![
        group(1, 7),
        RenderElement::TextStatic("a"),
        RenderElement::SoftLine,
        indent(1, 4, 7),
        RenderElement::TextStatic("b"),
        RenderElement::SoftLine,
        RenderElement::TextStatic("c"),
    ];
    let config = PrinterConfiguration {
        max_line_length: 2,
        use_spaces: true,
        indent_size: 4,
        ..Default::default()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "a\n    b\n    c\n");
}

#[test]
fn force_break_in_inner_group_breaks_inner_only() {
    let render_elements = vec![
        group(1, 8),
        RenderElement::TextStatic("short"),
        RenderElement::SoftLine,
        group(4, 7),
        RenderElement::TextStatic("x"),
        RenderElement::ForceBreakingParent,
        RenderElement::SoftLine,
        RenderElement::TextStatic("y"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "shortx\ny\n");
}

#[test]
fn group_after_hardline_measures_from_indent_position() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        RenderElement::HardLine,
        indent(2, 3, 7),
        group(4, 7),
        RenderElement::TextStatic("xxx"),
        RenderElement::SoftLine,
        RenderElement::TextStatic("yyy"),
    ];
    let config = PrinterConfiguration {
        max_line_length: 10,
        use_spaces: true,
        indent_size: 4,
        ..Default::default()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "a\n        xxx\n        yyy\n");
}

#[test]
fn if_break_both_none_emits_nothing() {
    let render_elements = vec![
        group(1, 4),
        RenderElement::TextStatic("a"),
        branch(None, None),
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "ab\n");
}

#[test]
fn if_break_flat_some_break_none_emits_nothing_when_break() {
    let render_elements = vec![
        group(1, 4),
        RenderElement::TextStatic("aaaaaaa"),
        branch(Some(RangeRenderElement { start: 3, end: 4 }), None),
        RenderElement::TextStatic("onlyflat"),
    ];
    let config = PrinterConfiguration {
        max_line_length: 3,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "aaaaaaa\n");
}

#[test]
fn trailing_blank_line_is_trimmed_to_single_newline() {
    let render_elements = vec![RenderElement::TextStatic("a"), RenderElement::BlankLine];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a\n");
}

#[test]
fn consecutive_hardlines_collapse_to_one() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        RenderElement::HardLine,
        RenderElement::HardLine,
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a\nb\n");
}

#[test]
fn space_after_broken_softline_emits_leading_space() {
    let render_elements = vec![
        group(1, 5),
        RenderElement::TextStatic("a"),
        RenderElement::SoftLine,
        RenderElement::Space,
        RenderElement::TextStatic("b"),
    ];
    let config = PrinterConfiguration {
        max_line_length: 2,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "a\n b\n");
}

#[test]
fn text_empty_range_emits_nothing() {
    let source = "hello world";
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        text(5, 5),
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        source,
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "ab\n");
}

#[test]
fn single_blank_line_emits_one_blank_line_regardless_of_cap() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        RenderElement::BlankLine,
        RenderElement::TextStatic("b"),
    ];
    let config = PrinterConfiguration {
        maximum_blank_lines: 3,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "a\n\nb\n");
}

#[test]
fn two_blank_lines_with_default_cap_emit_two_blank_lines() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        RenderElement::BlankLine,
        RenderElement::BlankLine,
        RenderElement::TextStatic("b"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a\n\n\nb\n");
}

#[test]
fn raw_source_inside_indent_preserves_internal_indent() {
    let source = "x\n    y";
    let render_elements = vec![
        RenderElement::TextStatic("a:"),
        RenderElement::HardLine,
        indent(1, 3, 4),
        RenderElement::UnformattedSource {
            range: RangeSourceBytes {
                start_byte: 0,
                end_byte: 7,
            },
        },
    ];
    let mut out = String::new();
    render(
        &render_elements,
        source,
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "a:\n    x\n    y\n");
}

#[test]
fn multiline_raw_source_inside_group_forces_break() {
    let source = "x\ny";
    let render_elements = vec![
        group(1, 4),
        RenderElement::TextStatic("["),
        RenderElement::UnformattedSource {
            range: RangeSourceBytes {
                start_byte: 0,
                end_byte: 3,
            },
        },
        RenderElement::TextStatic("]"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        source,
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "[x\ny]\n");
}

#[test]
fn continuation_indent_level_doubles_indent() {
    // By default, continuation_indent_level is 2 meaning continuation lines get
    // indented twice (as per official style guide). The formatter outputs: body
    // indent(1) + continuation indent(2) wrapping the continued line. In total
    // indent level becomes 3 on the continuation line
    let render_elements = vec![
        RenderElement::TextStatic("func"),
        RenderElement::TextStatic(":"),
        RenderElement::HardLine,
        indent(1, 4, 8), // this indent covers the body
        RenderElement::TextStatic("pass"),
        RenderElement::HardLine,
        indent(2, 7, 8), // This indent covers just the continuation line
        RenderElement::TextStatic(".y()"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "func:\n    pass\n            .y()\n");
}

#[test]
fn continuation_indent_level_one_adds_single_extra_indent() {
    // Default continuation_indent_level is 2. When user sets it to 1,
    // the continuation indent is indent(1) instead of indent(2).
    // Total: 2 indent levels (body 1 + continuation 1).
    let render_elements = vec![
        RenderElement::TextStatic("func"),
        RenderElement::TextStatic(":"),
        RenderElement::HardLine,
        indent(1, 4, 8), // body indent: covers pass + continuation
        RenderElement::TextStatic("pass"),
        RenderElement::HardLine,
        indent(1, 7, 8), // continuation indent: user set to 1
        RenderElement::TextStatic(".y()"),
    ];
    let mut out = String::new();
    render(
        &render_elements,
        "",
        &get_default_printer_configuration(),
        &mut out,
    );
    assert_eq!(out, "func:\n    pass\n        .y()\n");
}

#[test]
fn trim_trailing_whitespace_on() {
    // TextStatic with trailing spaces followed by a HardLine.
    // When trim_trailing_whitespace is enabled, the trailing spaces
    // are stripped before the newline is emitted.
    let render_elements = vec![
        RenderElement::TextStatic("a  "),
        RenderElement::HardLine,
        RenderElement::TextStatic("b"),
    ];
    let config = PrinterConfiguration {
        trim_trailing_whitespace: true,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "a\nb\n");
}

#[test]
fn trim_trailing_whitespace_off() {
    let render_elements = vec![
        RenderElement::TextStatic("a  "),
        RenderElement::HardLine,
        RenderElement::TextStatic("b"),
    ];
    let config = PrinterConfiguration {
        trim_trailing_whitespace: false,
        ..get_default_printer_configuration()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "a  \nb\n");
}

#[test]
fn indent_blank_lines_with_tabs() {
    let render_elements = vec![
        RenderElement::TextStatic("a"),
        RenderElement::HardLine,
        indent(1, 3, 6),
        RenderElement::TextStatic("b"),
        RenderElement::BlankLine,
        RenderElement::TextStatic("c"),
    ];
    let config = PrinterConfiguration {
        indent_blank_lines: true,
        ..Default::default()
    };
    let mut out = String::new();
    render(&render_elements, "", &config, &mut out);
    assert_eq!(out, "a\n\tb\n\t\n\tc\n");
}
