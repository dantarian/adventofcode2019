use std::process;
use structopt::StructOpt;

mod options;
pub use adventofcode2019::options::Opt;

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    if let Err(e) = adventofcode2019::run(opt) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    };
}
