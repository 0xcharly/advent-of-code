extern crate itertools;

use std::collections::HashSet;

type Coordinates = (i64, i64);

/// A rope with several knots.
struct Rope<const N: usize> {
    knots: [Coordinates; N],
}

impl<const N: usize> Rope<N> {
    /// A rope must contain at least 2 knots (head and tails), and an arbitrary amount of knots in
    /// between.
    fn new(origin: (i64, i64)) -> Self {
        assert!(N > 1);
        Self { knots: [origin; N] }
    }

    /// Returns a mutable reference to the head knot.
    fn head_mut<'a>(&'a mut self) -> &'a mut Coordinates {
        &mut self.knots[0]
    }

    /// Returns a copy of the tail knot.
    fn tail(&self) -> Coordinates {
        self.knots[N - 1]
    }

    /// Adjusts the position of `self.knot[idx + 1]` if needed.
    /// Returns `true` if the position was changed, `false` otherwise.
    fn play_simulation_for_next_knot(&mut self, idx: usize) -> bool {
        let head = self.knots[idx];
        let tail = &mut self.knots[idx + 1];

        let delta_x = head.0 - tail.0;
        let delta_y = head.1 - tail.1;

        *tail = match (delta_x, delta_y) {
            (x, y) if x.abs() <= 1 && y.abs() <= 1 => return false,
            (x, 2) if x.abs() <= 1 => (head.0, tail.1 + 1),
            (x, -2) if x.abs() <= 1 => (head.0, tail.1 - 1),
            (2, y) if y.abs() <= 1 => (tail.0 + 1, head.1),
            (-2, y) if y.abs() <= 1 => (tail.0 - 1, head.1),
            (x, y) if x.abs() == 2 && y.abs() == 2 => (tail.0 + x.signum(), tail.1 + y.signum()),
            (_, _) => panic!("step too large"),
        };

        true
    }

    /// Moves the position of the head knot, then adjusts the position of the following knots
    /// accordingly.
    fn perform_move(&mut self, direction: &str) {
        match direction {
            "L" => self.head_mut().0 -= 1,
            "R" => self.head_mut().0 += 1,
            "U" => self.head_mut().1 += 1,
            "D" => self.head_mut().1 -= 1,
            _ => panic!("invalid direction"),
        };

        // Run the simulation on other knots of the rope.
        for i in 0..N - 1 {
            if !self.play_simulation_for_next_knot(i) {
                break;
            }
        }
    }
}

/// Runs the simulation for a rope of size `N`.
fn run_simulation<const N: usize>(input: &str) -> usize {
    let origin = (0, 0);
    let mut rope = Rope::<N>::new(origin);
    let mut trail = HashSet::new();

    input
        .lines()
        .for_each(|motion| match motion.split_once(' ') {
            Some((direction, steps)) => {
                let steps = steps
                    .parse::<usize>()
                    .expect(&format!("expected number, got `{:?}`", steps));

                for _ in 0..steps {
                    rope.perform_move(direction);
                    trail.insert(rope.tail());
                }
            }
            _ => panic!("unexpected motion: {:?}", motion),
        });

    trail.len()
}

fn main() {
    let input = include_str!("../../puzzles/day09.prod");

    let unique_tail_position = run_simulation::<2>(input);
    println!("{:?}", unique_tail_position);

    let unique_tail_position = run_simulation::<10>(input);
    println!("{:?}", unique_tail_position);
}
