use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::convert::TryFrom;
use num::integer::gcd;

use crate::util::manhattan_distance;

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let lines = read(File::open(filename)?)?;
    let asteroids = find_asteroids(lines)?;

    if *part2 {
        // Do nothing for now.
    } else {
        match asteroids.iter().map(|x| count_visible(x, &asteroids)).max() {
            Some(max) => {
                println!("Asteroid with most lines-of-sight can see {} asteroids.", max);
            },
            None => {
                eprintln!("No asteroids found!");
            }
        }
    }

    Ok(()) 
}

fn read<R: Read>(io: R) -> Result<Vec<String>, std::io::Error> {
    let br = BufReader::new(io);
    br.lines().collect()
}

fn find_asteroids(lines: Vec<String>) -> Result<Vec<(isize, isize)>, <isize as TryFrom<usize>>::Error> {
    let mut asteroids: Vec<(isize, isize)> = vec![];
    for (rindex, row) in lines.iter().enumerate() {
        for (cindex, c) in row.chars().enumerate() {
            if c == '#' {
                asteroids.push((isize::try_from(cindex)?, isize::try_from(rindex)?));
            }
        }
    }

    Ok(asteroids)
}

fn count_visible(current_asteroid: &(isize, isize), asteroids: &Vec<(isize, isize)>) -> usize {
    let mut sorted_asteroids = asteroids.clone();
    sorted_asteroids.sort_by_cached_key(|a| manhattan_distance(*current_asteroid, *a));
    let mut encountered_asteroids = vec![];
    for asteroid in sorted_asteroids {
        if asteroid == *current_asteroid {
            continue;
        }

        let vector = vector(current_asteroid, &asteroid);
        let factor = gcd(vector.0, vector.1);
        let min_vector = (vector.0 / factor, vector.1 / factor);
        if factor > 1 {
            let mut blocked = false;
            for multiplier in 0..factor {
                if encountered_asteroids.contains(&(current_asteroid.0 + multiplier * (min_vector.0),
                                                    current_asteroid.1 + multiplier * (min_vector.1))) {
                                                        blocked = true;
                                                        break;
                                                    }                
            }

            if !blocked {
                encountered_asteroids.push(asteroid);
            }
        } else {
            encountered_asteroids.push(asteroid);
        }
    }

    encountered_asteroids.len()
}

fn vector(p1: &(isize, isize), p2: &(isize, isize)) -> (isize, isize) {
    (p2.0 - p1.0, p2.1 - p1.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_visible() {
        let asteroids = vec![(1,0), (4,0), (0,2), (1,2), (2,2), (3,2), (4,2), (4,3), (3,4), (4,4)];
        assert_eq!(7, count_visible(&(1,0), &asteroids));
        assert_eq!(7, count_visible(&(4,0), &asteroids));
        assert_eq!(6, count_visible(&(0,2), &asteroids));
        assert_eq!(7, count_visible(&(1,2), &asteroids));
        assert_eq!(7, count_visible(&(2,2), &asteroids));
        assert_eq!(7, count_visible(&(3,2), &asteroids));
        assert_eq!(5, count_visible(&(4,2), &asteroids));
        assert_eq!(7, count_visible(&(4,3), &asteroids));
        assert_eq!(8, count_visible(&(3,4), &asteroids));
        assert_eq!(7, count_visible(&(4,4), &asteroids));
    }
}
