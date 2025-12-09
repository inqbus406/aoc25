use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use itertools::Itertools;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let dir = if args.contains(&String::from("--test_input")) {
        PathBuf::from("test_input")
    } else {
        PathBuf::from("input")
    };
    let f = File::open(&dir.join("day09.txt"))?;
    let reader = BufReader::new(f);

    let mut points = HashSet::new();
    for line in reader.lines() {
        let Ok(line) = line else {
            continue;
        };

        let coords = line.split(',').map(|x| x.parse::<usize>().unwrap()).collect::<Vec<_>>();
        points.insert(Point(coords[0], coords[1]));
    }

    let part1 = part1(&points);
    println!("Part 1: {part1}");

    Ok(())
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone)]
struct Point(usize, usize);

fn part1(points: &HashSet<Point>) -> usize {
    points.iter().combinations(2)
        .map(|pair| rectangle_area(pair[0], pair[1]))
        .max()
        .unwrap()
}

fn rectangle_area(p1: &Point, p2: &Point) -> usize {
    (p2.0.abs_diff(p1.0) + 1) * (p2.1.abs_diff(p1.1) + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_area() {
        assert_eq!(rectangle_area(&Point(2, 5), &Point(9, 7)), 24);
        assert_eq!(rectangle_area(&Point(7, 1), &Point(11, 7)), 35);
        assert_eq!(rectangle_area(&Point(7, 3), &Point(2, 3)), 6);
    }
}
