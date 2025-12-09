use std::cmp::{max, min};
use itertools::Itertools;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use cached::proc_macro::cached;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let dir = if args.contains(&String::from("--test_input")) {
        PathBuf::from("test_input")
    } else {
        PathBuf::from("input")
    };
    let f = File::open(&dir.join("day09.txt"))?;
    let mut reader = BufReader::new(f);

    let mut points = HashSet::new();

    // Just the green edge points
    // Technically this is including the corner red tiles too but it makes it easier that way
    let mut green_points: HashSet<Point> = HashSet::new();
    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    let first_point = point_from_line(&buffer);
    let mut last = first_point.clone();
    for line in reader.lines() {
        let Ok(line) = line else {
            continue;
        };

        let cur = point_from_line(&line);
        green_points.extend(points_between(&last, &cur));

        points.insert(cur);
        last = cur;
    }
    // Connect last and first
    green_points.extend(points_between(&last, &first_point));

    let part1 = part1(&points);
    println!("Part 1: {part1}");

    let part2 = part2(&points, &green_points);
    println!("Part 2: {part2}");

    // draw_boundary(&green_points);
    // draw_boundary(&points);

    Ok(())
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone)]
struct Point(usize, usize);

fn part1(points: &HashSet<Point>) -> usize {
    points
        .iter()
        .combinations(2)
        .map(|pair| rectangle_area(pair[0], pair[1]))
        .max()
        .unwrap()
}

fn part2(
    corners: &HashSet<Point>,
    perimeter: &HashSet<Point>,
) -> usize {
    // Prepare data for parallel processing and progress tracking
    let pts: Vec<Point> = corners.iter().copied().collect();
    let n = pts.len();
    let total_pairs: u64 = (n as u64) * ((n as u64) - 1) / 2;
    eprintln!("Starting Part 2 search ({} pairs)...", total_pairs);

    // Progress reporter (very low overhead: row-granular)
    let processed = Arc::new(AtomicU64::new(0));
    let start = Instant::now();
    {
        let processed = Arc::clone(&processed);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(500));
                let done = processed.load(Ordering::Relaxed);
                // if done == 0 {
                //     continue;
                // }
                let p = (done as f64) / (total_pairs as f64);
                let elapsed = start.elapsed().as_secs_f64();
                let eta = if p > 0.0 { elapsed * (1.0 - p) / p } else { f64::INFINITY };
                eprint!("\r{:.2}% | ETA ~{:.1}s", p * 100.0, eta);
                let _ = std::io::stderr().flush();
                if done >= total_pairs {
                    break;
                }
            }
        });
    }

    let best = (0..n)
        .into_par_iter()
        .map(|i| {
            let p1 = pts[i];
            let mut local_best = 0usize;
            for j in (i + 1)..n {
                let p2 = pts[j];
                if let Some(area) = contained_rectangle_area(&p1, &p2, perimeter) {
                    if area > local_best {
                        local_best = area;
                    }
                }
            }
            // Update progress once per completed outer row
            processed.fetch_add((n - 1 - i) as u64, Ordering::Relaxed);
            local_best
        })
        .max()
        .expect("No contained rectangles found!");

    eprintln!("\nProcessed all pairs in {:.2}s", start.elapsed().as_secs_f64());
    best
}

// Return None if any part of the rectangle is outside the perimeter
fn contained_rectangle_area(
    p1: &Point,
    p2: &Point,
    perimeter: &HashSet<Point>,
) -> Option<usize> {
    let area = rectangle_area(p1, p2);

    let x1 = min(p1.0, p2.0);
    let y1 = min(p1.1, p2.1);
    let x2 = max(p1.0, p2.0);
    let y2 = max(p1.1, p2.1);

    // Check the rectangle boundary only (top, bottom, left, right edges)
    // Corners are shared between edges; rechecking is harmless.

    // Top and bottom edges
    for x in x1..=x2 {
        if !in_perimeter(&Point(x, y1), perimeter) { return None; }
        if !in_perimeter(&Point(x, y2), perimeter) { return None; }
    }

    // Left and right edges
    for y in y1..=y2 {
        if !in_perimeter(&Point(x1, y), perimeter) { return None; }
        if !in_perimeter(&Point(x2, y), perimeter) { return None; }
    }

    Some(area)
}

