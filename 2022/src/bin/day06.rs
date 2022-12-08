use std::collections::HashSet;

fn has_duplicates<const N: usize>(buf: &[char; N]) -> bool {
    let mut seen = HashSet::new();
    for i in 0..N {
        if seen.contains(&buf[i]) {
            return true;
        }
        seen.insert(buf[i]);
    }
    false
}

fn find_first_marker<const N: usize>(stream: &str) -> Option<usize> {
    assert!(stream.len() >= N);

    let mut buf = [char::default(); N];
    let mut idx = 0;

    let mut iter = stream.chars().enumerate();
    for i in 0..N {
        (_, buf[i]) = iter.next()?;
    }

    while let Some((pos, c)) = iter.next() {
        if !has_duplicates(&buf) {
            return Some(pos);
        }
        buf[idx] = c;
        idx = (idx + 1) % N;
    }

    None
}

fn main() {
    let input = include_str!("../../puzzles/day06.prod");

    println!("{:?}", find_first_marker::<4>(input).expect("marker not found"));
    println!("{:?}", find_first_marker::<14>(input).expect("marker not found"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_of_packet_test() {
        assert_eq!(find_first_marker::<4>("bvwbjplbgvbhsrlpgdmjqwftvncz"), Some(5));
        assert_eq!(find_first_marker::<4>("nppdvjthqldpwncqszvftbrmjlhg"), Some(6));
        assert_eq!(find_first_marker::<4>("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), Some(10));
        assert_eq!(find_first_marker::<4>("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), Some(11));
    }

    #[test]
    fn start_of_message_test() {
        assert_eq!(find_first_marker::<14>("mjqjpqmgbljsphdztnvjfqwrcgsmlb"), Some(19));
        assert_eq!(find_first_marker::<14>("bvwbjplbgvbhsrlpgdmjqwftvncz"), Some(23));
        assert_eq!(find_first_marker::<14>("nppdvjthqldpwncqszvftbrmjlhg"), Some(23));
        assert_eq!(find_first_marker::<14>("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), Some(29));
        assert_eq!(find_first_marker::<14>("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), Some(26));
    }
}
