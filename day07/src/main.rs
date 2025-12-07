use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufRead;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let dir = if args.contains(&String::from("--test_input")) {
        PathBuf::from("test_input")
    } else {
        PathBuf::from("input")
    };
    let mut map = Map::from_file(&dir.join("day07.txt"))?;

    while !map.step() {}

    println!("Part 1: {}", map.splits);
    println!("Part 2: {}", map.beams.values().sum::<usize>());

    Ok(())
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct Position(usize, usize);

#[derive(Debug)]
struct Map {
    splitters: HashSet<Position>,
    height: usize,
    beams: HashMap<Position, usize>,
    splits: usize,
}

impl Map {
    fn from_file(path: &impl AsRef<Path>) -> anyhow::Result<Self> {
        let f = File::open(path)?;
        let reader = std::io::BufReader::new(f);

        let mut height = 0;
        let mut splitters = HashSet::new();

        // beams should start with just 1 position, the starting position S
        let mut beams = HashMap::new();

        for (y, line) in reader.lines().enumerate() {
            let line = line?;

            for (x, c) in line.chars().enumerate() {
                let p = Position(x, y);

                match c {
                    'S' => {
                        beams.insert(p, 1);
                    }
                    '^' => {
                        splitters.insert(p);
                    }
                    _ => {}
                }
            }

            height = max(height, y + 1);
        }

        Ok(Self {
            splitters,
            height,
            beams,
            splits: 0,
        })
    }

    // Returns true if all beams have exited tachyon (simulation is done)
    fn step(&mut self) -> bool {
        if self.beams.iter().all(|p| p.0.1 == usize::MAX) {
            return true;
        }
        let mut new_beams = HashMap::new();

        for (beam, timelines) in self.beams.drain() {
            let next_pos = Self::next_pos(&beam, self.height);
            if self.splitters.contains(&next_pos) {
                *new_beams
                    .entry(Position(next_pos.0 - 1, next_pos.1))
                    .or_insert(0) += timelines;
                *new_beams
                    .entry(Position(next_pos.0 + 1, next_pos.1))
                    .or_insert(0) += timelines;
                self.splits += 1;
            } else {
                *new_beams.entry(next_pos).or_insert(0) += timelines;
            }
        }

        self.beams = new_beams;

        false
    }

    // If exited tachyon, set y position to usize::MAX
    fn next_pos(pos: &Position, max_y: usize) -> Position {
        if pos.1 >= max_y - 1 {
            Position(pos.0, usize::MAX)
        } else {
            Position(pos.0, pos.1 + 1)
        }
    }
}
