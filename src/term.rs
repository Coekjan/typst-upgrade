use std::fmt::Arguments;
use std::io::Write;

use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

pub fn term_println(
    mut stream: StandardStream,
    color: Color,
    whole_line: bool,
    motion: &str,
    args: Arguments,
) -> Result<(), std::io::Error> {
    stream
        .set_color(ColorSpec::new().set_bold(true).set_fg(Some(color)))
        .and_then(|_| {
            write!(&mut stream, "{:>12} ", motion)?;
            if whole_line {
                stream.set_color(ColorSpec::new().set_reset(true).set_fg(Some(color)))
            } else {
                stream.set_color(ColorSpec::new().set_reset(true))
            }?;
            writeln!(&mut stream, "{}", args)
        })
}

#[macro_export]
macro_rules! __term_println {
    (@COLOR_MOTION $stream:ident, $color:ident, $motion:literal, $($args:tt)*) => {
        use std::io::IsTerminal;
        use termcolor::{Color, ColorChoice, StandardStream};

        let stream = StandardStream::$stream(if std::io::$stream().is_terminal() {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        });

        $crate::term::term_println(stream, Color::$color, false, $motion, format_args!($($args)*))
            .expect(&format!("Cannot write to {}", stringify!($stream)));
    };

    (@COLOR_WHOLE_LINE $stream:ident, $color:ident, $motion:literal, $($args:tt)*) => {
        use std::io::IsTerminal;
        use termcolor::{Color, ColorChoice, StandardStream};

        let stream = StandardStream::$stream(if std::io::$stream().is_terminal() {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        });

        $crate::term::term_println(stream, Color::$color, true, $motion, format_args!($($args)*))
            .expect(&format!("Cannot write to {}", stringify!($stream)));
    };
}

#[macro_export]
macro_rules! info {
    ($motion:literal: $($args:tt)*) => {
        __term_println!(@COLOR_MOTION stdout, Green, $motion, $($args)*)
    };
    ($($args:tt)*) => {
        info!("INFO": $($args)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($motion:literal: $($args:tt)*) => {
        __term_println!(@COLOR_MOTION stderr, Yellow, $motion, $($args)*)
    };
    ($($args:tt)*) => {
        warn!("WARN": $($args)*)
    };
}

#[macro_export]
macro_rules! error {
    ($motion:literal: $($args:tt)*) => {
        __term_println!(@COLOR_MOTION stderr, Red, $motion, $($args)*)
    };
    ($($args:tt)*) => {
        error!("ERROR": $($args)*)
    };
}

#[macro_export]
macro_rules! diff {
    (del $($args:tt)*) => {
        __term_println!(@COLOR_WHOLE_LINE stdout, Red, "-", $($args)*)
    };
    (add $($args:tt)*) => {
        __term_println!(@COLOR_WHOLE_LINE stdout, Green, "+", $($args)*)
    };
}
