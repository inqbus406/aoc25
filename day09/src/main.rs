use cached::proc_macro::cached;
use itertools::Itertools;
use rayon::prelude::*;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

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
    // Technically this is including the corner red tiles too, but it makes it easier that way
    let mut green_points: HashSet<Point> = HashSet::new();

    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    let first_point = point_from_line(&buffer).expect("invalid first point");
    
    let mut last = first_point.clone();
    for line in reader.lines() {
        let Ok(line) = line else {
            continue;
        };

        if let Some(cur) = point_from_line(&line) {
            green_points.extend(points_between(&last, &cur));

            points.insert(cur);
            last = cur;
        }
    }
    // Connect last and first
    green_points.extend(points_between(&last, &first_point));

    let part1 = part1(&points);
    println!("Part 1: {part1}");

    let part2 = part2(&points, &green_points);
    println!("Part 2: {part2}");

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

fn part2(corners: &HashSet<Point>, perimeter: &HashSet<Point>) -> usize {
    // Global bounding box of the polygon for quick rejects
    let (min_x, max_x) = perimeter.iter().fold((usize::MAX, 0usize), |(lo, hi), p| {
        (lo.min(p.0), hi.max(p.0))
    });
    let (min_y, max_y) = perimeter.iter().fold((usize::MAX, 0usize), |(lo, hi), p| {
        (lo.min(p.1), hi.max(p.1))
    });

    // Precompute scanlines
    let scanlines = build_scanline_crossings(perimeter, min_y, max_y);

    // Track global best to cheaply prune pairs with area <= best so far
    let global_best = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();

    let combinations = corners.iter().combinations(2).collect::<Vec<_>>();

    let result = combinations
        .par_iter()
        .filter_map(|points| {
            let p1 = points[0];
            let p2 = points[1];
            let mut local_best = 0usize;

            // Cheap prune: if the area cannot beat the best so far, skip expensive checks
            let area = rectangle_area(&p1, &p2);
            let best_so_far = global_best.load(Ordering::Relaxed);
            if area <= best_so_far || area <= local_best {
                return None;
            }

            // Bounding box prune: if rectangle extends beyond polygon bbox, skip
            let x1 = min(p1.0, p2.0);
            let x2 = max(p1.0, p2.0);
            let y1 = min(p1.1, p2.1);
            let y2 = max(p1.1, p2.1);
            if x1 < min_x || x2 > max_x || y1 < min_y || y2 > max_y {
                return None;
            }

            if let Some(area) =
                contained_rectangle_area_scanline(&p1, &p2, perimeter, &scanlines, min_y)
            {
                if area > local_best {
                    local_best = area;
                    // Update global best as we go to improve pruning for other threads
                    let mut cur = best_so_far;
                    while area > cur {
                        match global_best.compare_exchange(
                            cur,
                            area,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(actual) => cur = actual,
                        }
                    }
                }
            }
            Some(local_best)
        })
        .max()
        .expect("No contained rectangles found!");

    let elapsed = start.elapsed().as_secs_f64();
    eprintln!("\nProcessed all pairs in {:.2}s", elapsed,);

    result
}

// Return None if any part of the rectangle is outside the perimeter
fn contained_rectangle_area_scanline(
    p1: &Point,
    p2: &Point,
    perimeter: &HashSet<Point>,
    scanlines: &Vec<Vec<usize>>, // sorted crossing x positions per y row (indexed by y - min_y)
    min_row_y: usize,
) -> Option<usize> {
    let area = rectangle_area(p1, p2);

    let x1 = min(p1.0, p2.0);
    let y1 = min(p1.1, p2.1);
    let x2 = max(p1.0, p2.0);
    let y2 = max(p1.1, p2.1);

    // Take a single interior sample point using fast scanlines parity; if outside, reject early
    let sample_x = (x1 + x2) / 2;
    let sample_y = (y1 + y2) / 2;
    if !point_inside_fast(Point(sample_x, sample_y), perimeter, scanlines, min_row_y) {
        return None;
    }

    // Validate the entire rectangle boundary
    // Top and bottom edges
    for x in x1..=x2 {
        let top = Point(x, y1);
        if !point_inside_fast(top, perimeter, scanlines, min_row_y) {
            return None;
        }
        let bottom = Point(x, y2);
        if !point_inside_fast(bottom, perimeter, scanlines, min_row_y) {
            return None;
        }
    }
    // Left and right edges
    for y in y1..=y2 {
        let left = Point(x1, y);
        if !point_inside_fast(left, perimeter, scanlines, min_row_y) {
            return None;
        }
        let right = Point(x2, y);
        if !point_inside_fast(right, perimeter, scanlines, min_row_y) {
            return None;
        }
    }

    Some(area)
}

// Cache the maximum x for a given perimeter instance to avoid O(P) work per query.
#[cached(key = "usize", convert = r#"{ perimeter as *const _ as usize }"#)]
fn perimeter_max_x(perimeter: &HashSet<Point>) -> usize {
    perimeter.iter().map(|pt| pt.0).max().unwrap()
}

// Build, for each scanline y, the sorted list of x columns where a vertical edge crosses [y, y+1).
// Index in the returned Vec is (y - min_y).
fn build_scanline_crossings(
    perimeter: &HashSet<Point>,
    min_y: usize,
    max_y: usize,
) -> Vec<Vec<usize>> {
    let rows = max_y - min_y + 1; // max_y is inclusive
    let mut scanlines: Vec<Vec<usize>> = vec![Vec::new(); rows];

    for p in perimeter.iter() {
        let x = p.0;
        let y = p.1;
        if y < max_y {
            // Make sure it's a vertical edge
            if perimeter.contains(&Point(x, y + 1)) {
                scanlines[y - min_y].push(x);
            }
        }
    }
    for xs in &mut scanlines {
        xs.sort_unstable();
    }

    scanlines
}

// Fast inside test using precomputed scanline crossings. Boundary counts as inside.
fn point_inside_fast(
    p: Point,
    perimeter: &HashSet<Point>,
    scanlines: &Vec<Vec<usize>>,
    min_row_y: usize,
) -> bool {
    if perimeter.contains(&p) {
        return true;
    }
    if p.1 < min_row_y {
        return false;
    }
    let row_index = p.1 - min_row_y;
    if row_index >= scanlines.len() {
        return false;
    }
    let crossings = &scanlines[row_index];
    // Count crossings to the right: number of x > p.x
    let index = crossings.partition_point(|&x| x <= p.0);
    let right_crossings = crossings.len() - index;
    right_crossings % 2 == 1
}

fn rectangle_area(p1: &Point, p2: &Point) -> usize {
    (p2.0.abs_diff(p1.0) + 1) * (p2.1.abs_diff(p1.1) + 1)
}

fn point_from_line(line: &str) -> Option<Point> {
    let coords = line
        .split(',')
        .flat_map(|x| x.trim().parse::<usize>())
        .collect::<Vec<_>>();

    if coords.len() != 2 {
        None
    } else {
        Some(Point(coords[0], coords[1]))
    }
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

    points
}

#[allow(dead_code)]
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
