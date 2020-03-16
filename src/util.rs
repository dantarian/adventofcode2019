use std::io::{BufRead, BufReader, ErrorKind, Read};

pub fn read_comma_separated_integers<R: Read>(io: R) -> Result<Vec<i32>, std::io::Error> {
  let br = BufReader::new(io);
  br.split(b',')
    .map(|r| r.and_then(|v| String::from_utf8(v).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))))
    .map(|r| r.unwrap())
    .map(|s| String::from(s.trim()))
    .filter(|s| s.len() > 0)
    .map(|s| s.parse::<i32>().map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e)))
    .collect()
}

pub fn read_digits<R:Read>(io: R) -> Vec<u8> {
    io.bytes()
        .filter_map(|r| r.ok())
        .filter(|&x| x >= b'0' && x <= b'9')
        .map(|x| x - 48)
        .collect()
}
