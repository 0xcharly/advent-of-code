extern crate clap;
extern crate itertools;

use clap::Parser;
use itertools::Itertools;
use std::borrow::Borrow;
use std::cmp;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(clap::ValueEnum, Clone)]
enum ChallengeStage {
    Stage1,
    Stage2,
}

#[derive(Parser)]
struct CmdlineArgs {
    // The path to the file to read.
    calorie_ledger_filename: std::path::PathBuf,

    // The part of the challenge to run. Defaults to the first stage.
    #[clap(short = 'c', long = "challenge", value_enum, default_value_t = ChallengeStage::Stage1)]
    challenge: ChallengeStage,
}

/// An input file consists of a newline-separated list of either:
///   - an empty line
///   - a positive number
enum CalorieLedgerToken {
    Newline,
    Number(u64), // `u64` should cover even the fattest of elves…
}

/// Parses the content `calories_ledger` and yields a stream of tokens.
///
/// Implements moderate error tolerance by:
///   - ignoring leading and trailing whitespaces on each line
///   - ignoring ill-formated calories values
fn iter_calorie_ledger(calories_ledger: File) -> impl Iterator<Item = CalorieLedgerToken> {
    io::BufReader::new(calories_ledger)
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let line = line.trim();
            if line.is_empty() {
                Some(CalorieLedgerToken::Newline)
            } else {
                let calories = line.parse::<u64>().ok()?;
                Some(CalorieLedgerToken::Number(calories))
            }
        })
}

/// The first part of the challenge consists in returning the largest value in the input set.
///
/// This was the first iteration of the Day 1 challenge which has been generalized in
/// `challenge_n_largest`.
fn challenge_stage1(iter: impl Iterator<Item = impl Borrow<CalorieLedgerToken>>) -> u64 {
    let mut max_calories = 0;
    let mut acc = 0;

    for entry in iter {
        match entry.borrow() {
            CalorieLedgerToken::Newline => {
                max_calories = cmp::max(acc, max_calories);
                acc = 0;
            }
            CalorieLedgerToken::Number(calories) => acc += calories,
        }
    }

    // Don't drop the latest values.
    cmp::max(acc, max_calories)
}

/// Converts a stream of `CalorieLedgerToken` into a stream of calories values.
fn iter_calories(
    iter: impl Iterator<Item = impl Borrow<CalorieLedgerToken>>,
) -> impl Iterator<Item = u64> {
    iter.batching(|iter| {
        iter.map_while(|token| match token.borrow() {
            CalorieLedgerToken::Newline => None,
            CalorieLedgerToken::Number(calories) => Some(calories.to_owned()),
        })
        .sum1()
    })
}

/// Keeps the largest N values from the (value, ...n_largest) set.
///
/// If `n_largest` contains duplicate values, the first smallest element in the input order is
/// replaced by `value`.
///
/// This means that:
///
/// ```
/// let mut values = [0; 3];
///
/// keep_n_largest(&mut values, 1);
/// assert_eq!(values, [1, 0, 0]);
/// ```
fn keep_n_largest<T: PartialOrd, const N: usize>(n_largest: &mut [T; N], value: T) {
    // This is O(n), and works with a `PartialOrd` bound.
    let index_of_min = n_largest
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal))
        .map(|(index, _)| index);

    if let Some(index_of_min) = index_of_min {
        if n_largest[index_of_min] < value {
            n_largest[index_of_min] = value;
        }
    }
}

/// The second part of the challenge consists in returning the sum of the 3 largest values in the
/// input set.
///
/// This function generalizes the concept by returning the sum of the N largest values in the input
/// set.
fn challenge_n_largest<const N: usize>(
    iter: impl Iterator<Item = impl Borrow<CalorieLedgerToken>>,
) -> u64 {
    let mut n_largest = [u64::MIN; N];

    for value in iter_calories(iter) {
        keep_n_largest(&mut n_largest, value);
    }

    n_largest.iter().sum()
}

