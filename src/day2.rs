use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Read};

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let vec = read(File::open(filename)?)?;
    let mut computer = Computer::new(vec);

    while computer.running {
        computer = run_instruction(computer).unwrap();
    }

    println!("Result: {}", computer.memory[0]);

    Ok(())
}

struct Computer {
    memory: mut Vec<u32>,
    loc: mut u32,
    running: mut bool,
}

impl Computer {
    fn new(memory: Vec<u32>) -> Result<Self> {
        Ok(Computer { memory: memory, loc: 0, running: true })
    }

    fn step(self) -> Result<Self> {
        match self.memory.get(self.loc) {
            Some(1) => self.memory[self.loc + 3] = self.memory[self.loc + 1] + self.memory[self.loc + 2],
            Some(2) => self.memory[self.loc + 3] = self.memory[self.loc + 1] * self.memory[self.loc + 2],
            Some(99) => self.running = false,
            _ => return Err("Unexpected register value!")
        }

        if self.running {
            self.loc = self.loc + 4;
        }

        Ok(self)
    }

    fn result(self) -> Result<u32> {
        match self.memory.get(0) {
            Some(a) => Ok(a),
            _ => Err("Empty memory!")
        }
    }
}

fn read<R: Read>(io: R) -> Result<Vec<u32>, std::io::Error> {
  let br = BufReader::new(io);
  br.lines()
    .map(|line| line.and_then(|v| v.parse().map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))))
    .collect()
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
