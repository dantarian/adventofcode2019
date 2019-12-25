use std::io::{BufReader, BufWriter, BufRead, Write};
use std::fmt;

pub struct Computer<'a> {
    memory: Vec<i32>,
    loc: i32,
    running: bool,
    input: BufReader<Box<dyn BufRead + 'a>>,
    output: BufWriter<Box<dyn Write + 'a>>,
}

impl<'a> fmt::Debug for Computer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory: {:?} Location: {:?} Running: {:?}", self.memory, self.loc, self.running)
    }
}

impl<'a> Computer<'a> {
    pub fn new<R: BufRead + 'a, W: Write + 'a>(memory: Vec<i32>, input: R, output: W) -> Self {
        let r_box = Box::new(input);
        let r_br = BufReader::new(r_box as _);
        let w_box = Box::new(output);
        let w_bw = BufWriter::new(w_box as _);
        Computer { memory: memory, loc: 0, running: true, input: r_br, output: w_bw }
    }

    pub fn run(&mut self) -> Result<i32, String> {
        while self.running {
            self.step()?;
        }

        self.result()
    }

    fn step(&mut self) -> Result<(), String> {
        let current_mem_value = self.memory.get(self.loc as usize);
        let (instruction_code, argument_types) = match current_mem_value {
            Some(x) => match Computer::read_instruction_code(*x) {
                Ok((a, b)) => (a, b),
                Err(err) => return Err(err)
            },
            None => return Err(format!("Current location {} is out of range.", self.loc))
        };

        Instruction::new(instruction_code, self.loc, argument_types, &self.memory)
            .and_then(|i| i.call(&mut self.memory, &mut self.input, &mut self.output))
            .and_then(|result| match result {
                CallResult::Step(distance) => {
                    self.loc = self.loc + (distance as i32);
                    Ok(())
                },
                CallResult::Stop => {
                    self.running = false;
                    Ok(())
                }
            })

    }

    fn read_instruction_code(code: i32) -> Result<(u32, Vec<ArgumentKind>), String> {
        if code < 1 {
            return Err(format!("Opcode must be positive, but got {}", code));
        }

        let abs_code = code.abs();
        if abs_code < 100 {
            return Ok((code as u32, vec![]));
        }

        let prefix = (abs_code / 100).to_string();
        if !prefix.chars().all(|x| x == '0' || x == '1') {
            return Err(format!("Unrecognised opcode format: {}", code));
        }
        
        Ok(((code % 100) as u32, (code.abs() / 100).to_string().chars().rfold(vec![], |mut acc, x| match x {
            '0' => { acc.push(ArgumentKind::Position); acc },
            _ => { acc.push(ArgumentKind::Immediate); acc }
        })))
    }

