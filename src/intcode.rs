use std::collections::VecDeque;
use std::fmt;
use std::sync::mpsc::{SyncSender, Receiver};

pub enum ComputerOutput {
    Queue(VecDeque<i32>),
    Channel(SyncSender<i32>)
}

pub enum ComputerInput {
    Queue(VecDeque<i32>),
    Channel(Receiver<i32>)
}

pub struct Computer {
    memory: Vec<i32>,
    loc: usize,
    running: bool,
    input: ComputerInput,
    output: ComputerOutput,
    alt_output: VecDeque<i32>,
    relative_base: i32,
}

impl fmt::Debug for Computer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory: {:?} Location: {:?} Running: {:?}", self.memory, self.loc, self.running)
    }
}

impl Computer {
    /// Creates a new Computer.
    ///
    /// The function takes the intial memory state for the computer, plus an optional input and
    /// output. An empty queue is used as the default for input and output, if no alternative is
    /// supplied.
    pub fn new(memory: Vec<i32>, input: Option<ComputerInput>, output: Option<ComputerOutput>) -> Self {
        Computer { 
            memory: memory, 
            loc: 0, 
            running: true, 
            input: input.unwrap_or(ComputerInput::Queue(VecDeque::new())),
            output: output.unwrap_or(ComputerOutput::Queue(VecDeque::new())),
            alt_output: VecDeque::new(),
            relative_base: 0
        }
    }

    pub fn run(&mut self) -> Result<i32, String> {
        while self.running {
            self.step()?;
        }

        self.result()
    }

