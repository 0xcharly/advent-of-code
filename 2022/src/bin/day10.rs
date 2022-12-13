extern crate itertools;

use itertools::Itertools;

/// Returns an iterator over the values of the `X` register for over time (ie. at each CPU cycle).
fn eval_inst<'a>(input: &'a str) -> impl Iterator<Item = i64> + 'a {
    let mut reg_x: i64 = 1;

    input
        .lines()
        .flat_map(move |line| match line.split_once(' ') {
            None => vec![reg_x],
            Some(("addx", val)) => {
                let prev_reg_x = reg_x;
                reg_x += val.parse::<i64>().unwrap();
                vec![prev_reg_x, prev_reg_x]
            }
            _ => panic!("invalid input line: {:?}", line),
        })
}

fn main() {
    let input = include_str!("../../puzzles/day10.prod");

    let sum_signal_strength_sample = (1i64..)
        .zip(eval_inst(input))
        .filter_map(|(cycle, reg_x)| match cycle % 40 == 20 {
            false => None,
            true => Some(reg_x * cycle),
        })
        .sum::<i64>();

    println!("{:?}", sum_signal_strength_sample);

    eval_inst(input).chunks(40).into_iter().for_each(|chunk| {
        let display_line = (0i64..)
            .zip(chunk)
            .map(|(pos, reg_x)| {
                if (reg_x - 1..=reg_x + 1).contains(&pos) {
                    '#'
                } else {
                    '.'
                }
            })
            .collect::<String>();

        println!("{}", display_line);
    });
}
