use cached::SizedCache;
use cached::proc_macro::cached;
use rayon::prelude::*;
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
    let part1 = graph.count_paths("you", "out");
    let part2 = graph.part2("svr", "out", false, false);

    println!("Part 1: {part1}");
    println!("Part 2: {part2}");

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

    fn count_paths(&self, start: &str, end: &str) -> usize {
        if start == end {
            return 1;
        }

        self.nodes
            .get(start)
            .expect("Node not found!")
            .par_iter()
            .map(|node| self.count_paths(node, end))
            .sum()
    }

    fn part2(&self, start: &str, end: &str, visited_dac: bool, visited_fft: bool) -> usize {
        part2_helper(&self.nodes, start, end, visited_dac, visited_fft)
    }
}

#[cached(
    ty = "SizedCache<String, usize>",
    create = "{ SizedCache::with_size(1000) }",
    convert = r#"{ format!("{}:{},{},{}", start, end, dac, fft) }"#
)]
fn part2_helper(
    nodes: &HashMap<String, HashSet<String>>,
    start: &str,
    end: &str,
    dac: bool,
    fft: bool,
) -> usize {
    if start == end {
        return if dac && fft { 1 } else { 0 };
    }

    nodes
        .get(start)
        .expect("Node not found!")
        .par_iter()
        .map(|node| part2_helper(nodes, node, end, dac || node == "dac", fft || node == "fft"))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let graph = Graph::from_file("../test_input/day11.txt").unwrap();
        assert_eq!(graph.count_paths("you", "out"), 5);
    }

    #[test]
    fn test_part2() {
        let graph = Graph::from_file("../test_input/day11part2.txt").unwrap();
        assert_eq!(graph.part2("svr", "out", false, false), 2);
    }
}
