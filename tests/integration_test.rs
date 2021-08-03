use discord_markdown::{convertor::*, parser::{parse, parse_with_md_hyperlinks}};

#[test]
fn convertor_basic() {
    assert_eq!(to_html(
        parse("foo > _foo bar_ *foo bar* **foo bar** __foo bar__\n> `foo bar` ``foo bar`` ||foo bar||\n> \n> test"),
    ), "foo &gt; <em>foo bar</em> <em>foo bar</em> <strong>foo bar</strong> <u>foo bar</u><br><blockquote><span class=\"inline_code\">foo bar</span> <span class=\"inline_code\">foo bar</span> <span class=\"spoiler\">foo bar</span></blockquote><blockquote><br></blockquote><blockquote>test</blockquote>");
}

#[test]
fn convertor_nested() {
    assert_eq!(to_html(
        parse("foo _> foo_ _bar\n> foo_ ||***__~~foo\nbar~~__***||"),
    ), "foo <em>&gt; foo</em> <em>bar<br><blockquote>foo</blockquote></em> <span class=\"spoiler\"><strong><em><u><span class=\"strikethrough\">foo<br>bar</span></u></em></strong></span>");
}

#[test]
fn convertor_regex() {
    assert_eq!(to_html_with_callbacks(
        parse("<#1234567890><@&1234567890><@1234567890><@!1234567890><:foo:1234567890><a:foo:1234567890>"),
        |filename| (filename.to_string(), None),
        |id| (id.to_string(), None),
        |x| (x.to_string(), Some(String::from("#ff00ff"))),
        |id| (id.to_string(), None),
    ), "<span class=\"channel\" data-id=\"1234567890\">#1234567890</span><div class=\"role\" style=\"color: #ff00ff\">@1234567890<span style=\"background-color: #ff00ff\"></span></div><span class=\"user\">@1234567890</span><span class=\"user\">@1234567890</span><img src=\"1234567890.png\" alt=\"foo\" class=\"emoji\" title=\"foo\"></img><img src=\"1234567890.gif\" alt=\"foo\" class=\"emoji\" title=\"foo\"></img>");
}

#[test]
fn convertor_hyperlinks() {
    assert_eq!(to_html(
        parse("<https://www.example.com/> https://example.com [foo](https://example.com/) [foo](<http://example.com>)"),
    ), "<a href=\"https://www.example.com/\" target=\"_blank\">https://www.example.com/</a> <a href=\"https://example.com\" target=\"_blank\">https://example.com</a> [foo](<a href=\"https://example.com/\" target=\"_blank\">https://example.com/</a>) [foo](<a href=\"http://example.com\" target=\"_blank\">http://example.com</a>)");
    assert_eq!(to_html(
        parse_with_md_hyperlinks("<https://www.example.com/> https://example.com [foo](https://example.com/) [foo](<http://example.com>)"),
    ), "<a href=\"https://www.example.com/\" target=\"_blank\">https://www.example.com/</a> <a href=\"https://example.com\" target=\"_blank\">https://example.com</a> <a href=\"https://example.com/\" target=\"_blank\">foo</a> <a href=\"http://example.com\" target=\"_blank\">foo</a>");
}
