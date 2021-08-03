//! Parse Discord MarkDown into an AST

use nom::{IResult, Slice, branch::alt, bytes::complete::{is_not, tag, take_until}, combinator::{cond, map_opt, map_parser, recognize}, regex::Regex, sequence::{delimited, pair, preceded, terminated}};
use lazy_static::lazy_static;

/// Enum to represent the AST
#[derive(Debug, PartialEq)]
pub enum Expression<'a> {
    Text(&'a str),
    CustomEmoji(&'a str, String),
    User(&'a str),
    Role(&'a str),
    Channel(&'a str),
    Hyperlink(&'a str, &'a str),
    MultilineCode(&'a str),
    InlineCode(&'a str),
    Blockquote(Vec<Expression<'a>>),
    Spoiler(Vec<Expression<'a>>),
    Underline(Vec<Expression<'a>>),
    Strikethrough(Vec<Expression<'a>>),
    Bold(Vec<Expression<'a>>),
    Italics(Vec<Expression<'a>>),
    Newline,
}

lazy_static! {
    static ref CUSTOM_EMOJI_RE: Regex = Regex::new(r"^<(a?):(\w+):(\d+)(>)").unwrap();
    static ref USER_RE: Regex = Regex::new(r"^<@!?(\d+)(>)").unwrap();
    static ref ROLE_RE: Regex = Regex::new(r"^<@&(\d+)(>)").unwrap();
    static ref CHANNEL_RE: Regex = Regex::new(r"^<#(\d+)(>)").unwrap();
    static ref LINK_RE: Regex = Regex::new(r"^(https?|ftp|file)(://[-A-Za-z0-9+&@#/%?=~_|!:,.;]*[A-Za-z0-9+&@#/%=~_|])").unwrap();
}

// Re-implement re_capture from nom, but make it take &'a Regex instead of Regex
// This provides a noticeable speed improvement since we don't have to RE.clone() each time
fn re_capture<'a, E>(re: &'a Regex) -> impl Fn(&'a str) -> IResult<&'a str, Vec<&'a str>, E>
    where
        E: nom::error::ParseError<&'a str>,
{
    move |i| {
        if let Some(c) = re.captures(i) {
            let v: Vec<_> = c
                .iter()
                .filter(|el| el.is_some())
                .map(|el| el.unwrap())
                .map(|m| i.slice(m.start()..m.end()))
                .collect();
            let offset = {
                let end = v.last().unwrap();
                end.as_ptr() as usize + end.len() - i.as_ptr() as usize
            };
            Ok((i.slice(offset..), v))
        } else {
            Err(nom::Err::Error(E::from_error_kind(i, nom::error::ErrorKind::RegexpCapture)))
        }
    }
}

// Parses custom emoji
fn custom_emoji<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, custom_emoji) = re_capture(&CUSTOM_EMOJI_RE)(input)?;
    let extension = if custom_emoji[1] == "a" { "gif" } else { "png" };
    Ok((input, Expression::CustomEmoji(custom_emoji[2], format!("{}.{}", custom_emoji[3], extension))))
}

// Parses user mentions
fn user<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, user) = re_capture(&USER_RE)(input)?;
    Ok((input, Expression::User(user[1])))
}

// Parses role mentions
fn role<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, role) = re_capture(&ROLE_RE)(input)?;
    Ok((input, Expression::Role(role[1])))
}

// Parses channel links
fn channel<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, channel) = re_capture(&CHANNEL_RE)(input)?;
    Ok((input, Expression::Channel(channel[1])))
}

fn hyperlink_internals(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, hyperlink) = alt((
        re_capture(&LINK_RE),
        delimited(tag("<"), re_capture(&LINK_RE), tag(">")),
    ))(input)?;
    Ok((input, (hyperlink[0], hyperlink[0])))
}

// Parses hyperlinks
fn hyperlink<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, hyperlink) = hyperlink_internals(input)?;
    Ok((input, Expression::Hyperlink(hyperlink.0, hyperlink.1)))
}

// Parses hyperlinks with support for alt text
fn md_hyperlink<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, hyperlink) = alt((
        hyperlink_internals,
        pair(
            delimited(tag("["), take_until("]"), tag("]")),
            delimited(tag("("), |input| {
                let x = hyperlink_internals(input)?;
                Ok((x.0, x.1.0))
            }, tag(")"))
        ),
    ))(input)?;
    Ok((input, Expression::Hyperlink(hyperlink.0, hyperlink.1)))
}

