use std::cmp::max;
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
    let f = File::open(dir.join("day03.txt"))?;
    let reader = std::io::BufReader::new(f);
    let lines = reader.lines();

    let mut part1 = 0;
    for line in lines {
        let Ok(line) = line else {
            continue;
        };
        let chars = line.chars().collect::<Vec<_>>();
        let mut highest = 0;
        for i in 0..(chars.len() - 1) {
            for j in i+1..chars.len() {
                let num = format!("{}{}", chars[i], chars[j]).parse::<usize>().unwrap();
                highest = max(highest, num);
            }
        }
        part1 += highest;
    }

    println!("Part 1: {part1}");

    Ok(())
}

