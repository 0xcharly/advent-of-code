extern crate anyhow;

use anyhow::{anyhow, Result};
use std::fmt;
use std::ops::RangeInclusive;
use std::str::FromStr;

struct RangePair<T: PartialOrd + FromStr> {
    first: RangeInclusive<T>,
    second: RangeInclusive<T>,
}

trait RangeInclusiveExtension {
    /// Whether `self` fully contains `other`.
    ///
    /// ```
    /// assert_eq!((1..=5).fully_contains(2..=3));
    /// assert_eq!((1..=5).fully_contains(2..=5));
    /// assert_ne!((1..=5).fully_contains(2..=7));
    /// ```
    fn fully_contains(&self, other: &Self) -> bool;
}

impl<T: PartialOrd> RangeInclusiveExtension for RangeInclusive<T> {
    fn fully_contains(&self, other: &Self) -> bool {
        self.start() <= other.start() && other.end() <= self.end()
    }
}

impl<T: PartialOrd + FromStr> RangePair<T> {
    /// Whether `self.first` fully contains `self.second`, or vice-versa.
    fn any_fully_contains_other(&self) -> bool {
        self.first.fully_contains(&self.second) || self.second.fully_contains(&self.first)
    }

    /// Whether `self.first` and `self.second` overlaps. Two ranges overlaps iff:
    ///   - one fully contains the other, or
    ///   - they share a common sub-range
    fn overlaps(&self) -> bool {
        self.any_fully_contains_other()
            || (self.first.end() >= self.second.start() && self.first.start() <= self.second.end())
    }
}

/// Parses a range of the form `"X-Y"`, where `X` and `Y` are both positive numbers.
fn parse_inclusive_range<T>(range: &str) -> Result<RangeInclusive<T>>
where
    T: PartialOrd + FromStr,
    <T as FromStr>::Err: fmt::Debug,
{
    let (lhs, rhs) = range
        .split_once('-')
        .ok_or_else(|| anyhow!("failed to split bounds: {:?}", range))?;
    let start = lhs.parse().map_err(|e| anyhow!("start bound: {:?}", e))?;
    let end = rhs.parse().map_err(|e| anyhow!("end bound: {:?}", e))?;

    Ok(start..=end)
}

impl<T> FromStr for RangePair<T>
where
    T: PartialOrd + FromStr,
    <T as FromStr>::Err: fmt::Debug,
{
    type Err = anyhow::Error;

    /// Parses a pair of ranges of the form `"A-B,C-D"`, where `A-B` and `C-D`
    /// are both inclusive ranges that can be parsed by `parse_inclusive_range`.
    fn from_str(s: &str) -> Result<Self> {
        let (lhs, rhs) = s
            .split_once(',')
            .ok_or(anyhow!("failed to split ranges: {:?}", s))?;

        Ok(RangePair {
            first: parse_inclusive_range(lhs)?,
            second: parse_inclusive_range(rhs)?,
        })
    }
}

/// Parses `input` and converts each line into a pair of inclusive ranges.
/// Returns the number of lines that matches `predicate`.
fn count_by<T, Predicate>(input: &str, predicate: Predicate) -> usize
where
    T: PartialOrd + FromStr,
    <T as FromStr>::Err: fmt::Debug,
    Predicate: Fn(&RangePair<T>) -> bool,
{
    input
        .lines()
        .into_iter()
        .filter_map(|line| predicate(&line.parse().ok()?).then_some(()))
        .count()
}

fn main() {
    let input = include_str!("../../puzzles/day04.prod");

    println!("{:?}", count_by(input, RangePair::<u64>::any_fully_contains_other));
    println!("{:?}", count_by(input, RangePair::<u64>::overlaps));
}
