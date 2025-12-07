use std::cmp::max;
use std::collections::HashSet;
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

    Ok(())
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct Position(usize, usize);

#[derive(Debug)]
struct Map {
    splitters: HashSet<Position>,
    width: usize,
    height: usize,
    beams: HashSet<Position>,
    splits: usize,
}

impl Map {
    fn from_file(path: &impl AsRef<Path>) -> anyhow::Result<Self> {
        let f = File::open(path)?;
        let reader = std::io::BufReader::new(f);

        let mut width = 0;
        let mut height = 0;
        let mut splitters = HashSet::new();

        // beams should start with just 1 position, the starting position S
        let mut beams = HashSet::new();

        for (y, line) in reader.lines().enumerate() {
            let line = line?;
            width = line.len();

            for (x, c) in line.chars().enumerate() {
                let p = Position(x, y);

                match c {
                    'S' => {
                        beams.insert(p);
                    },
                    '^' => {
                        splitters.insert(p);
                    },
                    _ => {}
                }
            }

            height = max(height, y + 1);
        }

        Ok(Self {
            splitters,
            width,
            height,
            beams,
            splits: 0,
        })
    }

    // Returns true if all beams have exited tachyon (simulation is done)
    fn step(&mut self) -> bool {
        if self.beams.iter().all(|p| p.1 == usize::MAX) {
            return true;
        }
        let mut new_beams = HashSet::new();

        for beam in self.beams.drain() {
            let next_pos = Self::next_pos(&beam, self.height);
            if self.splitters.contains(&next_pos) {
                new_beams.insert(Position(next_pos.0 - 1, next_pos.1));
                new_beams.insert(Position(next_pos.0 + 1, next_pos.1));
                self.splits += 1;
            } else {
                new_beams.insert(next_pos);
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
