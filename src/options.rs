use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about="Application for solving Advent of Code 2019 puzzles.")]
pub struct Opt {
    #[structopt(long)]
    /// Specify that Part 2 of the solution is to be run.
    pub part2: bool,

    #[structopt(subcommand)]
    pub cmd: Command
}

#[derive(Debug, StructOpt)]
/// Run the solution for Day 1.
pub enum Command {
    /// Calculate the amount of fuel needed.
    Day1 {
        /// The name of the file to be used for input.
        filename: PathBuf,
    },

    /// Run a simple computer.
    Day2 {
        /// The name of the file to be used for input.
        filename: PathBuf,
    },

    /// Calculate the Manhattan distance to the closest intersection to the origin
    Day3 {
        /// The name of the file to be used for input.
        filename: PathBuf,
    },

    /// Find possible passcodes.
    Day4 {
        /// The start of the range of possible values.
        range_start: u32,
        /// The end of the range of possible values.
        range_end: u32,
    },

    /// Run a slightly more complex computer.
    Day5 {
        /// The name of the file to be used for input.
        filename: PathBuf,
    },

    /// Orbital mechanics.
    Day6 {
        /// The name of the file to be used for input.
        filename: PathBuf,
    },

    /// Amplifier shenanigans
    Day7 {
        /// The name of the file to be used for inuput.
        filename: PathBuf,
    },

    /// Image processing
    Day8 {
        /// The name of the file to be used for input.
        filename: PathBuf,
    },

    /// 64-bit Intcode
    Day9 {
        /// The name of the file to be used for input.
        filename: PathBuf,
    },

    /// Asteroids
    Day10 {
        /// The name of the file to be used for input.
        filename: PathBuf,        
    },
}
