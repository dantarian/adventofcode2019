use std::collections::VecDeque
use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::process;

use crate::util;
use crate::intcode::Computer;

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let initial_state = util::read_comma_separated_integers(File::open(filename)?)?;

    if *part2 {
        // No action yet!
        Ok(())
    } else {
        let mut computer = Computer::new(initial_state, VecDeque::new());

        let result = computer.run();
        
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Problem running computer: {}", e);
                process::exit(1);
            }
        }
    }
}

