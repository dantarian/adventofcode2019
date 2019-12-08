use std::error::Error;

pub mod options;
pub mod day1;
pub mod day2;
use options::Opt;
use options::Command;

pub fn run(opt: Opt) -> Result<(), Box<dyn Error>> {
    match opt.cmd {
        Command::Day1 { filename } => day1::run_day1(&filename, &opt.part2),
        Command::Day2 { filename } => day2::run(&filename, &opt.part2),
    }
}

