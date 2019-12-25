use std::error::Error;

pub fn run(range_start: u32, range_end: u32, part2: &bool) -> Result<(), Box<dyn Error>> {
    let count = if *part2 {
        password_count2(6, &range_start, &range_end, &0, 0, 0, 0)
    } else {
        password_count(6, &range_start, &range_end, &0, 0, false)
    };

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
            continue;
        }
        if new_total > *max {
            break;
        }

        let pair_found = previous_pair || i == *previous_digit;
        count = count + password_count(digits_remaining - 1, min, max, &i, new_total, pair_found);
    }

    count
}

fn password_count2(digits_remaining: u32, min: &u32, max: &u32, previous_digit: &u32, total: u32, pair: u32, overload: u32) -> u32 {
    if digits_remaining == 0 {
        if pair > 0 && total >= *min {
            return 1;  
        } else {
            return 0;
        }
    }

    let mut count = 0;
    for i in *previous_digit..10 {
        let new_total = total + (i * 10u32.pow(digits_remaining - 1));
        if new_total < *min - 10u32.pow(digits_remaining - 1) {
            continue;
        }
        if new_total > *max {
            break;
        }

        let new_overload = if i == overload || i == pair { i } else { overload };
        let new_pair = if i == new_overload {
            0
        } else if pair == 0 && i == *previous_digit {
            i
        } else {
            pair
        };
        count = count + password_count2(digits_remaining - 1, min, max, &i, new_total, new_pair, new_overload);
    }

    count
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn test_step_single_add() {
    }
}
