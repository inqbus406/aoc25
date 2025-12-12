use std::fs::File;
use std::io::{BufRead, Read};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let dir = if args.contains(&String::from("--test_input")) {
        PathBuf::from("test_input")
    } else {
        PathBuf::from("input")
    };

    let f = File::open(dir.join("day12.txt"))?;
    let mut reader = std::io::BufReader::new(f);

    let mut buffer = String::new();
    let mut sizes: Vec<usize> = Vec::new();
    let mut index = 0;
    let mut trees = Vec::new();

    while reader.read_line(&mut buffer)? > 0 {
        let line = buffer.trim();
        if line.is_empty() {
            index += 1;
            buffer.clear();
            continue;
        }
        if line.contains('x') {
            trees.push(line.to_string());
            buffer.clear();
            break;
        }
        if line.contains(':') {
            sizes.push(0);
            buffer.clear();
            continue;
        }
        sizes[index] += line.chars().into_iter().filter(|c| *c == '#').count();

        buffer.clear();
    }

    trees.extend(reader.lines().into_iter().map(|l| l.unwrap()));

    let mut result = 0;
    for tree in trees {
        let tokens = tree.trim().split(':').collect::<Vec<_>>();
        let size = tokens[0]
            .split('x')
            .map(|s| s.parse::<usize>().unwrap())
            .product::<usize>();
        let counts = tokens[1]
            .split_whitespace()
            .map(|s| s.parse::<usize>().unwrap())
            .collect::<Vec<_>>();

        let required_size = counts
            .iter()
            .zip(sizes.iter())
            .map(|(c, s)| c * s)
            .sum::<usize>();
        if required_size <= size {
            result += 1;
        }

        buffer.clear();
    }

    println!("Part 1: {result}");

    Ok(())
}
