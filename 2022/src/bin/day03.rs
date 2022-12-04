extern crate itertools;

use itertools::Itertools;

fn priority(c: char) -> u64 {
    assert!(('a'..='z').contains(&c) || ('A'..='Z').contains(&c));

    match c.is_uppercase() {
        true => c as u64 - 'A' as u64 + 27,
        false => c as u64 - 'a' as u64 + 1,
    }
}

fn main() {
    let input = include_str!("../../puzzles/day03.prod");

    let result: u64 = input
        .lines()
        .into_iter()
        .filter_map(|line| {
            let (lhs, rhs) = line.split_at(line.len() / 2);
            let common_char = lhs.chars().find(|c| rhs.contains(*c))?;

            Some(priority(common_char))
        })
        .sum();

    println!("{:?}", result);

    let result: u64 = input
        .lines()
        .into_iter()
        .batching(|iter| {
            // Note: The following line would be a good candidate for an `ArrayVec`.
            // https://github.com/tgross35/rfcs/blob/stackvec/text/3316-array-vec.md
            let lines = iter.take(3).collect::<Vec<_>>();
            if lines.len() < 3 {
                None
            } else {
                let common_char = lines[0]
                    .chars()
                    .find(|c| lines[1].contains(*c) && lines[2].contains(*c))?;

                Some(priority(common_char))
            }
        })
        .sum();

    println!("{:?}", result);
}
