use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::process;
use std::io::{BufRead, BufReader, ErrorKind, Read};

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let mut initial_state = read(File::open(filename)?)?;

    if *part2 {
        let target: usize = 19690720;

        for noun in 0..100 {
            for verb in 0..100 {
                let mut run_initial_state = initial_state.clone();
                run_initial_state[1] = noun;
                run_initial_state[2] = verb;
                let mut computer = Computer::new(run_initial_state);
                let result = computer.run();

                match result {
                    Ok(x) if x == target => {
                        println!("Result: {}", 100 * noun + verb);
                        process::exit(0);
                    },
                    Ok(_) => println!("Missed for noun={}, verb={}", noun, verb),
                    Err(_) => println!("Errored for noun={}, verb={}", noun, verb)
                };
            }
        }
    } else {
        initial_state[1] = 12;
        initial_state[2] = 2;

        let mut computer = Computer::new(initial_state);

        let result = computer.run();
        
        match result {
            Ok(x) => println!("Result: {}", x),
            Err(e) => {
                eprintln!("Problem running computer: {}", e);
                process::exit(1);
            }
        };
    }

    Ok(())
}

fn read<R: Read>(io: R) -> Result<Vec<usize>, std::io::Error> {
  let br = BufReader::new(io);
  br.split(b',')
    .map(|r| r.and_then(|v| String::from_utf8(v).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))))
    .map(|r| r.unwrap())
    .map(|s| String::from(s.trim()))
    .filter(|s| s.len() > 0)
    .map(|s| s.parse::<usize>().map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e)))
    .collect()
}

#[derive(Debug)]
struct Computer {
    memory: Vec<usize>,
    loc: usize,
    running: bool,
}

impl Computer {
    fn new(memory: Vec<usize>) -> Self {
        Computer { memory: memory, loc: 0, running: true }
    }

    fn run(&mut self) -> Result<usize, &'static str> {
        while self.running {
            self.step()?;
        }

        self.result()
    }

    fn step(&mut self) -> Result<(), &'static str> {
        let (target, value) = match self.memory.get(self.loc) {
            Some(1) => 
                (self.memory[self.loc + 3], self.mem_lookup(self.loc + 1) + self.mem_lookup(self.loc + 2)),
            Some(2) =>
                (self.memory[self.loc + 3], self.mem_lookup(self.loc + 1) * self.mem_lookup(self.loc + 2)),
            Some(99) => {
                self.running = false;
                (0, 0)
            },
            _ => return Err("Unexpected register value!")
        };

        if self.running {
            self.loc = self.loc + 4;
            self.memory[target] = value;
        }

        Ok(())
    }

    fn mem_lookup(&self, location: usize) -> usize {
        self.memory[self.memory[location]]
    }

    fn result(&self) -> Result<usize, &'static str> {
        match self.memory.get(0) {
            Some(a) => Ok(a.clone()),
            _ => Err("Empty memory!")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_single_add() {
        let mut computer = Computer::new(vec![1, 0, 0, 0, 99]);
        computer.step().is_ok();
        assert_eq!(vec![2, 0, 0, 0, 99], computer.memory);
    }

    #[test]
    fn test_step_single_mutiply() {
        let mut computer = Computer::new(vec![2, 3, 0, 3, 99]);
        computer.step().is_ok();
        assert_eq!(vec![2, 3, 0, 6, 99], computer.memory);
    }

    #[test]
    fn test_step_single_mutiply_long() {
        let mut computer = Computer::new(vec![2, 4, 4, 5, 99, 0]);
        computer.step().is_ok();
        assert_eq!(vec![2, 4, 4, 5, 99, 9801], computer.memory);
    }

    #[test]
    fn test_run() {
        let mut computer = Computer::new(vec![1, 1, 1, 4, 99, 5, 6, 0, 99]);
        assert_eq!(30, computer.run().unwrap());
    }
}
