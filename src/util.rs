use std::convert::From;
use std::io::{BufRead, BufReader, ErrorKind, Read};
use std::str::FromStr;
use num::Integer;

pub fn read_comma_separated_integers<R, T>(io: R) -> Result<Vec<T>, std::io::Error> 
where
  R: Read,
  T: Integer + FromStr,
{
  let br = BufReader::new(io);
  br.split(b',')
    .map(|r| r.and_then(|v| String::from_utf8(v).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))))
    .map(|r| r.unwrap())
    .map(|s| String::from(s.trim()))
    .filter(|s| s.len() > 0)
    .map(|s| s.parse::<T>().map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "Failed to parse value.")))
    .collect()
}

pub fn read_digits<R:Read>(io: R) -> Vec<u8> {
    io.bytes()
        .filter_map(|r| r.ok())
        .filter(|&x| x >= b'0' && x <= b'9')
        .map(|x| x - 48)
        .collect()
}

pub fn manhattan_distance(p1: (isize, isize), p2: (isize, isize)) -> isize {
  (p1.0 - p2.0).abs() + (p1.1 - p2.1).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manhattan_zero() {
        assert_eq!(0, manhattan_distance((5, 7), (5, 7)));
    }

    #[test]
    fn test_manhattan_one_vertical_positive() {
        assert_eq!(1, manhattan_distance((5,7), (5, 8)));
    }

    #[test]
    fn test_manhattan_one_vertical_negative() {
        assert_eq!(1, manhattan_distance((5,7), (5, 6)));
    }

    #[test]
    fn test_manhattan_one_horizontal_positive() {
        assert_eq!(1, manhattan_distance((5,7), (6, 7)));
    }

    #[test]
    fn test_manhattan_one_horizontal_negative() {
        assert_eq!(1, manhattan_distance((5,7), (4, 7)));
    }

    #[test]
    fn test_manhattan_distance_from_origin() {
        assert_eq!(4, manhattan_distance((0,0), (2,2)));
        assert_eq!(4, manhattan_distance((0,0), (2,-2)));
        assert_eq!(4, manhattan_distance((0,0), (-2,2)));
        assert_eq!(4, manhattan_distance((0,0), (-2,-2)));
    }
}