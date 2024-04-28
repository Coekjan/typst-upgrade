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

pub fn show(old: &str, new: &str) {
    for line in diff::lines(old, new) {
        match line {
            diff::Result::Left(l) if *DIFF_CHOICE.get().unwrap() != DiffChoice::None => {
                diff!(del "{}", l);
            }
            diff::Result::Both(l, _) if *DIFF_CHOICE.get().unwrap() == DiffChoice::Full => {
                diff!("{}", l);
            }
            diff::Result::Right(r) if *DIFF_CHOICE.get().unwrap() != DiffChoice::None => {
                diff!(add "{}", r);
            }
            _ => (),
        }
    }
}
