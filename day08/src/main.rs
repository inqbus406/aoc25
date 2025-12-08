use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let (n, dir) = if args.contains(&String::from("--test_input")) {
        (10, PathBuf::from("test_input"))
    } else {
        (1000, PathBuf::from("input"))
    };

    let mut system = System::from_file(&dir.join("day08.txt"))?;
    let connections_sorted = system
        .make_n_closest_connections(n)
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    // Build circuits
    let mut circuit_finder = CircuitFinder::new(system.junction_boxes.len());
    for conn in connections_sorted {
        circuit_finder.join(conn.j0, conn.j1);
    }

    let mut roots_checked = HashSet::new();
    let sorted_circuit_sizes = (0..n)
        .flat_map(|x| {
            let root = circuit_finder.find(x);
            if roots_checked.insert(root) {
                Some(circuit_finder.circuit_size(root))
            } else {
                None
            }
        })
        .sorted()
        .rev()
        .collect::<Vec<_>>();

    let part1 = sorted_circuit_sizes.iter().take(3).product::<usize>();
    println!("Part 1: {part1}");

    // Now do part 2
    let connections_sorted = system.make_n_closest_connections(n * n);

    // Build circuits again
    circuit_finder = CircuitFinder::new(system.junction_boxes.len());
    for conn in connections_sorted {
        circuit_finder.join(conn.j0, conn.j1);
        if circuit_finder.all_connected() {
            let part2 = system.junction_boxes[conn.j0].0 * system.junction_boxes[conn.j1].0;
            println!("Part 2: {part2}");
            break;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct JunctionBox(usize, usize, usize);

// This will find circuits using Disjoint-Set Union (Union-Find)
struct CircuitFinder {
    parent: Vec<usize>,
    size: Vec<usize>,
    num_circuits: usize,
}

impl CircuitFinder {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            size: vec![1; n],
            num_circuits: n,
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] == x {
            x
        } else {
            let parent = self.find(self.parent[x]);
            self.parent[x] = parent;
            parent
        }
    }

    fn join(&mut self, x: usize, y: usize) {
        let mut x_root = self.find(x);
        let mut y_root = self.find(y);

        if x_root == y_root {
            // already in the same circuit
            return;
        }

        // merge x's subtree under y's root
        if self.size[x_root] < self.size[y_root] {
            std::mem::swap(&mut x_root, &mut y_root);
        }
        self.parent[y_root] = x_root;
        self.size[x_root] += self.size[y_root];

        // Merged, decrease number of circuits
        self.num_circuits -= 1;
    }

    fn circuit_size(&mut self, x: usize) -> usize {
        let root = self.find(x);
        self.size[root]
    }

    fn all_connected(&self) -> bool {
        self.num_circuits == 1
    }
}

#[derive(Debug, Clone)]
struct Connection {
    j0: usize,
    j1: usize,
    distance: f32,
}

impl Eq for Connection {}

impl PartialEq<Self> for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.j0 == other.j0 && self.j1 == other.j1 || self.j0 == other.j1 && self.j1 == other.j0
    }
}

impl PartialOrd<Self> for Connection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Connection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.partial_cmp(&other.distance).unwrap()
    }
}

impl Connection {
    fn new(j0: usize, j1: usize, distance: f32) -> Self {
        Self { j0, j1, distance }
    }
}

#[derive(Debug)]
struct System {
    // Vertices
    junction_boxes: Vec<JunctionBox>,
}

impl System {
    fn from_file(path: &impl AsRef<Path>) -> anyhow::Result<Self> {
        let f = File::open(path)?;
        let reader = std::io::BufReader::new(f);

        let mut junction_boxes = Vec::new();
        for line in reader.lines() {
            let Ok(line) = line else {
                continue;
            };
            let xyz = line
                .split(',')
                .map(|s| s.parse::<usize>().unwrap())
                .collect::<Vec<_>>();
            let junction_box = JunctionBox(xyz[0], xyz[1], xyz[2]);
            junction_boxes.push(junction_box);
        }

        Ok(Self { junction_boxes })
    }

    fn make_n_closest_connections(&mut self, n: usize) -> Vec<Connection> {
        let mut heap = BinaryHeap::new();
        for j0 in 0..self.junction_boxes.len() {
            for j1 in (j0 + 1)..self.junction_boxes.len() {
                let distance =
                    Self::euclidean_distance(&self.junction_boxes[j0], &self.junction_boxes[j1]);
                let conn = Connection::new(j0, j1, distance);

                if heap.len() < n {
                    heap.push(conn);
                    continue;
                }

                let current_biggest = heap.peek().unwrap();
                if conn.distance < current_biggest.distance {
                    heap.pop();
                    heap.push(conn);
                }
            }
        }

        heap.into_sorted_vec()
    }

    fn euclidean_distance(v0: &JunctionBox, v1: &JunctionBox) -> f32 {
        let dx = v0.0 as f32 - v1.0 as f32;
        let dy = v0.1 as f32 - v1.1 as f32;
        let dz = v0.2 as f32 - v1.2 as f32;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