    fn step(&mut self) -> Result<(), String> {
        let current_mem_value = self.memory.get(self.loc);
        let (instruction_code, argument_types) = match current_mem_value {
            Some(x) => match Computer::read_instruction_code(*x) {
                Ok((a, b)) => (a, b),
                Err(err) => return Err(err)
            },
            None => return Err(format!("Current location {} is out of range.", self.loc))
        };

        Instruction::new(instruction_code, self.loc, argument_types, &self.memory, self.relative_base)
            .and_then(|i| i.call(&mut self.memory, &mut self.input, &mut self.output, &mut self.alt_output, &mut self.relative_base))
            .and_then(|result| match result {
                CallResult::Step(distance) => {
                    self.loc = self.loc + distance;
                    Ok(())
                },
                CallResult::Jump(target) => {
                    self.loc = target;
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
        if !prefix.chars().all(|x| x == '0' || x == '1' || x == '2') {
            return Err(format!("Unrecognised opcode format: {}", code));
        }
        
        Ok(((code % 100) as u32, (code.abs() / 100).to_string().chars().rfold(vec![], |mut acc, x| match x {
            '0' => { acc.push(ArgumentKind::Position); acc },
            '1' => { acc.push(ArgumentKind::Immediate); acc },
            _ => { acc.push(ArgumentKind::Relative); acc }
        })))
    }

    fn result(&self) -> Result<i32, String> {
        match self.memory.get(0) {
            Some(a) => Ok(a.clone()),
            _ => Err(String::from("Empty memory!"))
        }
    }

    pub fn output(&self) -> VecDeque<i32> {
        match &self.output {
            ComputerOutput::Queue(q) => q.clone(),
            ComputerOutput::Channel(_) => self.alt_output.clone()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum ArgumentKind {
    Position,
    Immediate,
    Relative
}

#[derive(PartialEq, Eq, Debug)]
struct Argument {
    value: i32,
    kind: ArgumentKind,
    relative_base: i32
}

impl Argument {
    fn new(value: i32, kind: Option<&ArgumentKind>, relative_base: i32) -> Self {
        Argument { value: value, kind: kind.cloned().unwrap_or(ArgumentKind::Position), relative_base: relative_base }
    }

    fn get<'a>(&self, memory: &'a Vec<i32>) -> Option<i32> {
        match self.kind {
            ArgumentKind::Immediate => Some(self.value.clone()),
            ArgumentKind::Position => memory.get(self.value as usize).cloned(),
            ArgumentKind::Relative => memory.get((self.value + self.relative_base) as usize).cloned()
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
            },
            ArgumentKind::Relative => {
                match memory.get_mut((self.value + self.relative_base) as usize) {
                    Some(element) => { *element = new_value; Ok(()) },
                    None => Err(format!("Memory index out of bounds: {}", self.value + self.relative_base))
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
    JumpIfTrue(Argument, Argument),
    JumpIfFalse(Argument, Argument),
    LessThan(Argument, Argument, Argument),
    Equals(Argument, Argument, Argument),
    AdjustRelativeBase(Argument),
    Stop
}

#[derive(PartialEq, Eq, Debug)]
enum CallResult {
    Step(usize),
    Jump(usize),
    Stop
}

impl Instruction {
    fn new(code: u32, base_location: usize, argument_types: Vec<ArgumentKind>, memory: &Vec<i32>, relative_base: i32) -> Result<Self, String> {
        let address = |x| *(memory.get(x as usize).unwrap());
        match code {
            1 => {
                if base_location + 3 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::Add(Argument::new(address(base_location + 1), argument_types.get(0), relative_base),
                                    Argument::new(address(base_location + 2), argument_types.get(1), relative_base),
                                    Argument::new(address(base_location + 3), argument_types.get(2), relative_base)))
            },
            2 => {
                if base_location + 3 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::Multiply(Argument::new(address(base_location + 1), argument_types.get(0), relative_base),
                                         Argument::new(address(base_location + 2), argument_types.get(1), relative_base),
                                         Argument::new(address(base_location + 3), argument_types.get(2), relative_base)))
            },
            3 => {
                if base_location + 1 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::Input(Argument::new(address(base_location + 1), argument_types.get(0), relative_base)))
            },
            4 => {
                if base_location + 1 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::Output(Argument::new(address(base_location + 1), argument_types.get(0), relative_base)))
            },
            5 => {
                if base_location + 2 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::JumpIfTrue(Argument::new(address(base_location + 1), argument_types.get(0), relative_base),
                                           Argument::new(address(base_location + 2), argument_types.get(1), relative_base)))
            },
            6 => {
                if base_location + 2 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::JumpIfFalse(Argument::new(address(base_location + 1), argument_types.get(0), relative_base),
                                            Argument::new(address(base_location + 2), argument_types.get(1), relative_base)))
            },
            7 => {
                if base_location + 3 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::LessThan(Argument::new(address(base_location + 1), argument_types.get(0), relative_base),
                                         Argument::new(address(base_location + 2), argument_types.get(1), relative_base),
                                         Argument::new(address(base_location + 3), argument_types.get(2), relative_base)))
            },
            8 => {
                if base_location + 3 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::Equals(Argument::new(address(base_location + 1), argument_types.get(0), relative_base),
                                       Argument::new(address(base_location + 2), argument_types.get(1), relative_base),
                                       Argument::new(address(base_location + 3), argument_types.get(2), relative_base)))
            },
            9 => {
                if base_location + 1 > memory.len() {
                    return Err(String::from("Tried to read off end of memory."));
                }
                Ok(Instruction::AdjustRelativeBase(Argument::new(address(base_location + 1), argument_types.get(0), relative_base)))
            },
            99 => Ok(Instruction::Stop),
            x => Err(format!("Unsupported instruction: {}", x))
        }
    }

    fn length(&self) -> usize {
        match self {
            Instruction::Add(_,_,_) => 4,
            Instruction::Multiply(_,_,_) => 4,
            Instruction::Input(_) => 2,
            Instruction::Output(_) => 2,
            Instruction::JumpIfTrue(_,_) => 3,
            Instruction::JumpIfFalse(_,_) => 3,
            Instruction::LessThan(_,_,_) => 4,
            Instruction::Equals(_,_,_) => 4,
            Instruction::AdjustRelativeBase(_) => 2,
            Instruction::Stop => 0
        }
    }

