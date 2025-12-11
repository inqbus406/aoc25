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

    let graph = Graph::from_file(&dir.join("day11.txt"))?;
    let part1 = graph.paths_you_to_out("you", "out");

    println!("Part 1: {part1}");

    Ok(())
}

#[derive(Debug)]
struct Graph {
    nodes: HashMap<String, HashSet<String>>,
}

impl Graph {
    fn from_file(p: impl AsRef<Path>) -> anyhow::Result<Self> {
        let f = File::open(p)?;
        let reader = std::io::BufReader::new(f);

        let mut nodes = HashMap::new();

        for line in reader.lines() {
            let Ok(line) = line else {
                continue;
            };

            let parts = line
                .split([' ', ':'])
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();

            let vertex = parts[0].to_string();
            let mut outputs = HashSet::new();
            for edge in &parts[1..] {
                outputs.insert(edge.to_string());
            }

            nodes.insert(vertex, outputs);
        }

        Ok(Self { nodes })
    }

    fn paths_you_to_out(&self, start: &str, end: &str) -> usize {
        if start == end {
            return 1;
        }

        self.nodes
            .get(start)
            .expect("Node not found!")
            .iter()
            .map(|node| self.paths_you_to_out(node, end))
            .sum()
    }
}
