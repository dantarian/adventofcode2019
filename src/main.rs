use std::process;
use structopt::StructOpt;

mod options;
pub use adventlib::options::Opt;

extern crate num;

fn main() {
    let opt = Opt::from_args();

    if let Err(e) = adventlib::run(opt) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    };
}