    fn call<'a>(&self, 
                memory: &mut Vec<i32>, 
                reader: &mut ComputerInput, 
                writer: &mut ComputerOutput, 
                alt_output: &mut VecDeque<i32>,
                relative_base: &mut i32) -> Result<CallResult, String> {
        match self {
            Instruction::Add(input1, input2, output) => self.add(input1, input2, output, memory),
            Instruction::Multiply(input1, input2, output) => self.multiply(input1, input2, output, memory),
            Instruction::Input(destination) => self.input(destination, memory, reader),
            Instruction::Output(source) => self.output(source, memory, writer, alt_output),
            Instruction::JumpIfTrue(input, target) => self.jump_if_true(input, target, memory),
            Instruction::JumpIfFalse(input, target) => self.jump_if_false(input, target, memory),
            Instruction::LessThan(input1, input2, output) => self.less_than(input1, input2, output, memory),
            Instruction::Equals(input1, input2, output) => self.equals(input1, input2, output, memory),
            Instruction::AdjustRelativeBase(input) => self.adjust_relative_base(input, memory, relative_base),
            Instruction::Stop => Ok(CallResult::Stop),
        }
    }

    fn add(&self, input1: &Argument, input2: &Argument, output: &Argument, memory: &mut Vec<i32>) -> Result<CallResult, String> {
        match (input1.get(memory), input2.get(memory)) {
            (Some(a), Some(b)) => output.set(memory, a+b).and(Ok(CallResult::Step(self.length()))),
            _ => Err(String::from("Failed to find a referenced value."))
        }    
    }

    fn multiply(&self, input1: &Argument, input2: &Argument, output: &Argument, memory: &mut Vec<i32>) -> Result<CallResult, String> {
        match (input1.get(memory), input2.get(memory)) {
            (Some(a), Some(b)) => output.set(memory, a*b).and(Ok(CallResult::Step(self.length()))),
            _ => Err(String::from("Failed to find a referenced value."))
        }
    }

    fn input<'a>(&self, destination: &Argument, memory: &mut Vec<i32>, input: &mut ComputerInput) -> Result<CallResult, String> {
        match input {
            ComputerInput::Queue(q) => match q.pop_front() {
                Some(value) => {
                    destination.set(memory, value)?;
                    Ok(CallResult::Step(self.length()))
                },
                None => Err(String::from("Failed to find an input value."))
            },
            ComputerInput::Channel(rx) => match rx.recv() {
                Ok(val) => {
                    destination.set(memory, val)?;
                    Ok(CallResult::Step(self.length()))
                },
                Err(_) => Err(String::from("Failed to receive an input value."))
            }
        }
    }

    fn output<'a>(&self, source: &Argument, memory: &mut Vec<i32>, output: &mut ComputerOutput, alt_output: &mut VecDeque<i32>) -> Result<CallResult, String> {
        match source.get(memory) {
            Some(value) => {
                match output {
                    ComputerOutput::Queue(q) => q.push_back(value),
                    ComputerOutput::Channel(tx) => match tx.send(value) {
                        Err(_) => {
                            alt_output.push_back(value);
                        },
                        _ => ()
                    }
                };
                Ok(CallResult::Step(self.length()))
            },
            None => Err(String::from("Failed to find a referenced value."))
        }
    }

    fn jump_if_true(&self, input: &Argument, target: &Argument, memory: &mut Vec<i32>) -> Result<CallResult, String> {
        match input.get(memory) {
            Some(a) if a != 0 => match target.get(memory) {
                Some(b) => Ok(CallResult::Jump(b as usize)),
                None => Err(String::from("Failed to find a referenced value."))
            },
            Some(_a) => Ok(CallResult::Step(self.length())),
            None => Err(String::from("Failed to find a referenced value."))
        }
    }

    fn jump_if_false(&self, input: &Argument, target: &Argument, memory: &mut Vec<i32>) -> Result<CallResult, String> {
        match input.get(memory) {
            Some(a) if a == 0 => match target.get(memory) {
                Some(b) => Ok(CallResult::Jump(b as usize)),
                None => Err(String::from("Failed to find a referenced value."))
            },
            Some(_a) => Ok(CallResult::Step(self.length())),
            None => Err(String::from("Failed to find a referenced value."))
        }
    }
    
    fn less_than(&self, input1: &Argument, input2: &Argument, output: &Argument, memory: &mut Vec<i32>) -> Result<CallResult, String> {
        match (input1.get(memory), input2.get(memory)) {
            (Some(a), Some(b)) => output.set(memory, if a < b { 1 } else { 0 }).and(Ok(CallResult::Step(self.length()))),
            _ => Err(String::from("Failed to find a referenced value."))
        }
    }

    fn equals(&self, input1: &Argument, input2: &Argument, output: &Argument, memory: &mut Vec<i32>) -> Result<CallResult, String> {
        match (input1.get(memory), input2.get(memory)) {
            (Some(a), Some(b)) => output.set(memory, if a == b { 1 } else { 0 }).and(Ok(CallResult::Step(self.length()))),
            _ => Err(String::from("Failed to find a referenced value."))
        }
    }

    fn adjust_relative_base(&self, input: &Argument, memory: &mut Vec<i32>, relative_base: &mut i32) -> Result<CallResult, String> {
        match input.get(memory) {
            Some(a) => {
                *relative_base = *relative_base + a;
                Ok(CallResult::Step(self.length()))
            },
            None => Err(String::from("Failed to find a referenced value."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::sync_channel;
    use std::thread;

    #[test]
    fn test_new_instruction_add() {
        let instruction = Instruction::new(1, 0, vec![ArgumentKind::Position, ArgumentKind::Immediate], &vec![1, 2, 3, 4], 0).unwrap();
        assert_eq!(Instruction::Add(Argument { value: 2, kind: ArgumentKind::Position, relative_base: 0},
                                    Argument { value: 3, kind: ArgumentKind::Immediate, relative_base: 0},
                                    Argument { value: 4, kind: ArgumentKind::Position, relative_base: 0 }),
                   instruction);
    }

    #[test]
    fn test_new_instruction_mutiply() {
        let instruction = Instruction::new(2, 1, vec![ArgumentKind::Position, ArgumentKind::Immediate], &vec![3, 4, 5, 6, 7], 0).unwrap();
        assert_eq!(Instruction::Multiply(Argument { value: 5, kind: ArgumentKind::Position, relative_base: 0 },
                                         Argument { value: 6, kind: ArgumentKind::Immediate, relative_base: 0 },
                                         Argument { value: 7, kind: ArgumentKind::Position, relative_base: 0 }),
                   instruction);
    }

    #[test]
    fn test_new_instruction_stop() {
        let instruction = Instruction::new(99, 0, vec![], &vec![], 0).unwrap();
        assert_eq!(Instruction::Stop,
                   instruction);
    }

    #[test]
    fn test_positional_argument_get() {
        let argument = Argument::new(3, Some(&ArgumentKind::Position), 0);
        let result = argument.get(&vec![11, 12, 13, 14]).unwrap();
        assert_eq!(14, result);
    }

    #[test]
    fn test_positional_argument_set() {
        let argument = Argument::new(3, Some(&ArgumentKind::Position), 0);
        let mut memory = vec![11, 12, 13, 14];
        argument.set(&mut memory, 42).unwrap();
        assert_eq!(vec![11, 12, 13, 42], memory);
    }

    #[test]
    fn test_immediate_argument_get() {
        let argument = Argument::new(3, Some(&ArgumentKind::Immediate), 0);
        let result = argument.get(&vec![11, 12, 13, 14]).unwrap();
        assert_eq!(3, result);
    }

    #[test]
    fn test_relative_argument_get() {
        let argument = Argument::new(1, Some(&ArgumentKind::Relative), 1);
        let result = argument.get(&vec![1, 2, 3, 4]).unwrap();
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
    fn test_read_instruction_code_2001() {
        let (instruction_code, argument_kinds) = Computer::read_instruction_code(2001).unwrap();
        assert_eq!(1, instruction_code);
        assert_eq!(vec![ArgumentKind::Position, ArgumentKind::Relative], argument_kinds);
    }

    #[test]
    fn test_step_single_add() {
        let mut computer = Computer::new(vec![1, 0, 0, 0, 99], None, None);
        computer.step().is_ok();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![2, 0, 0, 0, 99], computer.memory);
    }

    #[test]
    fn test_step_single_mutiply() {
        let mut computer = Computer::new(vec![2, 3, 0, 3, 99], None, None);
        computer.step().is_ok();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![2, 3, 0, 6, 99], computer.memory);
    }

    #[test]
    fn test_step_single_mutiply_long() {
        let mut computer = Computer::new(vec![2, 4, 4, 5, 99, 0], None, None);
        computer.step().is_ok();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![2, 4, 4, 5, 99, 9801], computer.memory);
    }
    
    #[test]
    fn test_step_input() {
        let mut input = VecDeque::new();
        input.push_back(42);
        let mut computer = Computer::new(vec![3, 2, 0], Some(ComputerInput::Queue(input)), None);
        computer.step().is_ok();
        assert_eq!(2, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![3, 2, 42], computer.memory);
    }

    #[test]
    fn test_step_input_with_channel() {
        let (tx, rx) = sync_channel(0);

        thread::spawn(move || {
            tx.send(42).unwrap();
        });

        let mut computer = Computer::new(vec![3, 2, 0], Some(ComputerInput::Channel(rx)), None);
        computer.step().is_ok();
        assert_eq!(2, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![3, 2, 42], computer.memory);
    }

    #[test]
    fn test_step_output() {
        let mut computer = Computer::new(vec![4, 2, 42], None, None);
        computer.step().is_ok();
        assert_eq!(2, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![4, 2, 42], computer.memory);
        assert_eq!(vec![42], Vec::from(match computer.output {
            ComputerOutput::Queue(q) => q,
            _ => VecDeque::new()
        }));
    }

    #[test]
    fn test_step_output_with_channel() {
        let (tx, rx) = sync_channel(0);

        thread::spawn(move || {
            let mut computer = Computer::new(vec![4, 2, 42], None, Some(ComputerOutput::Channel(tx)));
            computer.step().is_ok();
            assert_eq!(2, computer.loc);
            assert_eq!(vec![4, 2, 42], computer.memory);
        });

        assert_eq!(42, rx.recv().unwrap());
    }

    #[test]
    fn test_step_output_with_channel_no_receiver() {
        let (tx, rx) = sync_channel(0);
        drop(rx);
        let mut computer = Computer::new(vec![4, 2, 43], None, Some(ComputerOutput::Channel(tx)));
        computer.step().is_ok();
        assert_eq!(2, computer.loc);
        assert_eq!(vec![4, 2, 43], computer.memory);
        assert_eq!(43, computer.output().pop_front().unwrap());
    }

    #[test]
    fn test_step_jump_if_true_true() {
        let mut computer = Computer::new(vec![1105, 1, 20], None, None);
        computer.step().is_ok();
        assert_eq!(20, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![1105, 1, 20], computer.memory);
    }

    #[test]
    fn test_step_jump_if_true_true_positional() {
        let mut computer = Computer::new(vec![5, 0, 2], None, None);
        computer.step().is_ok();
        assert_eq!(2, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![5, 0, 2], computer.memory);
    }

    #[test]
    fn test_step_jump_if_true_false() {
        let mut computer = Computer::new(vec![1105, 0, 20], None, None);
        computer.step().is_ok();
        assert_eq!(3, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![1105, 0, 20], computer.memory);
    }

    #[test]
    fn test_step_jump_if_true_false_positional() {
        let mut computer = Computer::new(vec![5, 3, 20, 0], None, None);
        computer.step().is_ok();
        assert_eq!(3, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![5, 3, 20, 0], computer.memory);
    }

    #[test]
    fn test_step_jump_if_false_false() {
        let mut computer = Computer::new(vec![1106, 0, 20], None, None);
        computer.step().is_ok();
        assert_eq!(20, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![1106, 0, 20], computer.memory);
    }

    #[test]
    fn test_step_jump_if_false_false_positional() {
        let mut computer = Computer::new(vec![6, 3, 3, 0], None, None);
        computer.step().is_ok();
        assert_eq!(0, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![6, 3, 3, 0], computer.memory);
    }

    #[test]
    fn test_step_jump_if_false_true() {
        let mut computer = Computer::new(vec![1106, 1, 20], None, None);
        computer.step().is_ok();
        assert_eq!(3, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![1106, 1, 20], computer.memory);
    }

    #[test]
    fn test_step_less_than_true() {
        let mut computer = Computer::new(vec![1107, -1, 0, 0], None, None);
        computer.step().is_ok();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![1, -1, 0, 0], computer.memory);
    }

    #[test]
    fn test_step_less_than_false() {
        let mut computer = Computer::new(vec![1107, 10, 10, 0], None, None);
        computer.step().is_ok();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![0, 10, 10, 0], computer.memory);
    }

    #[test]
    fn test_step_equals_true() {
        let mut computer = Computer::new(vec![1108, 42, 42, 0], None, None);
        computer.step().is_ok();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![1, 42, 42, 0], computer.memory);
    }

    #[test]
    fn test_step_equals_false() {
        let mut computer = Computer::new(vec![1108, 42, 43, 0], None, None);
        computer.step().is_ok();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(vec![0, 42, 43, 0], computer.memory);
    }

    #[test]
    fn test_adjust_relative_base() {
        let mut computer = Computer::new(vec![109, -7], None, None);
        computer.step().is_ok();
        assert_eq!(-7, computer.relative_base);
    }

    #[test]
    fn test_run() {
        let mut computer = Computer::new(vec![1, 1, 1, 4, 99, 5, 6, 0, 99], None, None);
        assert_eq!(30, computer.run().unwrap());
    }
}
