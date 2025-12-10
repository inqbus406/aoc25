use regex::Regex;
use std::collections::{HashMap, HashSet, VecDeque};
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
    let f = File::open(&dir.join("day10.txt"))?;
    let reader = std::io::BufReader::new(f);

    let mut part1 = 0;
    let mut part2 = 0;
    for line in reader.lines() {
        let Ok(line) = line else {
            continue;
        };

        let machine = Machine::from_str(&line)?;
        if let Some(steps) = machine.fewest_steps_to_target() {
            part1 += steps;
        }
        if let Some(steps) = machine.fewest_steps_to_target_joltage() {
            part2 += steps;
        }
    }

    println!("Part 1: {part1}");
    println!("Part 2: {part2}");

    Ok(())
}

struct Machine {
    target: u16,
    transitions: HashSet<u16>,
    target_joltage: Vec<u16>,
    joltage_transitions: HashSet<Vec<u16>>,
}

impl Machine {
    fn from_str(s: &str) -> anyhow::Result<Self> {
        let line_re = Regex::new(
            r"^\[(?P<lights>[.#]+)](?: \(\d+(?:,\d+)*\))+ (?P<joltage>\{\d+(?:,\d+)*})$",
        )?;
        let button_re = Regex::new(r"\(\d+(?:,\d+)*\)")?;

        let mut transitions = HashSet::new();
        let mut target = 0;
        let mut target_joltage = Vec::new();
        let mut joltage_transitions = HashSet::new();

        if let Some(caps) = line_re.captures(s) {
            target = Self::target_from_str(&caps["lights"])?;
            // println!("Target joltage is {}", &caps["joltage"]);
            target_joltage = Self::target_joltage_from_str(&caps["joltage"])?;

            for b in button_re.find_iter(s) {
                transitions.insert(Self::transition_from_str(b.as_str())?);
                joltage_transitions.insert(Self::joltage_transition_from_str(b.as_str())?);
            }
        }

        Ok(Self {
            target,
            transitions,
            target_joltage,
            joltage_transitions,
        })
    }

    fn target_from_str(s: &str) -> anyhow::Result<u16> {
        let result = s.chars().into_iter().enumerate().fold(0, |acc, (i, c)| {
            let mask = match c {
                '#' => 1 << i,
                _ => 0,
            };
            acc ^ mask
        });

        Ok(result)
    }

    fn transition_from_str(s: &str) -> anyhow::Result<u16> {
        let result = s
            .split(['(', ')', ','])
            .filter(|s| !s.is_empty())
            .flat_map(|s| s.parse::<u16>())
            .fold(0, |acc, n| acc | (1 << n));

        Ok(result)
    }

    fn target_joltage_from_str(s: &str) -> anyhow::Result<Vec<u16>> {
        let result = s
            .split(['{', '}', ','])
            .filter(|s| !s.is_empty())
            .flat_map(|s| s.parse::<u16>())
            .collect::<Vec<_>>();

        // println!("Target joltage for {} is {:?}", s, result);

        Ok(result)
    }

    fn joltage_transition_from_str(s: &str) -> anyhow::Result<Vec<u16>> {
        let result = s
            .split(['(', ')', ','])
            .filter(|s| !s.is_empty())
            .flat_map(|s| s.parse::<u16>())
            .collect::<Vec<_>>();

        Ok(result)
    }

    // Returns None if there is no solution
    fn fewest_steps_to_target(&self) -> Option<usize> {
        if self.target == 0 {
            return Some(0);
        }

        // Run BFS so we always get the shortest path. Transitions should be blazing fast because XOR
        let mut fringe = VecDeque::new();

        // Map state to number of steps to reach it
        let mut visited = HashMap::new();
        fringe.push_back(0);
        visited.insert(0, 0);

        while let Some(cur) = fringe.pop_front() {
            let cur_steps = visited[&cur];

            if cur == self.target {
                return Some(cur_steps);
            }

            for next in self.transitions.iter() {
                let next_state = cur ^ next;
                if !visited.contains_key(&next_state) {
                    visited.insert(next_state, cur_steps + 1);
                    fringe.push_back(next_state);
                }
            }
        }

        None
    }

    fn fewest_steps_to_target_joltage(&self) -> Option<usize> {
        if self.target_joltage.iter().all(|n| *n == 0) {
            return Some(0);
        }

        let mut fringe = VecDeque::new();
        let mut visited = HashMap::new();
        fringe.push_back(vec![0; self.target_joltage.len()]);
        visited.insert(vec![0; self.target_joltage.len()], 0);

        // dbg!(&self.target_joltage);

        while let Some(cur) = fringe.pop_front() {
            let cur_steps = visited[&cur];
            dbg!(cur_steps);
            // dbg!(&cur, &cur_steps);

            if cur == self.target_joltage {
                return Some(cur_steps);
            }

            for next in self.joltage_transitions.iter() {
                // dbg!(&next);
                let mut next_state = cur.clone();
                for n in next.iter() {
                    next_state[*n as usize] += 1;
                }
                if next_state.iter().zip(self.target_joltage.iter()).any(|(n1, n2)| n1 > n2) {
                    // Already exceeded at least one count, not worth exploring further
                    continue;
                }
                // dbg!(&next_state);

                if !visited.contains_key(&next_state) {
                    visited.insert(next_state.clone(), cur_steps + 1);
                    fringe.push_back(next_state);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_from_str() {
        assert_eq!(Machine::target_from_str("####").unwrap(), 0b1111);
        assert_eq!(Machine::target_from_str("#..#").unwrap(), 0b1001);
        assert_eq!(Machine::target_from_str("##..").unwrap(), 0b0011);
    }

    #[test]
    fn test_transition_from_str() {
        assert_eq!(Machine::transition_from_str("(0,1,2)").unwrap(), 0b111);
        assert_eq!(Machine::transition_from_str("(0,1)").unwrap(), 0b11);
        assert_eq!(Machine::transition_from_str("(0)").unwrap(), 0b1);
        assert_eq!(Machine::transition_from_str("(0,2)").unwrap(), 0b101);
        assert_eq!(Machine::transition_from_str("(0,3,4)").unwrap(), 0b11001);
    }
}
