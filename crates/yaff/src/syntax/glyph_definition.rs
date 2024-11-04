use std::str::FromStr;
use winnow::{
    ascii::{digit1, hex_digit1, oct_digit1},
    combinator::{alt, cut_err, opt, preceded, repeat, separated, separated_foldl1, terminated},
    error::StrContext,
    seq,
    token::{any, none_of},
    PResult, Parser,
};

use crate::{
    syntax::fragments::parse_line_terminator, GlyphDefinition, GlyphLabel, GlyphPaletteColor,
    GlyphValue,
};

use super::fragments::parse_whitespace;

pub fn parse_glyph_definition(input: &mut &str) -> PResult<GlyphDefinition> {
    seq!(GlyphDefinition {
        labels: repeat(
            1..,
            terminated(
                parse_glyph_label,
                (
                    opt(parse_whitespace),
                    ':',
                    opt(parse_whitespace),
                    parse_line_terminator,
                )
            )
        ),
        indent: cut_err(parse_whitespace),
        value: create_glyph_value_parser(&indent),
    })
    .parse_next(input)
}

pub fn parse_glyph_label(input: &mut &str) -> PResult<GlyphLabel> {
    alt((
        parse_codepoint_label,
        parse_character_label,
        parse_tag_label,
    ))
    .parse_next(input)
}

fn parse_codepoint_label(input: &mut &str) -> PResult<GlyphLabel> {
    separated(
        1..,
        alt((
            preceded(
                alt(("0x", "0X")),
                hex_digit1.try_map(|s| u32::from_str_radix(s, 16)),
            ),
            preceded(
                alt(("0o", "0O")),
                oct_digit1.try_map(|s| u32::from_str_radix(s, 8)),
            ),
            digit1.try_map(u32::from_str),
        )),
        comma_separator,
    )
    .try_map(GlyphLabel::try_from_codepoint)
    .parse_next(input)
}

fn parse_character_label(input: &mut &str) -> PResult<GlyphLabel> {
    separated_foldl1(
        alt((
            preceded(alt(("u+", "U+")), hex_digit1)
                .try_map(|s| u32::from_str_radix(s, 16))
                .try_map(char::try_from)
                .map(|c| vec![c]),
            preceded(
                '\'',
                repeat(
                    1..,
                    terminated(
                        repeat(0.., none_of(('\n', '\r', '\'')))
                            .map(|acc: Vec<_>| -> String { acc.into_iter().collect() })
                            .context(StrContext::Label("test")),
                        '\'',
                    ),
                )
                .map(|acc: Vec<_>| acc.join("'").chars().collect()),
            ),
        )),
        comma_separator,
        |mut v1, _, mut v2| {
            v1.append(&mut v2);
            v1
        },
    )
    .map(GlyphLabel::from_character)
    .parse_next(input)
}

fn parse_tag_label(input: &mut &str) -> PResult<GlyphLabel> {
    preceded(
        '"',
        repeat(
            1..,
            terminated(
                repeat(0.., none_of(('\n', '\r', '\"')))
                    .map(|acc: Vec<_>| -> String { acc.into_iter().collect() }),
                '"',
            ),
        )
        .map(|acc: Vec<_>| acc.join("\"")),
    )
    .map(|_| todo!())
    .parse_next(input)
}

fn create_glyph_value_parser<'a>(
    indent: &'a str,
) -> impl Fn(&mut &str) -> PResult<Option<GlyphValue>> + 'a {
    move |input| {
        alt((
            ('-', opt(parse_whitespace), parse_line_terminator).map(|_| None),
            separated(
                1..,
                parse_glyph_row,
                (opt(parse_whitespace), parse_line_terminator, indent),
            )
            .try_map(|acc: Vec<_>| GlyphValue::new(acc).map(Some)),
        ))
        .parse_next(input)
    }
}

fn parse_glyph_row(input: &mut &str) -> PResult<Vec<Option<GlyphPaletteColor>>> {
    repeat(
        1..,
        preceded(opt(' '), any.try_map(|ch| GlyphPaletteColor::try_from(ch))),
    )
    .parse_next(input)
}

fn comma_separator(input: &mut &str) -> PResult<()> {
    (opt(parse_whitespace), ',', opt(parse_whitespace))
        .void()
        .parse_next(input)
}
