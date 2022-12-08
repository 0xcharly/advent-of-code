use std::iter::repeat;
use std::str::FromStr;

use anyhow::{anyhow, Ok, Result};

extern crate anyhow;

#[derive(Clone)]
struct CrateStacks {
    stacks: Vec<Vec<char>>,
}

struct MoveCommand {
    crate_count: usize,
    src_index: usize,
    dst_index: usize,
}

impl FromStr for MoveCommand {
    type Err = anyhow::Error;

    /// Parses a move command of the form `move COUNT from SRC to DST`.
    fn from_str(s: &str) -> Result<Self> {
        let mut iter = s.splitn(6, ' ').skip(1).step_by(2);
        let crate_count = iter
            .next()
            .ok_or_else(|| anyhow!("unpexpected move syntax"))?
            .parse()
            .map_err(|e| anyhow!("failed to parse crate_number: {:?}", e))?;
        let src_index = iter
            .next()
            .ok_or_else(|| anyhow!("unpexpected move syntax"))?
            .parse()
            .map_err(|e| anyhow!("failed to parse crate_number: {:?}", e))?;
        let dst_index = iter
            .next()
            .ok_or_else(|| anyhow!("unpexpected move syntax"))?
            .parse()
            .map_err(|e| anyhow!("failed to parse crate_number: {:?}", e))?;

        Ok(MoveCommand {
            crate_count,
            src_index,
            dst_index,
        })
    }
}

impl CrateStacks {
    fn play_move_with_cratemover_9000(&mut self, move_cmd: &MoveCommand) {
        repeat(()).take(move_cmd.crate_count).for_each(|()| {
            let top = self.stacks[move_cmd.src_index - 1]
                .pop()
                .expect("unexpected empty stack");
            self.stacks[move_cmd.dst_index - 1].push(top);
        })
    }

    fn play_move_with_cratemover_9001(&mut self, move_cmd: &MoveCommand) {
        let src_size = self.stacks[move_cmd.src_index - 1].len();
        let tail = self.stacks[move_cmd.src_index - 1].split_off(src_size - move_cmd.crate_count);
        self.stacks[move_cmd.dst_index - 1].extend(tail);
    }

    /// Returns a `String` made out the top characters of each stack.
    /// Panics if one of the stack is empty.
    fn get_top_crates(&self) -> String {
        self.stacks
            .iter()
            .map(|stack| stack.last().expect("unexpected empty stack"))
            .collect::<String>()
    }
}

impl FromStr for CrateStacks {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut lines = s.lines().collect::<Vec<_>>();
        let indexes = lines.pop().expect("unexpected crate stack syntax");
        let stack_count = indexes
            .split(' ')
            .last()
            .expect("unexpected index line syntax")
            .parse::<usize>()
            .expect("unexpected index format");
        let mut stacks = vec![vec![]; stack_count];

        s.lines().rev().skip(1).for_each(|line| {
            for i in 0..stack_count {
                let pos = 1 + i * 4;
                match line.chars().nth(pos) {
                    None | Some(' ') => continue,
                    Some(c) => stacks[i].push(c),
                };
            }
        });

        Ok(CrateStacks { stacks })
    }
}

fn main() {
    let input = include_str!("../../puzzles/day05.prod");
    let (crate_stacks_initial_state, move_list) = input.split_once("\n\n").expect("invalid input");

    let crate_stacks = crate_stacks_initial_state
        .parse::<CrateStacks>()
        .expect("failed to parse initial state");

    let mut simulation_cratemover_9000_stack = crate_stacks.clone();
    move_list.lines().map(MoveCommand::from_str).for_each(|m| {
        simulation_cratemover_9000_stack
            .play_move_with_cratemover_9000(&m.expect("failed to parse move"))
    });
    println!("{:?}", simulation_cratemover_9000_stack.get_top_crates());

    let mut simulation_cratemover_9001_stack = crate_stacks.clone();
    move_list.lines().map(MoveCommand::from_str).for_each(|m| {
        simulation_cratemover_9001_stack
            .play_move_with_cratemover_9001(&m.expect("failed to parse move"))
    });

    println!("{:?}", simulation_cratemover_9001_stack.get_top_crates());
}
