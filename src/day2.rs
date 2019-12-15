use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::process;

use crate::util;
use crate::intcode::Computer;

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let mut initial_state = util::read_comma_separated_integers(File::open(filename)?)?;

    if *part2 {
        let target: i32 = 19690720;

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

