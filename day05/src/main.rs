use std::collections::HashSet;
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
    let f = File::open(dir.join("day05.txt"))?;
    let mut reader = std::io::BufReader::new(f);

    let mut buffer = String::new();
    let mut ranges = HashSet::new();
    while reader.read_line(&mut buffer)? > 0 {
        let line = buffer.trim();
        if line.is_empty() {
            break;
        }
        ranges.insert(Range::from_str(line)?);
        buffer.clear();
    }

    let lines = reader.lines();
    let mut fresh = HashSet::new();
    for line in lines {
        let Ok(line) = line else {
            continue;
        };
        let ingredient = line.parse::<u64>()?;
        for range in &ranges {
            if range.contains_val(ingredient) {
                fresh.insert(ingredient);
            }
        }
    }

    println!("Part 1: {}", fresh.len());

    Ok(())
}

// Might be better to build up a bitvector rather than a set of ranges

#[derive(Debug, Hash, Eq, PartialEq)]
struct Range {
    start: u64,
    end: u64,
}

impl Range {
    fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut split = s.split('-');
        Ok(Self::new(split.next().unwrap().parse()?, split.next().unwrap().parse()?))
    }

    fn contains_val(&self, n: u64) -> bool {
        n >= self.start && n <= self.end
    }

    // Might want to add a method to check if a range contains another range
}
