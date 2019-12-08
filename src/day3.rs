use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Read};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::process;

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let vec = read(File::open(filename)?)?;
    if vec.len() != 2 {
        eprintln!("Incorrect number of lines in input file. Expected 2, got {}.", vec.len());
        process::exit(1)
    }

    let wire1 = build_wire(&vec[0])?;
    let wire2 = build_wire(&vec[1])?;
    let wire_panel = WirePanel::new(wire1, wire2);
    println!("Result: {}", wire_panel.closest_intersection());

    Ok(())
}

fn read<R: Read>(io: R) -> Result<Vec<Vec<String>>, std::io::Error> {
  let br = BufReader::new(io);
  br.lines()
    .map(|r| r.and_then(|s| Ok(s.split(",").map(|s| String::from(s)).collect())))
    .collect()
}

fn build_wire(input: &Vec<String>) -> Result<BTreeSet<Location>, std::io::Error> {
    let mut set = BTreeSet::new();
    let mut current_location = Location::new(0, 0);
    for segment in input {
        let (direction, number_str) = segment.split_at(1);
        let number = number_str.parse::<u32>();
        for _ in 0..number.unwrap() {
            current_location = match direction {
                "U" => Location::new(current_location.x + 0, current_location.y + 1),
                "D" => Location::new(current_location.x + 0, current_location.y - 1),
                "L" => Location::new(current_location.x - 1, current_location.y + 0),
                "R" => Location::new(current_location.x + 1, current_location.y + 0),
                x => return Err(std::io::Error::new(ErrorKind::InvalidData,
                                                    format!("Unexpected direction: {}", x)))
            };
            set.insert(current_location);
        }
    }
    Ok(set)
}

#[derive(Copy, Clone, Debug)]
struct Location {
    x: i32,
    y: i32
}

impl Location {
    fn new(x: i32, y: i32) -> Self {
        Location { x: x, y: y }
    }

    fn distance(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.distance() == other.distance()
    }
}

impl Eq for Location {}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_dist = self.distance();
        let other_dist = other.distance();
        if self_dist == other_dist {
            if self.x == other.x {
                self.y.cmp(&other.y)
            } else {
                self.x.cmp(&other.x)
            }
        } else {
            self_dist.cmp(&other_dist)
        }
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct WirePanel {
    wire1: BTreeSet<Location>,
    wire2: BTreeSet<Location>
}

impl WirePanel {
    fn new(wire1: BTreeSet<Location>, wire2: BTreeSet<Location>) -> Self {
        WirePanel { wire1: wire1, wire2: wire2 }
    }

    fn closest_intersection(&self) -> i32 {
        self.wire1.intersection(&self.wire2).next().unwrap().distance()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let location = Location::new(3, 3);
        assert_eq!(6, location.distance());
    }

    #[test]
    fn test_intersection() {
        let wire1 = build_wire(&vec![String::from("R8"), 
                                     String::from("U5"), 
                                     String::from("L5"), 
                                     String::from("D3")]).unwrap();
        let wire2 = build_wire(&vec![String::from("U7"), 
                                     String::from("R6"), 
                                     String::from("D4"), 
                                     String::from("L4")]).unwrap();

        let panel = WirePanel::new(wire1, wire2);
        assert_eq!(6, panel.closest_intersection());
    }
}
