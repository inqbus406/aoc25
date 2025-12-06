use anyhow::{anyhow, bail};
use std::cmp::max;
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
    let f = File::open(dir.join("day06.txt"))?;
    let reader = std::io::BufReader::new(f);

    let mut rows = Vec::new();
    let mut lines = reader.lines().flatten().collect::<Vec<_>>();
    let operations_line = lines.pop().unwrap();
    let operations = operations_line
        .split_whitespace()
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    for line in &lines {
        rows.push(
            line.split_whitespace()
                .flat_map(|s| s.parse::<usize>())
                .collect::<Vec<_>>(),
        );
    }

    println!("Part 1: {}", part1(&rows, &operations));
    println!("Part 2: {}", part2(&lines, &operations_line)?);

    Ok(())
}

fn part1(rows: &Vec<Vec<usize>>, operations: &Vec<String>) -> usize {
    let mut total = 0;
    for (i, op) in operations.iter().enumerate() {
        let mut result = 0;
        match op.as_str() {
            "*" => {
                result = 1;
                for row in rows {
                    result *= row[i];
                }
            }
            "+" => {
                for row in rows {
                    result += row[i];
                }
            }
            _ => unreachable!("Unrecognized operation: {}", op),
        }
        total += result;
    }

    total
}

fn part2(lines: &Vec<String>, operations_line: &str) -> anyhow::Result<usize> {
    // Determine operator positions (byte indices) and corresponding chars
    let ops: Vec<(usize, char)> = operations_line
        .char_indices()
        .filter(|(_, c)| !c.is_whitespace())
        .collect();

    if ops.is_empty() {
        bail!("No operations found on the last line");
    }

    // Use the maximum length across all lines to ensure we don't miss trailing digits
    let line_len = lines
        .iter()
        .map(|l| l.len())
        .max()
        .ok_or_else(|| anyhow!("No lines found"))?;

    // Build column ranges using half-open intervals [start, end)
    // Simpler and matches visual layout: each column starts at the operator's
    // own index and ends at the next operator's index (or line_len for the last).
    let mut ranges: Vec<(usize, usize)> = Vec::with_capacity(ops.len());
    for i in 0..ops.len() {
        let start = ops[i].0;
        let end = if i + 1 < ops.len() {
            ops[i + 1].0
        } else {
            line_len
        };
        ranges.push((start, end));
    }

    let mut total = 0;

    for ((start, end), (_, op)) in ranges.into_iter().zip(ops.into_iter()) {
        // For each absolute character position within this column's span [start, end),
        // collect digits from all lines at that exact position to form a number.
        if end < start {
            continue;
        }

        let mut nums_in_col = Vec::new();
        for pos in start..end {
            let mut s = String::new();
            for line in lines {
                let b = line.chars().collect::<Vec<_>>();
                if pos < b.len() && b[pos].is_ascii_digit() {
                    s.push(b[pos]);
                }
            }
            if !s.is_empty() {
                // Safe to unwrap since s contains only digits
                nums_in_col.push(s.parse::<usize>()?);
            }
        }

        match op {
            '*' => {
                total += nums_in_col.iter().product::<usize>();
            }
            '+' => {
                total += nums_in_col.iter().sum::<usize>();
            }
            _ => bail!("Unrecognized operation at column {}", op),
        }
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part2() {
        // Matches the spacing/alignment from test_input/day06.txt
        let lines = vec![
            String::from("123 328  51 64"),
            String::from(" 45 64  387 23"),
            String::from("  6 98  215 314"),
        ];
        let ops = "*   +   *   +";

        let res = part2(&lines, ops).unwrap();
        // 356 * 24 * 1 = 8544
        // 8 + 248 + 369 = 625
        // 175 * 581 * 32 = 3_253_600
        // 4 + 431 + 623 = 1058
        // Total = 3_263_827
        assert_eq!(res, 3_263_827);
    }

    #[test]
    fn test_part2_simple() {
        // Two rows, plus operators, verifies span boundaries and vertical number building
        let lines = vec![String::from(" 9 12"), String::from("34  5")];
        let ops = "+   +"; // ops at indices 0 and 4

        // Column 0 range [0..4):
        // pos0 -> "3", pos1 -> "94", pos2 -> (skip), pos3 -> "1" => 3 + 94 + 1 = 98
        // Column 1 range [4..5):
        // pos4 -> "25" => 25
        // Total = 98 + 25 = 123
        let res = part2(&lines, ops).unwrap();
        assert_eq!(res, 123);
    }
}
