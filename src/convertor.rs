//! Convert the  AST into an HTML string

use html_escape;
use crate::parser::Expression;

trait Callback: Fn(&str) -> (String, Option<String>) {}

impl<T: Fn(&str) -> (String, Option<String>)> Callback for T {}

// Store all the callbacks in a struct so we can pass it around easily during recursion
struct Callbacks<A, B, C, D> {
    emoji: A,
    user: B,
    role: C,
    channel: D,
}

// Generates HTML from the AST
fn traverse(ast: Vec<Expression>, callbacks: &Callbacks<impl Callback, impl Callback, impl Callback, impl Callback>, first: bool) -> String {
    // String to store the final HTML
    let mut final_html = String::new();
    // Wumboji
    let mut wumboji = " wumboji";
    // Don't do this if we've started recursion
    if first {
        // If there is any text other than whitespace, don't wumboji
        for expression in &ast {
            match expression {
                Expression::CustomEmoji(_, _) => {}
                Expression::Text(text) => {
                    if !text.chars().all(char::is_whitespace) {
                        wumboji = "";
                        break;
                    }
                }
                _ => {
                    wumboji = "";
                    break;
                }
            }
        }
    }
    for expression in ast {
        let html = match expression {
            Expression::Text(text) => format!("{}", html_escape::encode_text(&text.to_string()).to_string()), // Escape HTML
            Expression::CustomEmoji(name, id) => {
                // Use user-provided callback to get emoji path
                let path = (callbacks.emoji)(&id).0;
                format!("<img src=\"{0}\" alt=\"{1}\" class=\"emoji{2}\" title=\"{1}\"></img>", path, name, wumboji)
            }
            // Expression::Emoji(emoji) => format!("<span class=\"emoji{}\">{}</span>", wumboji, emoji),
            Expression::User(id) => format!("<span class=\"user\">@{}</span>", (callbacks.user)(id).0),
            Expression::Role(id) => {
                let (name, color) = (callbacks.role)(id);
                format!(
                    "<div class=\"role\" style=\"color: {0}\">@{1}<span style=\"background-color: {0}\"></span></div>",
                    color.unwrap_or(String::from("#afafaf")),
                    name,
                )
            },
            Expression::Channel(id) => format!("<span class=\"channel\" data-id=\"{}\">#{}</span>", id, (callbacks.channel)(id).0),
            Expression::Hyperlink(text, href) => format!("<a href=\"{}\" target=\"_blank\">{}</a>", href, text),
            Expression::MultilineCode(text) => format!("<pre class=\"multiline_code\">{}</pre>", text.trim().replace("\n", "<br>")),
            Expression::InlineCode(text) => format!("<span class=\"inline_code\">{}</span>", text.replace("\n", "<br>")),
            Expression::Blockquote(a) => format!("<blockquote>{}</blockquote>", traverse(a, callbacks, false)),
            Expression::Spoiler(a) => format!("<span class=\"spoiler\">{}</span>", traverse(a, callbacks, false)),
            Expression::Underline(a) => format!("<u>{}</u>", traverse(a, callbacks, false)),
            Expression::Strikethrough(a) => format!("<span class=\"strikethrough\">{}</span>", traverse(a, callbacks, false)),
            Expression::Bold(a) => format!("<strong>{}</strong>", traverse(a, callbacks, false)),
            Expression::Italics(a) => format!("<em>{}</em>", traverse(a, callbacks, false)),
            Expression::Newline => String::from("<br>"),
        };
        final_html.push_str(&html);
    }
    final_html
}

// Wrapper functions for traverse

/// Generates an HTML string from a vector of `Expression`s
///
/// Don't use this if your input string contains custom emoji or mentions, because the default
/// callback functions used here probably don't do what you want. Use `to_html_with_callbacks`
/// instead.
///
/// ```
/// use discord_markdown::{parser::Expression::*, convertor::to_html};
///
/// assert_eq!(
///     to_html(vec![
///         Text("lorem ipsum"),
///         Bold(vec![
///             Text("dolor sit"),
///             Underline(vec![Text("amet")]),
///         ]),
///     ]),
///     "lorem ipsum<strong>dolor sit<u>amet</u></strong>",
/// );
/// ```
pub fn to_html(ast: Vec<Expression>) -> String {
    traverse(ast, &Callbacks {
        emoji: |x: &str| (x.to_owned(), None),
        user: |x: &str| (x.to_owned(), None),
        role: |x: &str| (x.to_owned(), None),
        channel: |x: &str| (x.to_owned(), None),
    }, true)
}

/// Generates an HTML string from a vector of `Expression`s with callback functions for resolving
/// custom emoji and user, role, and channel mentions
///
/// The second value in the tuple is ignored for all callbacks except for `role`, so you can just
/// supply `None`.
///
/// **emoji callback:** the input is an `&str` with the emoji ID followed by either .png or .gif.
/// The first value of the output tuple must be the path to where the emoji is stored (used as
/// `src` attribute for `<img>` tag).
///
/// **user callback:** the input is an `&str` with the user ID of the user being mentioned. The
/// first value of the output tuple must be the name of the user.
///
/// **role callback:** the input is an `&str` with the role ID of the role being mentioned. The
/// first value of the output tuple must be the name of the role, and the second value the color of
/// the role. Giving `None` will use the default color of `#afafaf`.
///
/// **channel callback:** the input is an `&str` with the channel ID of the channel being linked.
/// The first value of the output tuple must be the name of the channel.
///
/// ```
/// use discord_markdown::{parser::Expression::*, convertor::to_html_with_callbacks};
///
/// let html = to_html_with_callbacks(
///     vec![
///         CustomEmoji("foo", String::from("777888999777888999.png")),
///         User("111222333111222333"),
///         Role("444555666444555666"),
///         Channel("333666999333666999"),
///     ],
///     |name| (format!("/emojis/{}", name), None),
///     |_| ("Jane Doe".to_owned(), None),
///     |_| ("green".to_owned(), Some("#00ff00".to_owned())),
///     |_| ("general".to_owned(), None),
/// );
///
/// let expected_output = "<img src=\"/emojis/777888999777888999.png\" alt=\"foo\" class=\"emoji\" title=\"foo\"></img><span class=\"user\">@Jane Doe</span><div class=\"role\" style=\"color: #00ff00\">@green<span style=\"background-color: #00ff00\"></span></div><span class=\"channel\" data-id=\"333666999333666999\">#general</span>";
///
/// assert_eq!(html, expected_output);
/// ```
pub fn to_html_with_callbacks(
    ast: Vec<Expression>,
    emoji: impl Fn(&str) -> (String, Option<String>),
    user: impl Fn(&str) -> (String, Option<String>),
    role: impl Fn(&str) -> (String, Option<String>),
    channel: impl Fn(&str) -> (String, Option<String>),
) -> String {
    traverse(ast, &Callbacks {
        emoji,
        user,
        role,
        channel,
    }, true)
}