    fn result(&self) -> Result<i32, String> {
        match self.memory.get(0) {
            Some(a) => Ok(a.clone()),
            _ => Err(String::from("Empty memory!"))
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum ArgumentKind {
    Position,
    Immediate
}

#[derive(PartialEq, Eq, Debug)]
struct Argument {
    value: i32,
    kind: ArgumentKind
}

impl Argument {
    fn new(value: i32, kind: Option<&ArgumentKind>) -> Self {
        Argument { value: value, kind: kind.cloned().unwrap_or(ArgumentKind::Position) }
    }

    fn get<'a>(&self, memory: &'a Vec<i32>) -> Option<i32> {
        match self.kind {
            ArgumentKind::Immediate => Some(self.value.clone()),
            ArgumentKind::Position => memory.get(self.value as usize).cloned()
        }
    }

    fn set(&self, memory: &mut Vec<i32>, new_value: i32) -> Result<(), String> {
        match self.kind {
            ArgumentKind::Immediate => Err(String::from("Can't populate Immediate argument.")),
            ArgumentKind::Position => {
                match memory.get_mut(self.value as usize) {
                    Some(element) => { *element = new_value; Ok(()) },
                    None => Err(format!("Memory index out of bounds: {}", self.value))
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Instruction {
    Add(Argument, Argument, Argument),
    Multiply(Argument, Argument, Argument),
    Input(Argument),
    Output(Argument),
    Stop
}

#[derive(PartialEq, Eq, Debug)]
enum CallResult {
    Step(u32),
    Stop
}

impl Instruction {
    fn new(code: u32, base_location: i32, argument_types: Vec<ArgumentKind>, memory: &Vec<i32>) -> Result<Self, String> {
        let address = |x| *(memory.get(x as usize).unwrap());
        match code {
            1 => {
                if base_location < 0 || (base_location as usize) + 3 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::Add(Argument::new(address(base_location + 1), argument_types.get(0)),
                                    Argument::new(address(base_location + 2), argument_types.get(1)),
                                    Argument::new(address(base_location + 3), argument_types.get(2))))
            },
            2 => {
                if base_location < 0 || (base_location as usize) + 3 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::Multiply(Argument::new(address(base_location + 1), argument_types.get(0)),
                                         Argument::new(address(base_location + 2), argument_types.get(1)),
                                         Argument::new(address(base_location + 3), argument_types.get(2))))
            },
            3 => {
                if base_location < 0 || (base_location as usize) + 1 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::Input(Argument::new(address(base_location + 1), argument_types.get(0))))
            },
            4 => {
                if base_location < 0 || (base_location as usize) + 1 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::Output(Argument::new(address(base_location + 1), argument_types.get(0))))
            },
            99 => Ok(Instruction::Stop),
            x => Err(format!("Unsupported instruction: {}", x))
        }
    }

    fn length(&self) -> u32 {
        match self {
            Instruction::Add(_,_,_) => 4,
            Instruction::Multiply(_,_,_) => 4,
            Instruction::Input(_) => 2,
            Instruction::Output(_) => 2,
            Instruction::Stop => 0
        }
    }

    fn call<'a>(&self, memory: &mut Vec<i32>, reader: &mut BufReader<Box<dyn BufRead + 'a>>, writer: &mut BufWriter<Box<dyn Write + 'a>>) -> Result<CallResult, String> {
        match self {
            Instruction::Add(input1, input2, output) => 
                match (input1.get(memory), input2.get(memory)) {
                    (Some(a), Some(b)) => {
                        output.set(memory, a+b).and(Ok(CallResult::Step(self.length())))
                    },
                    _ => Err(String::from("Failed to find a referenced value."))
                },
            Instruction::Multiply(input1, input2, output) => 
                match (input1.get(memory), input2.get(memory)) {
                    (Some(a), Some(b)) => {
                        output.set(memory, a*b).and(Ok(CallResult::Step(self.length())))
                    },
                    _ => Err(String::from("Failed to find a referenced value."))
                },
            Instruction::Input(output) => {
                println!("Please provide an input:");
                let mut buffer = String::new();
                reader.read_line(&mut buffer).map_err(|err| err.to_string())?;
                let value: i32 = buffer.trim().parse().map_err(|err: std::num::ParseIntError| err.to_string())?;
                output.set(memory, value)?;
                Ok(CallResult::Step(self.length()))
            },
            Instruction::Output(input) => {
                match input.get(memory) {
                    Some(a) => {
                        writeln!(writer, "{}", a).map_err(|err| err.to_string())?;
                        Ok(CallResult::Step(self.length()))
                    },
                    _ => Err(String::from("Failed to find a referenced value."))
                }
            },
            Instruction::Stop => Ok(CallResult::Stop),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_instruction_add() {
        let instruction = Instruction::new(1, 0, vec![ArgumentKind::Position, ArgumentKind::Immediate], &vec![1, 2, 3, 4]).unwrap();
        assert_eq!(Instruction::Add(Argument { value: 2, kind: ArgumentKind::Position },
                                    Argument { value: 3, kind: ArgumentKind::Immediate },
                                    Argument { value: 4, kind: ArgumentKind::Position }),
                   instruction);
    }

    #[test]
    fn test_new_instruction_mutiply() {
        let instruction = Instruction::new(2, 1, vec![ArgumentKind::Position, ArgumentKind::Immediate], &vec![3, 4, 5, 6, 7]).unwrap();
        assert_eq!(Instruction::Multiply(Argument { value: 5, kind: ArgumentKind::Position },
                                         Argument { value: 6, kind: ArgumentKind::Immediate },
                                         Argument { value: 7, kind: ArgumentKind::Position }),
                   instruction);
    }

    #[test]
    fn test_new_instruction_stop() {
        let instruction = Instruction::new(99, 0, vec![], &vec![]).unwrap();
        assert_eq!(Instruction::Stop,
                   instruction);
    }

    #[test]
    fn test_positional_argument_get() {
        let argument = Argument::new(3, Some(&ArgumentKind::Position));
        let result = argument.get(&vec![11, 12, 13, 14]).unwrap();
        assert_eq!(14, result);
    }

    #[test]
    fn test_positional_argument_set() {
        let argument = Argument::new(3, Some(&ArgumentKind::Position));
        let mut memory = vec![11, 12, 13, 14];
        argument.set(&mut memory, 42).unwrap();
        assert_eq!(vec![11, 12, 13, 42], memory);
    }

    #[test]
    fn test_immediate_argument_get() {
        let argument = Argument::new(3, Some(&ArgumentKind::Immediate));
        let result = argument.get(&vec![11, 12, 13, 14]).unwrap();
        assert_eq!(3, result);
    }

    #[test]
    fn test_read_instruction_code_1() {
        let (instruction_code, argument_kinds) = Computer::read_instruction_code(1).unwrap();
        assert_eq!(1, instruction_code);
        assert_eq!(vec![] as Vec<ArgumentKind>, argument_kinds);
    }

    #[test]
    fn test_read_instruction_code_101() {
        let (instruction_code, argument_kinds) = Computer::read_instruction_code(101).unwrap();
        assert_eq!(1, instruction_code);
        assert_eq!(vec![ArgumentKind::Immediate], argument_kinds);
    }

    #[test]
    fn test_read_instruction_code_1001() {
        let (instruction_code, argument_kinds) = Computer::read_instruction_code(1001).unwrap();
        assert_eq!(1, instruction_code);
        assert_eq!(vec![ArgumentKind::Position, ArgumentKind::Immediate], argument_kinds);
    }

    #[test]
    fn test_step_single_add() {
        let input = std::io::stdin();
        let output = std::io::stdout();
        let mut computer = Computer::new(vec![1, 0, 0, 0, 99], &input, &output);
        computer.step().is_ok();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![2, 0, 0, 0, 99], computer.memory);
    }

    #[test]
    fn test_step_single_mutiply() {
        let input = std::io::stdin();
        let output = std::io::stdout();
        let mut computer = Computer::new(vec![2, 3, 0, 3, 99], &input, &output);
        computer.step().is_ok();
        assert_eq!(vec![2, 3, 0, 6, 99], computer.memory);
    }

    #[test]
    fn test_step_single_mutiply_long() {
        let input = std::io::stdin();
        let output = std::io::stdout();
        let mut computer = Computer::new(vec![2, 4, 4, 5, 99, 0], &input, &output);
        computer.step().is_ok();
        assert_eq!(vec![2, 4, 4, 5, 99, 9801], computer.memory);
    }

    #[test]
    fn test_run() {
        let input = std::io::stdin();
        let output = std::io::stdout();
        let mut computer = Computer::new(vec![1, 1, 1, 4, 99, 5, 6, 0, 99], &input, &output);
        assert_eq!(30, computer.run().unwrap());
    }
}