fn main() -> Result<(), std::io::Error> {
    let cmdline_args = CmdlineArgs::parse();
    let calorie_ledger =
        File::open(cmdline_args.calorie_ledger_filename).expect("unable to open input file");

    let iter = iter_calorie_ledger(calorie_ledger);
    let calories = match cmdline_args.challenge {
        ChallengeStage::Stage1 => challenge_stage1(iter),
        ChallengeStage::Stage2 => challenge_n_largest::<3>(iter),
    };

    println!("{calories}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Some tests, starting with part 1 of the challenge.

    #[test]
    fn challenge_stage1_empty_input() {
        assert_eq!(challenge_stage1([].iter()), 0);
    }

    #[test]
    fn challenge_stage1_newlines_only() {
        let input = [CalorieLedgerToken::Newline, CalorieLedgerToken::Newline];
        assert_eq!(challenge_stage1(input.iter()), 0);
    }

    #[test]
    fn challenge_stage1_single_group() {
        let input = [CalorieLedgerToken::Number(1), CalorieLedgerToken::Number(2)];
        assert_eq!(challenge_stage1(input.iter()), 3);
    }

    #[test]
    fn challenge_stage1_multiple_group() {
        let input = [
            CalorieLedgerToken::Number(1),
            CalorieLedgerToken::Number(2),
            CalorieLedgerToken::Newline,
            CalorieLedgerToken::Number(3),
            CalorieLedgerToken::Number(4),
        ];
        assert_eq!(challenge_stage1(input.iter()), 7);
    }

    #[test]
    fn challenge_stage1_with_eof() {
        let input = [
            CalorieLedgerToken::Number(1),
            CalorieLedgerToken::Number(2),
            CalorieLedgerToken::Newline,
            CalorieLedgerToken::Number(3),
            CalorieLedgerToken::Number(4),
            CalorieLedgerToken::Newline,
        ];
        assert_eq!(challenge_stage1(input.iter()), 7);
    }

    // Tests for part 2 of the challenge.

    #[test]
    fn iter_calories_empty() {
        let mut iter = iter_calories([].iter());

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_calories_newlines_only() {
        let input = [CalorieLedgerToken::Newline, CalorieLedgerToken::Newline];
        let mut iter = iter_calories(input.iter());

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_calories_single_group() {
        let input = [CalorieLedgerToken::Number(1), CalorieLedgerToken::Number(2)];
        let mut iter = iter_calories(input.iter());

        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_calories_multiple_group() {
        let input = [
            CalorieLedgerToken::Number(1),
            CalorieLedgerToken::Number(2),
            CalorieLedgerToken::Newline,
            CalorieLedgerToken::Number(3),
            CalorieLedgerToken::Number(4),
        ];
        let mut iter = iter_calories(input.iter());

        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(7));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_calories_with_eof() {
        let input = [
            CalorieLedgerToken::Number(1),
            CalorieLedgerToken::Number(2),
            CalorieLedgerToken::Newline,
            CalorieLedgerToken::Number(3),
            CalorieLedgerToken::Number(4),
            CalorieLedgerToken::Newline,
        ];
        let mut iter = iter_calories(input.iter());

        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(7));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn keep_n_largest_stable_replace() {
        let mut values = [0; 3];

        keep_n_largest(&mut values, 0);
        assert_eq!(values, [0, 0, 0]);

        keep_n_largest(&mut values, 1);
        assert_eq!(values, [1, 0, 0]);

        keep_n_largest(&mut values, 2);
        assert_eq!(values, [1, 2, 0]);

        keep_n_largest(&mut values, 1);
        assert_eq!(values, [1, 2, 1]);

        keep_n_largest(&mut values, 5);
        assert_eq!(values, [5, 2, 1]);

        keep_n_largest(&mut values, 7);
        assert_eq!(values, [5, 2, 7]);

        keep_n_largest(&mut values, 1);
        assert_eq!(values, [5, 2, 7]);
    }

    #[test]
    fn challenge_n_largest_generalizes_stage1() {
        let input = [
            CalorieLedgerToken::Number(1),
            CalorieLedgerToken::Number(2),
            CalorieLedgerToken::Newline,
            CalorieLedgerToken::Number(3),
            CalorieLedgerToken::Number(4),
            CalorieLedgerToken::Newline,
            CalorieLedgerToken::Number(5),
            CalorieLedgerToken::Number(6),
            CalorieLedgerToken::Newline,
            CalorieLedgerToken::Number(7),
            CalorieLedgerToken::Number(8),
            CalorieLedgerToken::Newline,
        ];

        assert_eq!(
            challenge_n_largest::<1>(input.iter()),
            challenge_stage1(input.iter())
        );
        assert_eq!(challenge_n_largest::<2>(input.iter()), 26);
        assert_eq!(challenge_n_largest::<3>(input.iter()), 33);
    }
}
