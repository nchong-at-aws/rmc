use rustc_lint_defs::declare_tool_lint;

macro_rules! declare_rustdoc_lint {
    ($(#[$attr:meta])* $name: ident, $level: ident, $descr: literal $(,)?) => {
        declare_tool_lint! {
            $(#[$attr])* pub(crate) rustdoc::$name, $level, $descr
        }
    }
}

declare_rustdoc_lint! {
    /// The `broken_intra_doc_links` lint detects failures in resolving
    /// intra-doc link targets. This is a `rustdoc` only lint, see the
    /// documentation in the [rustdoc book].
    ///
    /// [rustdoc book]: ../../../rustdoc/lints.html#broken_intra_doc_links
    BROKEN_INTRA_DOC_LINKS,
    Warn,
    "failures in resolving intra-doc link targets"
}

declare_rustdoc_lint! {
    /// This is a subset of `broken_intra_doc_links` that warns when linking from
    /// a public item to a private one. This is a `rustdoc` only lint, see the
    /// documentation in the [rustdoc book].
    ///
    /// [rustdoc book]: ../../../rustdoc/lints.html#private_intra_doc_links
    PRIVATE_INTRA_DOC_LINKS,
    Warn,
    "linking from a public item to a private one"
}

declare_rustdoc_lint! {
    /// The `invalid_codeblock_attributes` lint detects code block attributes
    /// in documentation examples that have potentially mis-typed values. This
    /// is a `rustdoc` only lint, see the documentation in the [rustdoc book].
    ///
    /// [rustdoc book]: ../../../rustdoc/lints.html#invalid_codeblock_attributes
    INVALID_CODEBLOCK_ATTRIBUTES,
    Warn,
    "codeblock attribute looks a lot like a known one"
}

declare_rustdoc_lint! {
    /// The `missing_crate_level_docs` lint detects if documentation is
    /// missing at the crate root. This is a `rustdoc` only lint, see the
    /// documentation in the [rustdoc book].
    ///
    /// [rustdoc book]: ../../../rustdoc/lints.html#missing_crate_level_docs
    MISSING_CRATE_LEVEL_DOCS,
    Allow,
    "detects crates with no crate-level documentation"
}

declare_rustdoc_lint! {
    /// The `missing_doc_code_examples` lint detects publicly-exported items
    /// without code samples in their documentation. This is a `rustdoc` only
    /// lint, see the documentation in the [rustdoc book].
    ///
    /// [rustdoc book]: ../../../rustdoc/lints.html#missing_doc_code_examples
    MISSING_DOC_CODE_EXAMPLES,
    Allow,
    "detects publicly-exported items without code samples in their documentation"
}

declare_rustdoc_lint! {
    /// The `private_doc_tests` lint detects code samples in docs of private
    /// items not documented by `rustdoc`. This is a `rustdoc` only lint, see
    /// the documentation in the [rustdoc book].
    ///
    /// [rustdoc book]: ../../../rustdoc/lints.html#private_doc_tests
    PRIVATE_DOC_TESTS,
    Allow,
    "detects code samples in docs of private items not documented by rustdoc"
}

declare_rustdoc_lint! {
    /// The `invalid_html_tags` lint detects invalid HTML tags. This is a
    /// `rustdoc` only lint, see the documentation in the [rustdoc book].
    ///
    /// [rustdoc book]: ../../../rustdoc/lints.html#invalid_html_tags
    INVALID_HTML_TAGS,
    Allow,
    "detects invalid HTML tags in doc comments"
}

declare_rustdoc_lint! {
    /// The `bare_urls` lint detects when a URL is not a hyperlink.
    /// This is a `rustdoc` only lint, see the documentation in the [rustdoc book].
    ///
    /// [rustdoc book]: ../../../rustdoc/lints.html#bare_urls
    BARE_URLS,
    Warn,
    "detects URLs that are not hyperlinks"
}

declare_rustdoc_lint! {
   /// The `invalid_rust_codeblocks` lint detects Rust code blocks in
   /// documentation examples that are invalid (e.g. empty, not parsable as
   /// Rust code). This is a `rustdoc` only lint, see the documentation in the
   /// [rustdoc book].
   ///
   /// [rustdoc book]: ../../../rustdoc/lints.html#invalid_rust_codeblocks
   INVALID_RUST_CODEBLOCKS,
   Warn,
   "codeblock could not be parsed as valid Rust or is empty"
}
