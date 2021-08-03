//! This parser was written for use with [cheesecake](https://github.com/cubetastic33/cheesecake),
//! so the convertor function provided is designed for that. If this function doesn't suit your
//! use-case, you can write your own convertor function to generate HTML from the parsed AST. The
//! text in the generated HTML will be HTML-escaped, so you can safely insert the output into the
//! DOM.
//!
//! # Usage
//! Call `parser::parse` on the input string, and it will return a vector of `Expression`s. Supply
//! this vector to `convertor::to_html` to get an HTML string. If your input text will also have
//! custom emoji, user mentions, role mentions, or channel mentions, then use
//! `convertor::to_html_with_callbacks` instead.
//!
//! Call `parser::parse_with_md_hyperlinks` instead if you want to also parse links with alt text,
//! which is supported in discord embeds (Like `[example](https://example.com)`)
//!
//! ### Note:
//! Newlines are not converted to `Expression::Newline` inside code blocks, so that must be handled
//! in the covertor.
//!
//! # Examples
//!
//! If all you want is to generate the AST, it's really simple:
//! ```
//! use discord_markdown::parser::{parse, Expression::*};
//!
//! fn main() {
//!     let ast = parse("> _**example** formatted_ ||string||");
//!     assert_eq!(ast, vec![
//!         Blockquote(vec![
//!             Italics(vec![
//!                 Bold(vec![Text("example")]),
//!                 Text(" formatted"),
//!             ]),
//!             Text(" "),
//!             Spoiler(vec![Text("string")]),
//!         ]),
//!     ]);
//! }
//! ```
//!
//! If you want to generate an HTML string, it's like this:
//! ```
//! use discord_markdown::{parser::parse, convertor::*};
//!
//! fn dummy_callback(x: &str) -> (String, Option<String>) {
//!     (x.to_owned(), None)
//! }
//!
//! fn id_to_name(id: &str) -> (String, Option<String>) {
//!     (
//!         if id == "123456789123456789" {"member"} else {"unknown role"}.to_owned(),
//!         Some("#ff0000".to_owned()),
//!     )
//! }
//!
//! fn main() {
//!     let html = to_html(parse("> _**example** formatted_ ||string||"));
//!     assert_eq!(html, "<blockquote><em><strong>example</strong> formatted\
//!     </em> <span class=\"spoiler\">string</span></blockquote>");
//!
//!     // With role mentions
//!     let html = to_html_with_callbacks(
//!         parse("<@&123456789123456789>"),
//!         dummy_callback,
//!         dummy_callback,
//!         id_to_name,
//!         dummy_callback,
//!     );
//!     assert_eq!(html, "<div class=\"role\" style=\"color: #ff0000\">@member\
//!     <span style=\"background-color: #ff0000\"></span></div>");
//! }
//! ```
//!
//! You can then add styling for `.role` and `.role span` in your stylesheet. Here's some example
//! CSS:
//! ```css
//! .role {
//!     background-color: initial;
//!     display: inline-block;
//!     position: relative;
//!     word-break: keep-all;
//! }
//!
//! .role span {
//!     border-radius: 4px;
//!     height: 100%;
//!     width: 100%;
//!     opacity: .12;
//!     position: absolute;
//!     left: 0;
//!     top: 0;
//! }
//! ```

pub mod parser;
pub mod convertor;

#[cfg(test)]
mod tests {
    use super::parser::{parse, Expression::*};

    #[test]
    fn parser_basic() {
        // Tests the basic parsers
        assert_eq!(parse("_foo_ *bar*"), vec![
            Italics(vec![Text("foo")]), Text(" "), Italics(vec![Text("bar")])
        ]);
        assert_eq!(parse("**foo bar**"), vec![Bold(vec![Text("foo bar")])]);
        assert_eq!(parse("__foo bar__"), vec![Underline(vec![Text("foo bar")])]);
        assert_eq!(parse("~~foo bar~~"), vec![Strikethrough(vec![Text("foo bar")])]);
        assert_eq!(parse("||foo bar||"), vec![Spoiler(vec![Text("foo bar")])]);
        assert_eq!(parse("`foo` ``foo ` bar``"), vec![
            InlineCode("foo"), Text(" "), InlineCode("foo ` bar")
        ]);
        assert_eq!(parse("```foo\nbar```"), vec![MultilineCode("foo\nbar")]);
        assert_eq!(parse("> foo bar"), vec![Blockquote(vec![Text("foo bar")])]);
    }

    #[test]
    fn parse_regex() {
        // Tests the parsers that use regex
        assert_eq!(parse("<#123456789123456789>"), vec![Channel("123456789123456789")]);
        assert_eq!(parse("<@123456789123456789><@!123456789123456789>"), vec![
            User("123456789123456789"),
            User("123456789123456789"),
        ]);
        assert_eq!(parse("<@&123456789123456789>"), vec![Role("123456789123456789")]);
        assert_eq!(parse("<a:foo:123456789123456789><:foo:123456789123456789>"), vec![
            CustomEmoji("foo", "123456789123456789.gif".to_owned()),
            CustomEmoji("foo", "123456789123456789.png".to_owned()),
        ]);
    }
}
