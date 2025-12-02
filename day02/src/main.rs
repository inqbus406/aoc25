use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let dir = if args.contains(&String::from("--test_input")) {
        PathBuf::from("test_input")
    } else {
        PathBuf::from("input")
    };
    let f = File::open(dir.join("day02.txt"))?;
    let reader = std::io::BufReader::new(f);
    let lines = reader.lines();

    let mut ranges = Vec::new();
    for line in lines {
        let Ok(line) = line else {
            continue;
        };
        if line.is_empty() {
            continue;
        }
        ranges.extend(line
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|r| {
                r.split('-')
                    .map(|n| n.parse::<usize>().unwrap())
                    .collect::<Vec<_>>()
            }));
    }

    let part1_result = part1(&ranges);
    let part2_result = part2(&ranges);
    println!("Part 1: {part1_result}");
    println!("Part 2: {part2_result}");

    Ok(())
}

fn part1(ranges: &[Vec<usize>]) -> usize {
    let mut result = 0;
    for range in ranges {
        for num in range[0]..=range[1] {
            let num_string = num.to_string();
            let midpoint = num_string.len() / 2;

            if num_string[..midpoint] == num_string[midpoint..] {
                result += num;
            }
        }
    }

    result
}

fn part2(ranges: &[Vec<usize>]) -> usize {
    let mut result = 0;

    for range in ranges {
        for num in range[0]..=range[1] {
            let chars = num.to_string().chars().collect::<Vec<_>>();
            for i in (1..num.to_string().len()).rev() {
                let chunks = chars.chunks(i).collect::<Vec<_>>();
                if chunks.iter().skip(1).all(|c| *c == chunks[0]) {
                    result += num;
                    break;
                }
            }
        }
    }

    result
}
