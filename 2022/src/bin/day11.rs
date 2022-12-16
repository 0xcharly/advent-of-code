#[derive(Clone)]
enum WorryValue {
    Old,
    Num(u64),
}

impl WorryValue {
    fn eval(&self, old: u64) -> u64 {
        match self {
            &WorryValue::Old => old,
            &WorryValue::Num(value) => value,
        }
    }
}

#[derive(Clone)]
enum WorryFn {
    Add(WorryValue),
    Mul(WorryValue),
}

impl WorryFn {
    fn apply(&self, old: u64) -> u64 {
        match self {
            WorryFn::Add(value) => old + value.eval(old),
            WorryFn::Mul(value) => old * value.eval(old),
        }
    }
}

#[derive(Clone)]
struct TestFn {
    divisible: u64,
    target_if_divisible: usize,
    target_if_not_divisible: usize,
}

impl TestFn {
    fn new(divisible: u64, target_if_divisible: usize, target_if_not_divisible: usize) -> Self {
        TestFn {
            divisible,
            target_if_divisible,
            target_if_not_divisible,
        }
    }
}

#[derive(Clone)]
struct Monkey {
    items: Vec<u64>,
    worry: WorryFn,
    test: TestFn,
}

fn main() {
    let _input = include_str!("../../puzzles/day11.test");

    let puzzle_input = [
        Monkey {
            items: vec![65, 58, 93, 57, 66],
            worry: WorryFn::Mul(WorryValue::Num(7)),
            test: TestFn::new(19, 6, 4),
        },
        Monkey {
            items: vec![76, 97, 58, 72, 57, 92, 82],
            worry: WorryFn::Add(WorryValue::Num(4)),
            test: TestFn::new(3, 7, 5),
        },
        Monkey {
            items: vec![90, 89, 96],
            worry: WorryFn::Mul(WorryValue::Num(5)),
            test: TestFn::new(13, 5, 1),
        },
        Monkey {
            items: vec![72, 63, 72, 99],
            worry: WorryFn::Mul(WorryValue::Old),
            test: TestFn::new(17, 0, 4),
        },
        Monkey {
            items: vec![65],
            worry: WorryFn::Add(WorryValue::Num(1)),
            test: TestFn::new(2, 6, 2),
        },
        Monkey {
            items: vec![97, 71],
            worry: WorryFn::Add(WorryValue::Num(8)),
            test: TestFn::new(11, 7, 3),
        },
        Monkey {
            items: vec![83, 68, 88, 55, 87, 67],
            worry: WorryFn::Add(WorryValue::Num(2)),
            test: TestFn::new(5, 2, 1),
        },
        Monkey {
            items: vec![64, 81, 50, 96, 82, 53, 62, 92],
            worry: WorryFn::Add(WorryValue::Num(5)),
            test: TestFn::new(7, 3, 0),
        },
    ];
    let mut inspect_count = [0; 8];

    let mut monkeys = puzzle_input.clone();
    for _ in 0..20 {
        for idx in 0..monkeys.len() {
            let items: Vec<u64> = monkeys[idx].items.drain(..).collect();
            let monkey = monkeys[idx].clone();
            for item in items {
                inspect_count[idx] += 1;
                let item = monkey.worry.apply(item) / 3;
                let target_idx = if item % monkey.test.divisible == 0 {
                    monkey.test.target_if_divisible
                } else {
                    monkey.test.target_if_not_divisible
                };
                monkeys[target_idx].items.push(item);
            }
        }
    }

    inspect_count.sort();
    let monkey_business_level: u64 = inspect_count.iter().rev().take(2).product();

    println!("{:?}", monkey_business_level);

    let mut monkeys = puzzle_input.clone();
    let mut inspect_count = [0; 8];
    let common_multiple: u64 = monkeys.iter().map(|monkey| monkey.test.divisible).product();

    for _ in 0..10_000 {
        for idx in 0..monkeys.len() {
            let items: Vec<u64> = monkeys[idx].items.drain(..).collect();
            let monkey = monkeys[idx].clone();
            for item in items {
                inspect_count[idx] += 1;
                let item = monkey.worry.apply(item) % common_multiple;
                let target_idx = if item % monkey.test.divisible == 0 {
                    monkey.test.target_if_divisible
                } else {
                    monkey.test.target_if_not_divisible
                };
                monkeys[target_idx].items.push(item);
            }
        }
    }

    inspect_count.sort();
    let monkey_business_level: u64 = inspect_count.iter().rev().take(2).product();

    println!("{:?}", monkey_business_level);
}
