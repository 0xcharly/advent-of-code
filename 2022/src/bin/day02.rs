extern crate clap;

use std::fs::File;
use std::io::{self, BufRead};

use clap::Parser;

#[derive(Clone)]
enum GameMove {
    Rock,
    Paper,
    Scissors,
}

impl GameMove {
    /// The score for the shape you selected:
    ///   - 1 for Rock
    ///   - 2 for Paper
    ///   - 3 for Scissors
    fn score(&self) -> u64 {
        match *self {
            GameMove::Rock => 1,
            GameMove::Paper => 2,
            GameMove::Scissors => 3,
        }
    }
}

enum GameOutcome {
    Loss,
    Draw,
    Win,
}

impl GameOutcome {
    /// The score for the outcome of the round:
    ///  - 0 if you lost
    ///  - 3 if the round was a draw
    ///  - 6 if you won).
    fn score(&self) -> u64 {
        match *self {
            GameOutcome::Loss => 0,
            GameOutcome::Draw => 3,
            GameOutcome::Win => 6,
        }
    }
}

/// Each game contains many rounds; in each round, the players each simultaneously choose one of
/// Rock, Paper, or Scissors.
struct GameRound {
    opponent_move: GameMove,
    strategy_move: GameMove,
}

impl GameRound {
    /// The score for a single round is the score for the shape you selected (1 for Rock, 2 for
    /// Paper, and 3 for Scissors) plus the score for the outcome of the round (0 if you lost, 3 if
    /// the round was a draw, and 6 if you won).
    fn score(&self) -> u64 {
        self.strategy_move.score() + self.outcome().score()
    }

    /// Rock defeats Scissors, Scissors defeats Paper, and Paper defeats Rock. If both players
    /// choose the same shape, the round instead ends in a draw.
    fn outcome(&self) -> GameOutcome {
        match (&self.opponent_move, &self.strategy_move) {
            (GameMove::Rock, GameMove::Rock) => GameOutcome::Draw,
            (GameMove::Rock, GameMove::Paper) => GameOutcome::Win,
            (GameMove::Rock, GameMove::Scissors) => GameOutcome::Loss,
            (GameMove::Paper, GameMove::Rock) => GameOutcome::Loss,
            (GameMove::Paper, GameMove::Paper) => GameOutcome::Draw,
            (GameMove::Paper, GameMove::Scissors) => GameOutcome::Win,
            (GameMove::Scissors, GameMove::Rock) => GameOutcome::Win,
            (GameMove::Scissors, GameMove::Paper) => GameOutcome::Loss,
            (GameMove::Scissors, GameMove::Scissors) => GameOutcome::Draw,
        }
    }
}

/// Simple one-to-one mapping from character to move.
fn decrypt_opponent_move(encrypted_move: char) -> Option<GameMove> {
    match encrypted_move {
        'A' => Some(GameMove::Rock),
        'B' => Some(GameMove::Paper),
        'C' => Some(GameMove::Scissors),
        _ => None,
    }
}

/// Simple one-to-one mapping from character to move, only valid for stage 1 of the challenge.
fn decrypt_strategy_move(encrypted_move: char) -> Option<GameMove> {
    match encrypted_move {
        'X' => Some(GameMove::Rock),
        'Y' => Some(GameMove::Paper),
        'Z' => Some(GameMove::Scissors),
        _ => None,
    }
}

fn iter_strategy_guide(strategy_guide: File) -> impl Iterator<Item = (char, char)> {
    io::BufReader::new(strategy_guide)
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let line = line.trim();
            let (lhs, rhs) = line.split_once(' ')?;
            Some((lhs.chars().nth(0)?, rhs.chars().nth(0)?))
        })
}

/// A strategically played round: the opponent's move, and the desired game outcome.
struct GameStrategy {
    opponent_move: GameMove,
    strategy_outcome: GameOutcome,
}

