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
    let f = File::open(dir.join("day03.txt"))?;
    let reader = std::io::BufReader::new(f);
    let lines = reader.lines();

    let mut part1_result = 0;
    let mut part2_result = 0;
    for line in lines {
        let Ok(line) = line else {
            continue;
        };

        let nums = line
            .chars()
            .map(|c| c.to_digit(10).unwrap())
            .collect::<Vec<_>>();
        part1_result += find_highest_num(&nums, 2);
        part2_result += find_highest_num(&nums, 12);
    }

    println!("Part 1: {part1_result}");
    println!("Part 2: {part2_result}");

    Ok(())
}

fn find_highest_num(nums: &[u32], n: usize) -> usize {
    if n > nums.len() {
        panic!("Looking for {n} digits but only have {}", nums.len());
    }
    match n {
        0 => return 0,
        // Base case: pick the highest digit remaining
        1 => return *nums.iter().max().unwrap() as usize,
        _ => {}
    }

    // Only have enough digits left, return early
    if n == nums.len() {
        return nums.iter().fold(0, |acc, n| (acc * 10) + *n as usize);
    }

    // Find the highest digit that is at least n digits away from the end of the array and recurse
    let mut highest = 0;
    let mut index = 0;
    for i in 0..(nums.len() - n + 1) {
        if nums[i] > highest {
            index = i;
            highest = nums[i];
        }
    }
    find_highest_num(&nums[(index + 1)..], n - 1) + (highest as usize * 10usize.pow((n - 1) as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let num = [9, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 1, 1];
        assert_eq!(find_highest_num(&num, 12), 987654321111);
    }

    #[test]
    fn test2() {
        let num = [8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9];
        assert_eq!(find_highest_num(&num, 12), 811111111119);
    }
}
