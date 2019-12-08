use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Read};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
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

    if *part2 {
        println!("Result: {}", wire_panel.closest_combined_distance());
    } else {
        println!("Result: {}", wire_panel.closest_intersection());
    }

    Ok(())
}

fn read<R: Read>(io: R) -> Result<Vec<Vec<String>>, std::io::Error> {
  let br = BufReader::new(io);
  br.lines()
    .map(|r| r.and_then(|s| Ok(s.split(",").map(|s| String::from(s)).collect())))
    .collect()
}

fn build_wire(input: &Vec<String>) -> Result<Wire, std::io::Error> {
    let mut wire = Wire::new();
    let mut current_location = Location::new(0, 0);
    let mut distance_travelled = 0;
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
            distance_travelled = distance_travelled + 1;
            wire.add_point(current_location, distance_travelled);
        }
    }
    Ok(wire)
}

#[derive(Hash, Copy, Clone, Debug)]
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

struct Wire {
    locations: BTreeSet<Location>,
    distances: HashMap<Location, u32>
}

impl Wire {
    fn new() -> Self {
        Wire { locations: BTreeSet::new(), distances: HashMap::new() }
    }

    fn add_point(&mut self, location: Location, distance: u32) {
        self.locations.insert(location.clone());
        self.distances.insert(location, distance);
    }

    fn distance(&self, location: &Location) -> u32 {
        self.distances.get(location).unwrap().clone()
    }
}

struct WirePanel {
    wire1: Wire,
    wire2: Wire
}

impl WirePanel {
    fn new(wire1: Wire, wire2: Wire) -> Self {
        WirePanel { wire1: wire1, wire2: wire2 }
    }

    fn intersections(&self) -> Vec<Location> {
        self.wire1.locations.intersection(&self.wire2.locations).map(|l| l.clone()).collect()
    }

    fn closest_intersection(&self) -> i32 {
        self.wire1.locations.intersection(&self.wire2.locations).next().unwrap().distance()
    }

    fn closest_combined_distance(&self) -> u32 {
        self.intersections().into_iter().map(|l| self.wire1.distance(&l) + self.wire2.distance(&l)).min().unwrap()
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

    #[test]
    fn test_combined_distance() {
        let wire1 = build_wire(&vec![String::from("R8"), 
                                     String::from("U5"), 
                                     String::from("L5"), 
                                     String::from("D3")]).unwrap();
        let wire2 = build_wire(&vec![String::from("U7"), 
                                     String::from("R6"), 
                                     String::from("D4"), 
                                     String::from("L4")]).unwrap();

        let panel = WirePanel::new(wire1, wire2);
        assert_eq!(30, panel.closest_combined_distance());
    }
}