fn multiline_code<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, multiline_code) = delimited(tag("```"), take_until("```"), tag("```"))(input)?;
    Ok((input, Expression::MultilineCode(multiline_code)))
}

fn inline_code<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, inline_code) = alt((
        // If the inline code block is delimited by ``
        delimited(tag("``"), take_until("``"), tag("``")),
        // If the inline code block is delimited by `
        delimited(tag("`"), is_not("`"), tag("`")),
    ))(input)?;
    Ok((input, Expression::InlineCode(inline_code)))
}

fn blockquote<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, blockquote) = map_parser(alt((
        // Blockquote until end of line
        delimited(tag("> "), is_not("\n"), tag("\n")),
        // Special case for `> \n`
        preceded(tag("> "), tag("\n")),
        // Blockquote until end of file
        preceded(tag("> "), is_not("\n")),
    )), parse_section)(input)?;
    Ok((input, Expression::Blockquote(blockquote)))
}

fn spoiler<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, spoiler) = map_parser(
        delimited(tag("||"), take_until("||"), tag("||")),
        parse_section,
    )(input)?;
    Ok((input, Expression::Spoiler(spoiler)))
}

fn underline<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, underline) = map_parser(
        alt((
            // Special case with four surrounding underlines
            delimited(tag("____"), take_until("____"), tag("____")),
            // Special case with three surrounding underlines
            delimited(
                tag("__"),
                recognize(delimited(tag("_"), take_until("___"), tag("_"))),
                tag("__"),
            ),
            // Special case with three underscores at the end alone
            delimited(
                tag("__"),
                recognize(terminated(take_until("___"), tag("_"))),
                tag("__"),
            ),
            delimited(tag("__"), take_until("__"), tag("__")),
        )),
        parse_section,
    )(input)?;
    Ok((input, Expression::Underline(underline)))
}

fn strikethrough<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, strikethrough) = map_parser(
        delimited(tag("~~"), take_until("~~"), tag("~~")),
        parse_section,
    )(input)?;
    Ok((input, Expression::Strikethrough(strikethrough)))
}

fn bold<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, bold) = map_parser(
        alt((
            // Special case with four surrounding asterisks
            delimited(tag("****"), take_until("****"), tag("****")),
            // Special case with three surrounding asterisks
            delimited(
                tag("**"),
                recognize(delimited(tag("*"), take_until("***"), tag("*"))),
                tag("**"),
            ),
            // Special case with three asterisks at the end alone
            delimited(
                tag("**"),
                recognize(terminated(take_until("***"), tag("*"))),
                tag("**"),
            ),
            delimited(tag("**"), take_until("**"), tag("**")),
        )),
        parse_section,
    )(input)?;
    Ok((input, Expression::Bold(bold)))
}

fn italics<'a>(input: &'a str) -> IResult<&str, Expression<'a>> {
    let (input, italics) = map_parser(
        alt((
            delimited(tag("_"), is_not("_"), tag("_")),
            delimited(tag("*"), is_not("*"), tag("*")),
        )),
        parse_section,
    )(input)?;
    Ok((input, Expression::Italics(italics)))
}

fn apply_parsers(
    allow_blockquote: bool,
    md_hyperlinks: bool,
    input: &str,
) -> IResult<&str, Expression> {
    alt((
        map_opt(cond(allow_blockquote, blockquote), |o| o),
        custom_emoji,
        user,
        role,
        channel,
        if md_hyperlinks {md_hyperlink} else {hyperlink},
        multiline_code,
        inline_code,
        spoiler,
        underline,
        strikethrough,
        bold,
        italics,
    ))(input)
}

