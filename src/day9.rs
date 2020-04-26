use std::collections::VecDeque;
use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::process;

use crate::util;
use crate::intcode::{Computer, ComputerInput};

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let initial_state = util::read_comma_separated_integers::<File, i64>(File::open(filename)?)?;

    let input = if *part2 {
        VecDeque::from(vec![2i64])
    } else {
        VecDeque::from(vec![1i64])
    };

    let mut computer = Computer::new(initial_state.clone(), Some(ComputerInput::Queue(input)), None);
    match computer.run() {
        Ok(_) => {
            match computer.output().pop_front() {
                Some(element) => { println!("{}", element); },
                None => {
                    eprintln!("No output found from computer!");
                    process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("Problem running computer: {}", e);
            process::exit(1);
        }
    }

    Ok(())
}