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
    let f = File::open(dir.join("day06.txt"))?;
    let reader = std::io::BufReader::new(f);

    let mut rows = Vec::new();
    let mut lines = reader.lines().flatten().collect::<Vec<_>>();
    let operations = lines.pop().unwrap().split_whitespace()
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    for line in lines {
        rows.push(
            line.split_whitespace()
                .flat_map(|s| s.parse::<usize>())
                .collect::<Vec<_>>(),
        );
    }

    let mut part1 = 0;
    for (i, op) in operations.iter().enumerate() {
        let mut result = 0;
        match op.as_str() {
            "*" => {
                result = 1;
                for row in &rows {
                    result *= row[i];
                }
            },
            "+" => {
                for row in &rows {
                    result += row[i];
                }
            }
            _ => unreachable!("Unrecognized operation: {}", op)
        }
        part1 += result;
    }

    println!("Part 1: {part1}");

    Ok(())
}
