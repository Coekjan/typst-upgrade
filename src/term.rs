#[macro_export]
macro_rules! __term_println {
    (@COLOR_MOTION $stream:ident, $color:ident, $motion:literal, $($args:tt)*) => {
        use std::io::{IsTerminal, Write};
        use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

        let mut stream = StandardStream::$stream(if std::io::stdin().is_terminal() {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        });
        stream
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::$color)))
            .and_then(|_| {
                write!(&mut stream, "{:>12} ", $motion)?;
                stream.set_color(ColorSpec::new().set_reset(true))?;
                writeln!(&mut stream, $($args)*)
            })
            .expect("Cannot write to stdout");
    };

    (@COLOR_WHOLE_LINE $stream:ident, $color:ident, $motion:literal, $($args:tt)*) => {
        use std::io::{IsTerminal, Write};
        use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

        let mut stream = StandardStream::$stream(if std::io::stdin().is_terminal() {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        });
        stream
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::$color)))
            .and_then(|_| {
                write!(&mut stream, "{:>12} ", $motion)?;
                stream.set_color(ColorSpec::new().set_reset(true).set_fg(Some(Color::$color)))?;
                writeln!(&mut stream, $($args)*)
            })
            .expect("Cannot write to stdout");

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