fn parse_internals<'a>(
    mut input: &'a str,
    mut allow_blockquote: bool,
    md_hyperlinks: bool,
) -> IResult<&str, Vec<Expression<'a>>> {
    // Attempt to parse everything until we encounter a newline/end of input
    let mut result = Vec::new();

    'outer: while input.len() != 0 {
        for (i, c) in input.char_indices() {
            if c == '\n' {
                // If it's a newline, we can parse blockquotes starting from the next character
                if i > 0 {
                    result.push(Expression::Text(&input[..i]))
                }
                result.push(Expression::Newline);
                allow_blockquote = true;
                // Remove the parsed part from `input` and restart the for loop
                // We can safely do i + 1 because the input can't end with \n (it's stripped)
                input = &input[i + 1..];
                continue 'outer;
            } else if c == '¯' && input[i..].starts_with(r"¯\_(ツ)_/¯") {
                // Parse shrug emote
                if i > 0 {
                    result.push(Expression::Text(&input[..i]))
                }
                // Push the shrug emote as Expression::Text
                result.push(Expression::Text(r"¯\_(ツ)_/¯"));
                // Remove the parsed part from `input` and restart the for loop
                input = &input[i + r"¯\_(ツ)_/¯".len()..];
                continue 'outer;
            } else if c == '\\' && input[i..].len() > 1 {
                // If it's a backslash, we should escape the following character
                if i > 0 {
                    result.push(Expression::Text(&input[..i]))
                }
                // Push the escaped character as Expression::Text
                let (char_pos, c) = input.char_indices().nth(i + 1).unwrap();
                result.push(Expression::Text(&input[char_pos..char_pos + c.len_utf8()]));
                // Remove the parsed part from `input` and restart the for loop
                input = &input[char_pos + c.len_utf8()..];
                continue 'outer;
            }
            if let Ok((remaining, expr)) = apply_parsers(allow_blockquote, md_hyperlinks, &input[i..]) {
                // Don't reset blockquote if we just matched on a blockquote because it consumes a
                // succeeding newline if it exists, and if it doesn't, `allow_blockquote` doesn't
                // matter anyway
                if !matches!(expr, Expression::Blockquote(_)) {
                    // Reset allow_blockquote because we're not immediately after a newline
                    allow_blockquote = false;
                }
                // Add the text up to the parsed expression as Expression::Text
                if i > 0 {
                    result.push(Expression::Text(&input[..i]))
                }
                // Add the parsed expression
                result.push(expr);
                // Remove the parsed part from `input` and restart the for loop
                input = remaining;
                continue 'outer;
            } else {
                allow_blockquote = false;
            }
        }
        if input.len() != 0 {
            result.push(Expression::Text(input));
            input = "";
        }
    }

    Ok((input, result))
}

fn parse_section<'a>(mut input: &'a str) -> IResult<&str, Vec<Expression<'a>>> {
    parse_internals(&mut input, false, false)
}

/// Parses the given input string as Discord MarkDown and returns a vector of `Expression`s
///
/// ```
/// use discord_markdown::parser::{parse, Expression::*};
///
/// let ast = parse(
///     "> Can someone link the rust website?\n<@123456789123456789> https://www.rust-lang.org"
/// );
///
/// assert_eq!(ast, vec![
///     Blockquote(vec![Text("Can someone link the rust website?")]),
///     User("123456789123456789"),
///     Text(" "),
///     Hyperlink("https://www.rust-lang.org", "https://www.rust-lang.org"),
/// ]);
/// ```
pub fn parse(mut input: &str) -> Vec<Expression> {
    parse_internals(&mut input, true, false).unwrap().1
}

/// Parses the given input string as Discord MarkDown with support for hyperlinks with alt text
/// (used in discord embeds) and returns a vector of `Expression`s
///
/// ```
/// use discord_markdown::parser::{parse_with_md_hyperlinks, Expression::*};
///
/// let ast = parse_with_md_hyperlinks("_link_: [example](https://example.com)");
/// assert_eq!(ast, vec![
///     Italics(vec![Text("link")]),
///     Text(": "),
///     Hyperlink("example", "https://example.com"),
/// ]);
/// ```
pub fn parse_with_md_hyperlinks(mut input: &str) -> Vec<Expression> {
    parse_internals(&mut input, true, true).unwrap().1
}
