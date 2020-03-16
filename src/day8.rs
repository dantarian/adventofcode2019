use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::process;

use crate::util;

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let values = util::read_digits(File::open(filename)?);
    let row_length = 25;
    let row_count = 6;
    let slice_size = row_length * row_count;

    if *part2 {
        let mut result = Vec::new();
        for _ in 0..slice_size {
            result.push(None);
        }

        for slice in values.chunks(slice_size) {
            for (index, &val) in slice.iter().enumerate() {
                if let None = result[index] {
                    if val != 2 {
                        result[index] = Some(val);
                    }
                }
            }
        }

        for row in result.chunks(row_length) {
            let row_string = row.iter().map(|x| match x {
                Some(x) if *x == 1 => "*",
                _ => " "
            }).collect::<String>();
            println!("{}", row_string);
        }
    } else {
        let iter = values.chunks(slice_size);
        let stats = iter.map(|chunk| (chunk.iter().filter(|&&x| x == 0).count(),
                                      chunk.iter().filter(|&&x| x == 1).count(),
                                      chunk.iter().filter(|&&x| x == 2).count()));

        let min = stats.min_by(|x, y| x.0.cmp(&(y.0)));

        match min {
            Some(x) => println!("{}", x.1 * x.2),
            None => {
                eprintln!("No minimum value found!");
                process::exit(1);
            }
        }
    }

    Ok(())
}