// -----------------------------------------------------------------------------
// Previous implementation (full interior scan) kept here for easy revert.
// To revert, replace the current function body above with the one below.
//
// fn contained_rectangle_area(
//     p1: &Point,
//     p2: &Point,
//     perimeter: &HashSet<Point>,
// ) -> Option<usize> {
//     let area = rectangle_area(p1, p2);
//
//     let start_x = min(p1.0, p2.0);
//     let start_y = min(p1.1, p2.1);
//     let end_x = max(p1.0, p2.0);
//     let end_y = max(p1.1, p2.1);
//     for y in start_y..=end_y {
//         for x in start_x..=end_x {
//             if !in_perimeter(&Point(x, y), perimeter) {
//                 return None;
//             }
//         }
//     }
//     Some(area)
// }

#[cached(
    key = "((usize, usize), usize)",
    convert = r#"{ ((p.0, p.1), perimeter as *const _ as usize) }"#
)]
fn in_perimeter(p: &Point, perimeter: &HashSet<Point>) -> bool {
    // If the point is exactly on the perimeter, consider it inside
    if perimeter.contains(p) {
        return true;
    }

    // Ray-casting to the right: count intersections with VERTICAL edges only.
    // The perimeter set contains all grid points along the boundary (axis-aligned).
    // A vertical edge crossing at column x for scanline y is identified when both (x, y) and (x, y+1)
    // are on the perimeter. This corresponds to the half-open interval rule [y, y+1) to avoid
    // double-counting at vertices (we include the bottom, exclude the top of vertical segments).
    let max_x = perimeter.iter().map(|pt| pt.0).max().unwrap();

    let y = p.1;
    let crossings = ((p.0 + 1)..=max_x)
        .filter(|&x| {
            let here = Point(x, y);
            let above = Point(x, y + 1);
            perimeter.contains(&here) && perimeter.contains(&above)
        })
        .count();

    crossings % 2 == 1
}

fn rectangle_area(p1: &Point, p2: &Point) -> usize {
    (p2.0.abs_diff(p1.0) + 1) * (p2.1.abs_diff(p1.1) + 1)
}

fn point_from_line(line: &str) -> Point {
    let coords = line
        .split(',')
        .map(|x| x.trim().parse::<usize>().unwrap())
        .collect::<Vec<_>>();

    Point(coords[0], coords[1])
}

// Points MUST be on a line!
fn points_between(p1: &Point, p2: &Point) -> HashSet<Point> {
    let mut points = HashSet::new();
    if p1.0 == p2.0 {
        let start = min(p1.1, p2.1);
        let end = max(p1.1, p2.1);
        for y in start..=end {
            points.insert(Point(p1.0, y));
        }
    } else if p1.1 == p2.1 {
        let start = min(p1.0, p2.0);
        let end = max(p1.0, p2.0);
        for x in start..=end {
            points.insert(Point(x, p1.1));
        }
    } else {
        panic!("Points not on a line!");
    }
    // println!("{} points between {:?} and {:?}", points.len(), p1, p2);

    points
}

fn draw_boundary(points: &HashSet<Point>) {
    let max_x = points.iter().map(|p| p.0).max().unwrap() + 3;
    let max_y = points.iter().map(|p| p.1).max().unwrap() + 3;
    for y in 0..=max_y {
        for x in 0..=max_x {
            if points.contains(&Point(x, y)) {
                print!("#");
            } else {
                print!(".");
            }
        }
        println!();
    }
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