impl GameStrategy {
    /// Given the opponent's move, and the desired outcome, returns the round that needs to be
    /// played.
    fn strategy_round(&self) -> GameRound {
        let strategy_move = match (&self.opponent_move, &self.strategy_outcome) {
            (GameMove::Rock, GameOutcome::Loss) => GameMove::Scissors,
            (GameMove::Rock, GameOutcome::Draw) => GameMove::Rock,
            (GameMove::Rock, GameOutcome::Win) => GameMove::Paper,
            (GameMove::Paper, GameOutcome::Loss) => GameMove::Rock,
            (GameMove::Paper, GameOutcome::Draw) => GameMove::Paper,
            (GameMove::Paper, GameOutcome::Win) => GameMove::Scissors,
            (GameMove::Scissors, GameOutcome::Loss) => GameMove::Paper,
            (GameMove::Scissors, GameOutcome::Draw) => GameMove::Scissors,
            (GameMove::Scissors, GameOutcome::Win) => GameMove::Rock,
        };
        GameRound {
            opponent_move: self.opponent_move.to_owned(),
            strategy_move,
        }
    }
}

/// Simple one-to-one mapping from character to outcome, only valid for stage 2 of the challenge.
fn decrypt_strategy_outcome(encrypted_outcome: char) -> Option<GameOutcome> {
    match encrypted_outcome {
        'X' => Some(GameOutcome::Loss),
        'Y' => Some(GameOutcome::Draw),
        'Z' => Some(GameOutcome::Win),
        _ => None,
    }
}

#[derive(clap::ValueEnum, Clone)]
enum ChallengeStage {
    Stage1,
    Stage2,
}

#[derive(Parser)]
struct CmdlineArgs {
    // The path to the file to read.
    strategy_guide_filename: std::path::PathBuf,

    // The part of the challenge to run. Defaults to the first stage.
    #[clap(short = 'c', long = "challenge", value_enum, default_value_t = ChallengeStage::Stage1)]
    challenge: ChallengeStage,
}

fn main() {
    let cmdline_args = CmdlineArgs::parse();
    let strategy_guide =
        File::open(cmdline_args.strategy_guide_filename).expect("unable to open input file");

    let iter = iter_strategy_guide(strategy_guide);
    let total_score = match cmdline_args.challenge {
        ChallengeStage::Stage1 => iter
            .filter_map(|(opponent_move, strategy_move)| {
                let opponent_move = decrypt_opponent_move(opponent_move)?;
                let strategy_move = decrypt_strategy_move(strategy_move)?;
                Some(GameRound { opponent_move, strategy_move }.score())
            })
            .sum::<u64>(),
        ChallengeStage::Stage2 => iter
            .filter_map(|(opponent_move, strategy_outcome)| {
                let opponent_move = decrypt_opponent_move(opponent_move)?;
                let strategy_outcome = decrypt_strategy_outcome(strategy_outcome)?;
                Some(GameStrategy { opponent_move, strategy_outcome }.strategy_round().score())
            })
            .sum::<u64>(),
    };

    println!("{total_score}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_round_score_loss() {
        assert_eq!(
            GameRound {
                opponent_move: GameMove::Paper,
                strategy_move: GameMove::Rock
            }
            .score(),
            1
        );
        assert_eq!(
            GameRound {
                opponent_move: GameMove::Scissors,
                strategy_move: GameMove::Paper
            }
            .score(),
            2
        );
        assert_eq!(
            GameRound {
                opponent_move: GameMove::Rock,
                strategy_move: GameMove::Scissors
            }
            .score(),
            3
        );
    }

    #[test]
    fn test_game_round_score_draw() {
        assert_eq!(
            GameRound {
                opponent_move: GameMove::Rock,
                strategy_move: GameMove::Rock
            }
            .score(),
            4
        );
        assert_eq!(
            GameRound {
                opponent_move: GameMove::Paper,
                strategy_move: GameMove::Paper
            }
            .score(),
            5
        );
        assert_eq!(
            GameRound {
                opponent_move: GameMove::Scissors,
                strategy_move: GameMove::Scissors
            }
            .score(),
            6
        );
    }

    #[test]
    fn test_game_round_score_win() {
        assert_eq!(
            GameRound {
                opponent_move: GameMove::Scissors,
                strategy_move: GameMove::Rock
            }
            .score(),
            7
        );
        assert_eq!(
            GameRound {
                opponent_move: GameMove::Rock,
                strategy_move: GameMove::Paper
            }
            .score(),
            8
        );
        assert_eq!(
            GameRound {
                opponent_move: GameMove::Paper,
                strategy_move: GameMove::Scissors
            }
            .score(),
            9
        );
    }
}
