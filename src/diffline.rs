use std::{fmt::Display, sync::OnceLock};

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffChoice {
    Short,
    Full,
    None,
}

impl ValueEnum for DiffChoice {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Short, Self::Full, Self::None]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Short => clap::builder::PossibleValue::new("short"),
            Self::Full => clap::builder::PossibleValue::new("full"),
            Self::None => clap::builder::PossibleValue::new("none"),
        })
    }
}

impl Display for DiffChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

static DIFF_CHOICE: OnceLock<DiffChoice> = OnceLock::new();

pub fn init(diff: DiffChoice) {
    DIFF_CHOICE.set(diff).unwrap();
}

#[cfg(not(tarpaulin_include))]
pub fn show(old: &str, new: &str) {
    show_difflines(old, new, *DIFF_CHOICE.get().unwrap(), |res| match res {
        diff::Result::Left(l) => {
            diff!(del "{}", l);
        }
        diff::Result::Both(l, _) => {
            diff!("{}", l);
        }
        diff::Result::Right(r) => {
            diff!(add "{}", r);
        }
    })
}

fn show_difflines(old: &str, new: &str, diff: DiffChoice, out: impl Fn(diff::Result<&str>)) {
    for line in diff::lines(old, new) {
        match line {
            diff::Result::Left(l) if diff != DiffChoice::None => out(diff::Result::Left(l)),
            diff::Result::Both(l, r) if diff == DiffChoice::Full => out(diff::Result::Both(l, r)),
            diff::Result::Right(r) if diff != DiffChoice::None => out(diff::Result::Right(r)),
            _ => (),
        }
    }
}

#[cfg(test)]
mod test {
    use clap::ValueEnum;

    use super::DiffChoice;

    #[test]
    fn variants() {
        let variants: &[DiffChoice] = DiffChoice::value_variants();
        assert_eq!(
            variants,
            &[DiffChoice::Short, DiffChoice::Full, DiffChoice::None]
        );
    }

    #[test]
    fn parse() {
        for diff in ["short", "full", "none"] {
            let choice: DiffChoice = clap::ValueEnum::from_str(diff, false).unwrap();
            assert_eq!(choice.to_string(), diff);
        }
    }

    #[test]
    fn init() {
        super::init(DiffChoice::Short);
        assert!(std::panic::catch_unwind(|| {
            super::init(DiffChoice::Full);
        })
        .is_err());
        assert!(std::panic::catch_unwind(|| {
            super::init(DiffChoice::None);
        })
        .is_err());
    }

    #[test]
    fn show() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline2\nline4";
        super::show_difflines(old, new, DiffChoice::None, |_| {
            panic!("should not be called")
        });
        super::show_difflines(old, new, DiffChoice::Short, |res| match res {
            diff::Result::Left(l) => assert_eq!(l, "line3"),
            diff::Result::Both(_, _) => panic!("should not be called"),
            diff::Result::Right(r) => assert_eq!(r, "line4"),
        });
        super::show_difflines(old, new, DiffChoice::Full, |res| match res {
            diff::Result::Left(l) => assert_eq!(l, "line3"),
            diff::Result::Both(l, _) => assert!(matches!(l, "line1" | "line2")),
            diff::Result::Right(r) => assert_eq!(r, "line4"),
        });
    }
}
