use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Read};

pub fn run_day1(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let vec = read(File::open(filename)?)?;

    let total: u32 = if *part2 {
        vec.iter().map(|m| more_fuel(&m)).sum()
    } else {
        vec.iter().map(|m| fuel(&m)).sum()
    };

    println!("Total fuel: {}", total);

    Ok(())
}

fn read<R: Read>(io: R) -> Result<Vec<u32>, std::io::Error> {
  let br = BufReader::new(io);
  br.lines()
    .map(|line| line.and_then(|v| v.parse().map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))))
    .collect()
}

fn fuel(mass: &u32) -> u32 {
    (*mass / 3) - 2
}

fn more_fuel(amount: &u32) -> u32 {
    if *amount < 9 {
        return 0;
    }

    return fuel(amount) + more_fuel(&fuel(amount));
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fuel_test1() {
    assert_eq!(2, fuel(&12));
  }

  #[test]
  fn fuel_test2() {
    assert_eq!(2, fuel(&14));
  }

  #[test]
  fn fuel_test3() {
    assert_eq!(654, fuel(&1969));
  }

  #[test]
  fn fuel_test4() {
    assert_eq!(33583, fuel(&100756));
  }

  #[test]
  fn more_fuel_test1() {
      assert_eq!(0, more_fuel(&8));
  }

  #[test]
  fn more_fuel_test2() {
      assert_eq!(1, more_fuel(&9));
  }
  
  #[test]
  fn more_fuel_test3() {
      assert_eq!(16763, more_fuel(&33583));
  }
}
