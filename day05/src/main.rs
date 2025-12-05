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

    let mut checker = ProduceChecker::new();
    checker.update_from_reader(&mut reader)?;

    let lines = reader.lines();
    let mut fresh = HashSet::new();
    for line in lines {
        let Ok(line) = line else {
            continue;
        };
        let ingredient = line.parse::<usize>()?;
        if checker.check_produce(ingredient) {
            fresh.insert(ingredient);
        }
    }

    println!("Part 1: {}", fresh.len());
    println!("Part 2: {}", checker.count_all_fresh());

    Ok(())
}

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
struct Range {
    start: usize,
    end: usize,
}

impl Range {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut split = s.split('-');
        Ok(Self::new(
            split.next().unwrap().parse()?,
            split.next().unwrap().parse()?,
        ))
    }

    fn contains_val(&self, n: usize) -> bool {
        n >= self.start && n <= self.end
    }

    fn contains_range(&self, other: &Range) -> bool {
        self.contains_val(other.start) && self.contains_val(other.end)
    }

    fn overlap(&self, other: &Range) -> Option<Range> {
        if self.end < other.start || self.start > other.end {
            return None;
        }
        let potential = Range::new(
            std::cmp::min(self.start, other.start),
            std::cmp::max(self.end, other.end),
        );
        if potential.is_valid() {
            Some(potential)
        } else {
            None
        }
    }

    fn is_valid(&self) -> bool {
        self.start <= self.end
    }

    fn size(&self) -> usize {
        self.end - self.start + 1
    }
}

struct ProduceChecker {
    ranges: HashSet<Range>,
}

impl ProduceChecker {
    fn new() -> Self {
        Self {
            ranges: HashSet::new(),
        }
    }

    fn update_from_reader(&mut self, reader: &mut impl BufRead) -> anyhow::Result<()> {
        let mut buffer = String::new();
        while reader.read_line(&mut buffer)? > 0 {
            let line = buffer.trim();
            if line.is_empty() {
                break;
            }
            self.add_range(&Range::from_str(line)?);
            buffer.clear();
        }

        Ok(())
    }

    fn add_range(&mut self, range: &Range) {
        let mut new_ranges = HashSet::new();
        let mut range = range.clone();

        for existing in self.ranges.drain() {
            if let Some(merged) = Self::merge_ranges(&existing, &range) {
                range = merged;
            } else {
                new_ranges.insert(existing);
            }
        }
        self.ranges = new_ranges;
        self.ranges.insert(range);
    }

    fn merge_ranges(range0: &Range, range1: &Range) -> Option<Range> {
        if range0 == range1 {
            return Some(range0.clone());
        }
        if range0.contains_range(range1) {
            return Some(range0.clone());
        }
        if range1.contains_range(range0) {
            return Some(range1.clone());
        }
        range0.overlap(range1)
    }

    fn check_produce(&self, ingredient: usize) -> bool {
        self.ranges.iter().any(|r| r.contains_val(ingredient))
    }

    fn count_all_fresh(&self) -> usize {
        self.ranges.iter().fold(0, |acc, r| acc + r.size())
    }
}
