use std::fmt::Arguments;
use std::io::Write;
use std::sync::OnceLock;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

static COLOR_CHOICE: OnceLock<ColorChoice> = OnceLock::new();

pub fn init(color: clap::ColorChoice) {
    COLOR_CHOICE
        .set(match color {
            clap::ColorChoice::Auto => ColorChoice::Auto,
            clap::ColorChoice::Always => ColorChoice::Always,
            clap::ColorChoice::Never => ColorChoice::Never,
        })
        .unwrap();
}

pub fn color_choice() -> ColorChoice {
    *COLOR_CHOICE.get().unwrap_or(&ColorChoice::Auto)
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn term_println(
    mut stream: StandardStream,
    color: Option<Color>,
    whole_line: bool,
    motion: &str,
    args: Arguments,
) -> Result<(), std::io::Error> {
    stream
        .set_color(ColorSpec::new().set_bold(true).set_fg(color))
        .and_then(|_| {
            write!(&mut stream, "{motion:>12} ")?;
            if whole_line {
                stream.set_color(ColorSpec::new().set_reset(true).set_fg(color))
            } else {
                stream.set_color(ColorSpec::new().set_reset(true))
            }?;
            writeln!(&mut stream, "{args}")
        })
}

macro_rules! __term_println {
    (@COLOR_MOTION $stream:ident, $color:expr, $motion:literal, $($args:tt)*) => {
        use std::io::IsTerminal;
        use termcolor::{ColorChoice, StandardStream};

        let stream = StandardStream::$stream(if std::io::$stream().is_terminal() {
            $crate::term::color_choice()
        } else {
            ColorChoice::Never
        });

        $crate::term::term_println(stream, $color, false, $motion, format_args!($($args)*))
            .expect(&format!("Cannot write to {}", stringify!($stream)));
    };

    (@COLOR_WHOLE_LINE $stream:ident, $color:expr, $motion:literal, $($args:tt)*) => {
        use std::io::IsTerminal;
        use termcolor::{ColorChoice, StandardStream};

        let stream = StandardStream::$stream(if std::io::$stream().is_terminal() {
            $crate::term::color_choice()
        } else {
            ColorChoice::Never
        });

        $crate::term::term_println(stream, $color, true, $motion, format_args!($($args)*))
            .expect(&format!("Cannot write to {}", stringify!($stream)));
    };
}

#[macro_export]
macro_rules! info {
    ($motion:literal: $($args:tt)*) => {
        __term_println!(@COLOR_MOTION stdout, Some(termcolor::Color::Green), $motion, $($args)*)
    };
    ($($args:tt)*) => {
        info!("INFO": $($args)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($motion:literal: $($args:tt)*) => {
        __term_println!(@COLOR_MOTION stderr, Some(termcolor::Color::Yellow), $motion, $($args)*)
    };
    ($($args:tt)*) => {
        warn!("WARN": $($args)*)
    };
}

#[macro_export]
macro_rules! error {
    ($motion:literal: $($args:tt)*) => {
        __term_println!(@COLOR_MOTION stderr, Some(termcolor::Color::Red), $motion, $($args)*)
    };
    ($($args:tt)*) => {
        error!("ERROR": $($args)*)
    };
}

#[macro_export]
macro_rules! diff {
    (del $($args:tt)*) => {
        __term_println!(@COLOR_WHOLE_LINE stdout, Some(termcolor::Color::Red), "-", $($args)*)
    };
    (add $($args:tt)*) => {
        __term_println!(@COLOR_WHOLE_LINE stdout, Some(termcolor::Color::Green), "+", $($args)*)
    };
    ($($args:tt)*) => {
        __term_println!(@COLOR_WHOLE_LINE stdout, None, "", $($args)*)
    };
}

#[cfg(test)]
mod test {
    use termcolor::ColorChoice;

    #[test]
    fn init() {
        super::init(clap::ColorChoice::Auto);
        assert_eq!(super::color_choice(), ColorChoice::Auto);

        assert!(
            std::panic::catch_unwind(|| {
                super::init(clap::ColorChoice::Always);
            })
            .is_err()
        );

        assert!(
            std::panic::catch_unwind(|| {
                super::init(clap::ColorChoice::Never);
            })
            .is_err()
        );
    }
}
