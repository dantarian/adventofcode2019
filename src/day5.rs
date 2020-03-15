use std::collections::VecDeque;
use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::process;

use crate::util;
use crate::intcode::{Computer, ComputerInput};

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let initial_state = util::read_comma_separated_integers(File::open(filename)?)?;

    if *part2 {
        let mut input = VecDeque::new();
        input.push_back(5);
        let mut computer = Computer::new(initial_state, Some(ComputerInput::Queue(input)), None);

        let result = computer.run();
        
        match result {
            Ok(_) => {
                println!("Output:");
                for value in computer.output() {
                    println!("{}", value);
                }
                Ok(())
            },
            Err(e) => {
                eprintln!("Problem running computer: {}", e);
                process::exit(1);
            }
        }
    } else {
        let mut input = VecDeque::new();
        input.push_back(1);
        let mut computer = Computer::new(initial_state, Some(ComputerInput::Queue(input)), None);

        let result = computer.run();
        
        match result {
            Ok(_) => {
                println!("Output:");
                for value in computer.output() {
                    println!("{}", value);
                }
                Ok(())
            },
            Err(e) => {
                eprintln!("Problem running computer: {}", e);
                process::exit(1);
            }
        }
    }
}

