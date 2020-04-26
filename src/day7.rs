use std::collections::VecDeque;
use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::{process, thread, time};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

use itertools::Itertools;

use crate::util;
use crate::intcode::{Computer, ComputerInput, ComputerOutput};

pub fn run(filename: &PathBuf, part2: &bool) -> Result<(), Box<dyn Error>> {
    let initial_state = util::read_comma_separated_integers(File::open(filename)?)?;

    if *part2 {
        // No action yet!
        let possibilities = (5..10).permutations(5);
        let mut max_result = 0;
        let mut count = 0;

        for permutation in possibilities {
            count += 1;
            let (tx_a, tx_a2, rx_b) = make_channel_with_spare_transmitter();
            let (tx_b, tx_b2, rx_c) = make_channel_with_spare_transmitter();
            let (tx_c, tx_c2, rx_d) = make_channel_with_spare_transmitter();
            let (tx_d, tx_d2, rx_e) = make_channel_with_spare_transmitter();
            let (tx_e, tx_e2, rx_a) = make_channel_with_spare_transmitter();

            // We use the transmitter for the previous amp in the chain in each case.
            send_phase_and_initial_value(tx_e2, permutation[0]);
            send_phase(tx_a2, permutation[1]);
            send_phase(tx_b2, permutation[2]);
            send_phase(tx_c2, permutation[3]);
            send_phase(tx_d2, permutation[4]);

            // Give the messages a chance to send.
            thread::sleep(time::Duration::from_millis(10));

            launch_computer(initial_state.clone(), rx_a, tx_a);
            launch_computer(initial_state.clone(), rx_b, tx_b);
            launch_computer(initial_state.clone(), rx_c, tx_c);
            launch_computer(initial_state.clone(), rx_d, tx_d);
            let mut amp_e = make_computer(initial_state.clone(), rx_e, tx_e);

            match amp_e.run() {
                Ok(_) => {
                    match amp_e.output().pop_front() {
                        Some(element) => {
                            if element > max_result {
                                max_result = element;
                            }
                        },
                        None => {
                            eprintln!("No output found from computer for case {}: {:?}", count, permutation);
                            process::exit(1);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Problem running computer: {}", e);
                    process::exit(1);
                }
            }
        }

        println!("{}", max_result);

        Ok(())
    } else {
        let possibilities = (0..5).permutations(5);
        let mut max_result = 0;

        for permutation in possibilities {
            let mut result = 0;
            for phase in permutation {
                let input = VecDeque::from(vec![phase, result]);
                let mut computer = Computer::new(initial_state.clone(), Some(ComputerInput::Queue(input)), None);
                match computer.run() {
                    Ok(_) => {
                        match computer.output().pop_front() {
                            Some(element) => { result = element; },
                            None => {
                                eprintln!("No output found from computer!");
                                process::exit(1);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Problem running computer: {}", e);
                        process::exit(1);
                    }
                }
            }

            if result > max_result {
                max_result = result;
            }
        }

        println!("{}", max_result);

        Ok(())
    }
}

fn make_computer(initial_state: Vec<i32>, input: Receiver<i32>, output: SyncSender<i32>) -> Computer<i32> {
    Computer::new(initial_state, Some(ComputerInput::Channel(input)), Some(ComputerOutput::Channel(output)))
}

fn launch_computer(initial_state: Vec<i32>, input: Receiver<i32>, output: SyncSender<i32>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut c = make_computer(initial_state, input, output);
        c.run().unwrap_or_else(|e| {
            eprintln!("Problem running computer: {}", e);
            process::exit(1);
        });
    })
}

fn make_channel_with_spare_transmitter() -> (SyncSender<i32>, SyncSender<i32>, Receiver<i32>) {
    let (tx, rx) = sync_channel(0);
    let tx2 = tx.clone();
    (tx, tx2, rx)
}

fn send_phase(sender: SyncSender<i32>, phase: i32) {
    thread::spawn(move || {
        sender.send(phase).unwrap();
    });
}

fn send_phase_and_initial_value(sender: SyncSender<i32>, phase: i32) {
    thread::spawn(move || {
        sender.send(phase).unwrap();
        sender.send(0).unwrap();
    });
}
