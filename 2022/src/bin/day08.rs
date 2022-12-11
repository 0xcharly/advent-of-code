/// A rectangular forest of trees. Each tree is represented by its height (a 0-9 integer value).
struct Forest {
    trees: Vec<u8>,
    width: usize,
}

impl Forest {
    fn len(&self) -> usize {
        self.trees.len()
    }

    fn at(&self, x: usize, y: usize) -> u8 {
        self.trees[y * self.width + x]
    }

    fn height(&self) -> usize {
        self.trees.len() / self.width
    }

    fn is_tree_hidden(&self, index: usize) -> bool {
        let (x, y) = (index / self.height(), index % self.width);
        let value = self.at(x, y);

        if x == 0 || x == self.width - 1 || y == 0 || y == self.height() - 1 {
            return false;
        }

        (0..x).any(|row| self.at(row, y) >= value)
            && (x + 1..self.width).any(|row| self.at(row, y) >= value)
            && (0..y).any(|col| self.at(x, col) >= value)
            && (y + 1..self.height()).any(|col| self.at(x, col) >= value)
    }
}

fn parse_forest_map(input: &str) -> Forest {
    Forest {
        trees: input
            .chars()
            .filter(|c| *c != '\n')
            .map(|c| {
                assert!(c >= '0' && c <= '9');
                c as u8 - '0' as u8
            })
            .collect(),
        width: input.lines().take(1).next().unwrap().chars().count(),
    }
}

fn viewing_distance<I, F>(range: I, predicate: F) -> Option<usize>
where
    F: Fn(usize) -> bool,
    I: Iterator<Item = usize>,
{
    range
        .enumerate()
        .find(|(_, i)| predicate(*i))
        .and_then(|(d, _)| Some(d + 1))
}

impl Forest {
    fn scenic_score(&self, index: usize) -> usize {
        let (w, h) = (self.width, self.height());
        let (x, y) = (index / h, index % w);
        let value = self.at(x, y);

        if x == 0 || x == w - 1 || y == 0 || y == h - 1 {
            return 0;
        }

        viewing_distance((0..x).rev(), |row| self.at(row, y) >= value).unwrap_or(x)
            * viewing_distance(x + 1..w, |row| self.at(row, y) >= value).unwrap_or(w - x - 1)
            * viewing_distance((0..y).rev(), |col| self.at(x, col) >= value).unwrap_or(y)
            * viewing_distance(y + 1..h, |col| self.at(x, col) >= value).unwrap_or(h - y - 1)
    }
}

fn main() {
    let forest = parse_forest_map(include_str!("../../puzzles/day08.prod"));

    let num_visible = (0..forest.len())
        .filter(|index| !forest.is_tree_hidden(*index))
        .count();
    println!("{:?}", num_visible);

    let highest_scenic_score = (0..forest.len())
        .map(|index| forest.scenic_score(index))
        .max()
        .unwrap();
    println!("{:?}", highest_scenic_score);
}
