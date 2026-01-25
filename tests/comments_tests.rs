use remix::comments::remove_comments;

#[test]
fn test_remove_comments_rust() {
    let code = r#"
// This is a comment
fn main() {
    println!("Hello"); // inline comment
    /* block comment */
}
"#;

    let expected = "\n\nfn main() {\n    println!(\"Hello\"); \n    \n}\n";

    assert_eq!(remove_comments(code, "rs"), expected);
}

#[test]
fn test_remove_comments_python() {
    let code = r#"
# This is a comment
def main():
    print("Hello")  # inline comment
    """
    Multi-line string (not comment)
    """
    pass
"#;

    let expected = "\n\ndef main():\n    print(\"Hello\")  \n    \"\"\"\n    Multi-line string (not comment)\n    \"\"\"\n    pass\n";

    assert_eq!(remove_comments(code, "py"), expected);
}

#[test]
fn test_remove_comments_javascript() {
    let code = r#"
// Single line comment
function test() {
    console.log("Hello"); // inline
    /* block comment */
}
/* another block */
"#;

    let expected = "\n\nfunction test() {\n    console.log(\"Hello\"); \n    \n}\n\n";

    assert_eq!(remove_comments(code, "js"), expected);
}

#[test]
fn test_remove_comments_c_style() {
    let code = r#"
// Comment
int main() {
    printf("Hello"); // inline
    /* block */
}
/* end */
"#;

    let expected = "\n\nint main() {\n    printf(\"Hello\"); \n    \n}\n\n";

    assert_eq!(remove_comments(code, "c"), expected);
}

#[test]
fn test_remove_comments_html() {
    let html = r#"
<!-- comment -->
<div>Hello</div>
<!-- another comment -->
"#;

    let expected = "\n\n<div>Hello</div>\n\n";

    assert_eq!(remove_comments(html, "html"), expected);
}

#[test]
fn test_remove_comments_css() {
    let css = r#"
/* comment */
.class {
    color: red; /* inline */
}
/* another */
"#;

    let expected = "\n\n.class {\n    color: red; \n}\n\n";

    assert_eq!(remove_comments(css, "css"), expected);
}

#[test]
fn test_remove_comments_unsupported() {
    let code = "// comment\ncode";
    assert_eq!(remove_comments(code, "txt"), "// comment\ncode");
}

#[test]
fn test_remove_comments_edge_cases() {
    // Empty input
    assert_eq!(remove_comments("", "rs"), "");

    // No comments
    let code = "fn main() {}\n";
    assert_eq!(remove_comments(code, "rs"), code);

    // Only comments
    assert_eq!(remove_comments("// comment\n/* block */", "rs"), "\n");
}
