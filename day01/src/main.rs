use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let f = if args.contains(&String::from("--test_input")) {
        File::open("test_input/day01.txt")?
    } else {
        File::open("input/day01.txt")?
    };
    let reader = BufReader::new(f);
    let lines = reader.lines();

    let mut number = 50;
    let mut part1_result = 0;
    let mut part2_result = 0;
    for line in lines {
        let Ok(line) = line else {
            continue;
        };
        if line.is_empty() {
            continue;
        }
        let magnitude = get_magnitude(&line)?;
        match line.chars().nth(0) {
            Some('L') => {
                part2_result += if magnitude < number {
                    0
                } else if magnitude == number {
                    1
                } else {
                    (magnitude - number) / 100
                };
                if magnitude > number && number > 0 {
                    part2_result += 1;
                }
                number = (number - magnitude).rem_euclid(100);
            }
            Some('R') => {
                part2_result += (number + magnitude) / 100;
                number = (number + magnitude).rem_euclid(100);
            }
            _ => unreachable!(),
        }
        if number == 0 {
            part1_result += 1;
        }
    }
    println!("Part 1: {part1_result}");
    println!("Part 2: {part2_result}");

    Ok(())
}

fn get_magnitude(line: &str) -> anyhow::Result<i32> {
    Ok(line.chars().skip(1).collect::<String>().parse::<i32>()?)
}
