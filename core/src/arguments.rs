use crate::{formatter::FormatterArgs, Display, Formatter, Result, StyleDiff};

stackbox::custom_dyn! {
    pub dyn StdFmtFn: Fn(&mut core::fmt::Formatter<'_>) -> Result {
        fn call(self: &Self, arg: &mut core::fmt::Formatter<'_>) -> Result {
            self(arg)
        }
    }
}

impl core::fmt::Display for StackBoxDynStdFmtFn<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result {
        self.call(f)
    }
}

impl core::fmt::Debug for StackBoxDynStdFmtFn<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result {
        self.call(f)
    }
}

#[doc(hidden)] // workaround https://github.com/rust-lang/rust/issues/85522
#[allow(missing_debug_implementations)]
pub enum FormatTrait<'a> {
    Display(StackBoxDynStdFmtFn<'a>),
    Debug(StackBoxDynStdFmtFn<'a>),
    Octal(StackBoxDynStdFmtFn<'a>),
    LowerHex(StackBoxDynStdFmtFn<'a>),
    UpperHex(StackBoxDynStdFmtFn<'a>),
    Pointer(StackBoxDynStdFmtFn<'a>),
    Binary(StackBoxDynStdFmtFn<'a>),
    LowerExp(StackBoxDynStdFmtFn<'a>),
    UpperExp(StackBoxDynStdFmtFn<'a>),
    Stylish(&'a dyn Display),
}

#[doc(hidden)] // workaround https://github.com/rust-lang/rust/issues/85522
#[allow(missing_debug_implementations)]
pub enum Argument<'a> {
    Lit(&'a str),

    Arg {
        args: &'a FormatterArgs<'a>,
        style: StyleDiff,
        arg: FormatTrait<'a>,
    },
}

/// A precompiled version of a format string and its by-reference arguments.
///
/// Currently this can only be constructed via [`stylish::format_args!`], but it
/// may be possible to dynamically construct this at runtime in the future.
///
/// ```rust
/// let args = stylish::format_args!("{:(bg=red)} Will Robinson", "Danger");
/// assert_eq!(
///     stylish::html::format!("{:s}", args),
///     "<span style=background-color:red>Danger</span> Will Robinson",
/// );
/// ```
#[allow(missing_debug_implementations)]
pub struct Arguments<'a> {
    #[doc(hidden)] // pub for macros
    pub pieces: &'a [Argument<'a>],
}

impl Display for FormatTrait<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Display(arg) => std_write!(f, Display, arg),
            Self::Debug(arg) => std_write!(f, Debug, arg),
            Self::Octal(arg) => std_write!(f, Display, arg),
            Self::LowerHex(arg) => std_write!(f, Display, arg),
            Self::UpperHex(arg) => std_write!(f, Display, arg),
            Self::Pointer(arg) => std_write!(f, Display, arg),
            Self::Binary(arg) => std_write!(f, Display, arg),
            Self::LowerExp(arg) => std_write!(f, Display, arg),
            Self::UpperExp(arg) => std_write!(f, Display, arg),
            Self::Stylish(arg) => arg.fmt(f),
        }
    }
}

impl Display for Arguments<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for piece in self.pieces {
            match piece {
                Argument::Lit(lit) => f.write_str(lit)?,
                Argument::Arg { args, style, arg } => {
                    arg.fmt(&mut f.with(style).with_args(args))?
                }
            }
        }
        Ok(())
    }
}
