use std::collections::VecDeque;
use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::process;

use itertools::Itertools;

use crate::util;
use crate::intcode::Computer;

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let initial_state = util::read_comma_separated_integers(File::open(filename)?)?;

    if *part2 {
        // No action yet!
        Ok(())
    } else {
        let possibilities = (0..5).permutations(5);
        let mut max_result = 0;

        for permutation in possibilities {
            let mut result = 0;
            for phase in permutation {
                let input = VecDeque::from(vec![phase, result]);
                let mut computer = Computer::new(initial_state.clone(), input);
                match computer.run() {
                    Ok(_) => {
                        match computer.output().pop_front() {
                            Some(element) => { result = element; },
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
            }

            if result > max_result {
                max_result = result;
            }
        }

        println!("{}", max_result);

        Ok(())
    }
}

