use chumsky::{
    error::Simple,
    primitive::{any, choice, end, filter, just, none_of},
    Parser,
};
use stylish_style::{Background, Color, Foreground, Intensity, Style, StyleDiff};

#[derive(Debug, Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, Clone)]
pub enum FormatArgRef {
    Positional(usize),
    Named(String),
}

#[derive(Debug, Clone)]
pub enum Count {
    Parameter(FormatArgRef),
    Integer(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum DebugHex {
    Lower,
    Upper,
}

#[derive(Debug, Default, Clone)]
pub struct FormatterArgs {
    pub align: Option<Align>,
    pub sign: Option<Sign>,
    pub alternate: bool,
    pub zero: bool,
    pub width: Option<Count>,
    pub precision: Option<Count>,
    pub debug_hex: Option<DebugHex>,
}

#[derive(Debug, Clone, Copy)]
enum Restyle {
    Fg(Color),
    Bg(Color),
    Intensity(Intensity),
}

#[derive(Debug, Clone, Copy)]
pub enum FormatTrait {
    Display,
    Debug,
    Octal,
    LowerHex,
    UpperHex,
    Pointer,
    Binary,
    LowerExp,
    UpperExp,
    Stylish,
}

impl Default for FormatTrait {
    fn default() -> Self {
        Self::Display
    }
}

#[derive(Debug, Default, Clone)]
pub struct FormatSpec {
    pub formatter_args: FormatterArgs,
    pub style: StyleDiff,
    pub format_trait: FormatTrait,
}

#[derive(Debug, Clone)]
pub struct FormatArg {
    pub arg: Option<FormatArgRef>,
    pub format_spec: FormatSpec,
}

#[derive(Debug, Clone)]
#[allow(variant_size_differences)]
pub enum Piece {
    Lit(String),
    Arg(FormatArg),
}

#[derive(Debug, Clone)]
pub struct Format {
    pub pieces: Vec<Piece>,
}

fn parser() -> impl Parser<char, Format, Error = Simple<char>> {
    let identifier = just('_')
        .or(filter(|&c| unicode_ident::is_xid_start(c)))
        .map(|c| vec![c])
        .chain::<char, _, _>(filter(|&c| unicode_ident::is_xid_continue(c)).repeated())
        .collect::<String>()
        .labelled("identifier");

    let color = choice((
        just("black").to(Color::Black),
        just("red").to(Color::Red),
        just("green").to(Color::Green),
        just("yellow").to(Color::Yellow),
        just("blue").to(Color::Blue),
        just("magenta").to(Color::Magenta),
        just("cyan").to(Color::Cyan),
        just("white").to(Color::White),
        just("default").to(Color::Default),
    ))
    .labelled("color")
    .boxed();

    let intensity = choice((
        just("normal").to(Intensity::Normal),
        just("bold").to(Intensity::Bold),
        just("faint").to(Intensity::Faint),
    ))
    .labelled("intensity");

    let restyles = choice((
        just("fg=")
            .ignore_then(color.clone())
            .map(Restyle::Fg)
            .labelled("fg"),
        just("bg=")
            .ignore_then(color)
            .map(Restyle::Bg)
            .labelled("bg"),
        intensity.map(Restyle::Intensity),
    ))
    .separated_by(just(','));

    let style_diff = restyles.try_map(|restyles, span| {
        struct Seen {
            fg: bool,
            bg: bool,
            intensity: bool,
        }
        let mut seen = Seen {
            fg: false,
            bg: false,
            intensity: false,
        };
        let mut style = Style::default();
        for restyle in restyles {
            match restyle {
                Restyle::Fg(color) => {
                    if seen.fg {
                        return Err(Simple::custom(span, "duplicate fg style"));
                    } else {
                        seen.fg = true;
                        style = style.with(Foreground(color));
                    }
                }
                Restyle::Bg(color) => {
                    if seen.bg {
                        return Err(Simple::custom(span, "duplicate bg style"));
                    } else {
                        seen.bg = true;
                        style = style.with(Background(color));
                    }
                }
                Restyle::Intensity(intensity) => {
                    if seen.intensity {
                        return Err(Simple::custom(span, "duplicate intensity style"));
                    } else {
                        seen.intensity = true;
                        style = style.with(intensity);
                    }
                }
            }
        }
        Ok(style.diff_from(Style::default()))
    });

    let align = choice((
        just('<').to(Align::Left),
        just('^').to(Align::Center),
        just('>').to(Align::Right),
    ))
    .labelled("align");

    let sign = choice((just('+').to(Sign::Plus), just('-').to(Sign::Minus))).labelled("sign");

    let number = chumsky::text::int(10)
        .from_str::<usize>()
        .unwrapped()
        .labelled("number");

    let format_arg_ref = choice((
        number.map(FormatArgRef::Positional),
        identifier.map(FormatArgRef::Named),
    ))
    .labelled("format_arg_ref");

    let count = choice((
        format_arg_ref.then_ignore(just('$')).map(Count::Parameter),
        number.map(Count::Integer),
    ))
    .labelled("count")
    .boxed();

    let format_spec = any()
        .then(align)
        .or(align.map(|align| (' ', align)))
        .try_map(|(fill, align), span| {
            if fill == ' ' {
                Ok((fill, align))
            } else {
                Err(Simple::custom(span, "non space fill not yet supported"))
            }
        })
        .or_not()
        .labelled("align and fill")
        .then(sign.or_not())
        .then(just('#').or_not().labelled("alternate"))
        .then(just('0').or_not().labelled("zero"))
        .then(count.clone().or_not().labelled("width"))
        .then(just('.').ignore_then(count).or_not().labelled("precision"))
        .then(
            style_diff
                .delimited_by(just('('), just(')'))
                .or_not()
                .labelled("style"),
        )
        .then(
            choice((
                just('?').to((None, FormatTrait::Debug)),
                just("x?").to((Some(DebugHex::Lower), FormatTrait::Debug)),
                just("X?").to((Some(DebugHex::Upper), FormatTrait::Debug)),
                just('o').to((None, FormatTrait::Octal)),
                just('x').to((None, FormatTrait::LowerHex)),
                just('X').to((None, FormatTrait::UpperHex)),
                just('p').to((None, FormatTrait::Pointer)),
                just('b').to((None, FormatTrait::Binary)),
                just('e').to((None, FormatTrait::LowerExp)),
                just('E').to((None, FormatTrait::UpperExp)),
                just('s').to((None, FormatTrait::Stylish)),
            ))
            .or_not(),
        )
        .map(
            |(
                ((((((fill_and_align, sign), alternate), zero), width), precision), style),
                debug_hex_and_format_trait,
            )| {
                let align = fill_and_align.map(|(_, align)| align);
                let debug_hex = debug_hex_and_format_trait.and_then(|(debug_hex, _)| debug_hex);
                let format_trait = debug_hex_and_format_trait.map(|(_, format_trait)| format_trait);
                FormatSpec {
                    formatter_args: FormatterArgs {
                        align,
                        sign,
                        alternate: alternate.is_some(),
                        zero: zero.is_some(),
                        width,
                        precision,
                        debug_hex,
                    },
                    style: style.unwrap_or_default(),
                    format_trait: format_trait.unwrap_or_default(),
                }
            },
        )
        .boxed();

    let format_arg = format_arg_ref
        .or_not()
        .then(just(':').ignore_then(format_spec).or_not())
        .map(|(arg, format_spec)| FormatArg {
            arg,
            format_spec: format_spec.unwrap_or_default(),
        });

    let literal = choice((
        none_of::<char, &str, Simple<char>>("{}"),
        just('{').then_ignore(just('{')),
        just('}').then_ignore(just('}')),
    ))
    .repeated()
    .at_least(1)
    .collect()
    .labelled("literal");

    let piece = choice((
        literal.map(Piece::Lit),
        format_arg
            .delimited_by(just('{'), just('}'))
            .map(Piece::Arg),
    ))
    .labelled("piece");

    piece
        .repeated()
        .map(|pieces| Format { pieces })
        .then_ignore(end())
}

impl Format {
    pub(crate) fn parse(s: &str) -> Result<Self, Vec<Simple<char>>> {
        parser().parse(s)
    }
}
