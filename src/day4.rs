use std::error::Error;

pub fn run(range_start: u32, range_end: u32, _part2: &bool) -> Result<(), Box<dyn Error>> {
    let count = password_count(6, &range_start, &range_end, &0, 0, false);

    println!("Result: {}", count);

    Ok(())
}

fn password_count(digits_remaining: u32, min: &u32, max: &u32, previous_digit: &u32, total: u32, previous_pair: bool) -> u32 {
    if digits_remaining == 0 {
        if previous_pair && total >= *min {
            return 1;  
        } else {
            return 0;
        }
    }

    let mut count = 0;
    for i in *previous_digit..10 {
        let new_total = total + (i * 10u32.pow(digits_remaining - 1));
        if new_total < *min - 10u32.pow(digits_remaining - 1) {
            //println!("New total: {} < {}", new_total, *min - 10u32.pow(digits_remaining - 1));
            continue;
        }
        if new_total > *max {
            //println!("New total: {} > {}", new_total, *max);
            break;
        }

        let pair_found = previous_pair || i == *previous_digit;
        count = count + password_count(digits_remaining - 1, min, max, &i, new_total, pair_found);
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_single_add() {
    }
}
