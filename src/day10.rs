use std::error::Error;
use std::path::PathBuf;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::collections::{BTreeMap, VecDeque};
use std::convert::TryFrom;
use std::mem;
use num::integer::gcd;

use crate::util::manhattan_distance;

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let lines = read(File::open(filename)?)?;
    let asteroids = find_asteroids(lines)?;

    if *part2 {
        // Put the asteroids into buckets in a BTreeMap<f64, VecDeque<(isize, isize)>>, where the key
        // is the angle from the vertical (applying appropriate transformations because our plane is
        // flipped on the x-axis from the Cartesian). Then repeatedly iterate through the map, popping
        // elements as we go, until we find the 200th element.

        let laser_station = asteroids.iter().max_by_key(|x| count_visible(x, &asteroids)).unwrap();
        let mut other_asteroids: Vec<(isize, isize)> = asteroids.clone()
                                                                 .iter()
                                                                 .filter(|&x| *x != *laser_station)
                                                                 .cloned()
                                                                 .collect();
        other_asteroids.sort_by_cached_key(|&a| manhattan_distance(*laser_station, a));

        let mut mapped_asteroids = BTreeMap::new();
        for asteroid in other_asteroids {
            let key = Angle::new(angle(*laser_station, asteroid));
            if !mapped_asteroids.contains_key(&key) {
                mapped_asteroids.insert(key.clone(), VecDeque::new());
            }
            if let Some(vec) = mapped_asteroids.get_mut(&key) {
                (*vec).push_back(asteroid)
            }
        }

        let mut destroyed_asteroids = vec![];
        while destroyed_asteroids.len() < 200 {
            for (_, targeted_asteroids) in mapped_asteroids.iter_mut() {
                if let Some(asteroid) = targeted_asteroids.pop_front() {
                    destroyed_asteroids.push(asteroid);
                }
            }
        }

        let two_hundredth = destroyed_asteroids[199];
        println!("{}", two_hundredth.0 * 100 + two_hundredth.1);
        
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

fn integer_decode(val: f64) -> (i16, u64) {
    let bits: u64 = unsafe { mem::transmute(val) };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };

    exponent -= 1023 + 52;
    (exponent, mantissa)
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
struct Angle((i16, u64));

impl Angle {
    fn new(val: f64) -> Angle {
        Angle(integer_decode(val))
    }
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

/// Find the angle of the vector from the negative y-axis, with +pi/2 being at the
/// positive x-axis.
fn angle(origin: (isize, isize), target: (isize, isize)) -> f64 {
    let v = vector(&origin, &target);
    // Deal with the degenerate cases first.
    if v.1 == 0 {
        if v.0 > 0 {
            return PI/2f64;
        } else {
            return 3f64 * PI/2f64;
        }
    }

    if v.1 < 0 {
        if v.0 < 0 {
            return (2f64 * PI) + (-v.0 as f64 / v.1 as f64).atan();
        } else {
            return (-v.0 as f64/v.1 as f64).atan();
        }
    } else {
        return PI + (-v.0 as f64/v.1 as f64).atan();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

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

    #[test]
    fn test_angle_up() {
        assert_approx_eq!(0f64, angle((3,3), (3,2)), 1e-3f64);
    }

    #[test]
    fn test_angle_right() {
        assert_approx_eq!(PI/2f64, angle((3,3), (4,3)), 1e-3f64);
    }

    #[test]
    fn test_angle_down() {
        assert_approx_eq!(PI, angle((3,3), (3,4)), 1e-3f64);
    }

    #[test]
    fn test_angle_left() {
        assert_approx_eq!(3f64 * PI/2f64, angle((3,3), (2,3)), 1e-3f64);
    }

    #[test]
    fn test_angle_1_1_is_3_pi_by_4() {
        assert_approx_eq!(3f64 * PI/4f64, angle((0,0), (1,1)), 1e-3f64);
    }

    #[test]
    fn test_angle_1_minus_1_is_pi_by_4() {
        assert_approx_eq!(PI/4f64, angle((0,0), (1,-1)), 1e-3f64);
    }

    #[test]
    fn test_angle_minus_1_1_is_5_pi_by_4() {
        assert_approx_eq!(5f64 * PI/4f64, angle((0,0), (-1,1)), 1e-3f64);
    }

    #[test]
    fn test_angle_minus_1_minus_1_is_7_pi_by_4() {
        assert_approx_eq!(7f64 * PI/4f64, angle((0,0), (-1,-1)), 1e-3f64);
    }
}
