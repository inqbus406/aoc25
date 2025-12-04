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

    let mut map = Map::from_file(dir.join("day04.txt"))?;

    println!("Part 1: {}", map.count_accessible().len());
    println!("Part 2: {}", map.remove_accessible());

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Map {
    rolls: HashSet<Position>,
    x_size: usize,
    y_size: usize,
}

impl Map {
    fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let f = File::open(path)?;
        let reader = std::io::BufReader::new(f);
        let lines = reader.lines();

        let mut rolls = HashSet::new();
        let mut x_size = 0;
        let mut y_size = 0;
        for (y, line) in lines.enumerate() {
            y_size = max(y, y_size);
            for (x, c) in line?.chars().enumerate() {
                x_size = max(x, x_size);
                match c {
                    '@' => {
                        rolls.insert(Position {
                            x: x as i32,
                            y: y as i32,
                        });
                    }
                    _ => {}
                }
            }
        }
        Ok(Self {
            rolls,
            x_size: x_size + 1,
            y_size: y_size + 1,
        })
    }

    fn count_accessible(&self) -> HashSet<Position> {
        let mut accessible = HashSet::new();
        for roll in self.rolls.iter() {
            let mut adjacent_count = 0;
            // println!("Checking {} adjacent rolls", self.get_adjacent(roll).len());
            for adjacent in self.get_adjacent(roll) {
                if self.rolls.contains(&adjacent) {
                    adjacent_count += 1;
                }
            }
            if adjacent_count < 4 {
                accessible.insert(*roll);
            }
        }

        accessible
    }

    fn remove_accessible(&mut self) -> usize {
        let mut result = 0;
        loop {
            let accessible = self.count_accessible();
            if accessible.is_empty() {
                break;
            }
            self.rolls.retain(|p| !accessible.contains(p));
            result += accessible.len();
        }

        result
    }


    fn get_adjacent(&self, pos: &Position) -> Vec<Position> {
        [
            Position {
                x: pos.x - 1,
                y: pos.y,
            },
            Position {
                x: pos.x,
                y: pos.y - 1,
            },
            Position {
                x: pos.x + 1,
                y: pos.y,
            },
            Position {
                x: pos.x,
                y: pos.y + 1,
            },
            Position {
                x: pos.x - 1,
                y: pos.y - 1,
            },
            Position {
                x: pos.x - 1,
                y: pos.y + 1,
            },
            Position {
                x: pos.x + 1,
                y: pos.y - 1,
            },
            Position {
                x: pos.x + 1,
                y: pos.y + 1,
            }
        ]
        .iter()
        .filter(|&p| self.is_valid(p))
        .copied()
        .collect()
    }

    fn is_valid(&self, pos: &Position) -> bool {
        pos.x >= 0 && pos.x < self.x_size as i32 && pos.y >= 0 && pos.y < self.y_size as i32
    }
}
