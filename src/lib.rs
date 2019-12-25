use std::error::Error;

pub mod options;
pub mod day1;
pub mod day2;
pub mod day3;
pub mod day4;
pub mod day5;
pub mod util;
pub mod intcode;
use options::Opt;
use options::Command;

pub fn run(opt: Opt) -> Result<(), Box<dyn Error>> {
    match opt.cmd {
        Command::Day1 { filename } => day1::run_day1(&filename, &opt.part2),
        Command::Day2 { filename } => day2::run(&filename, &opt.part2),
        Command::Day3 { filename } => day3::run(&filename, &opt.part2),
        Command::Day4 { range_start, range_end } => day4::run(range_start, range_end, &opt.part2),
        Command::Day5 { filename } => day5::run(&filename, &opt.part2),
    }
}

