use std::collections::{VecDeque, HashMap};
use std::fmt;
use std::hash::Hash;
use std::sync::mpsc::{SyncSender, Receiver};
use num::{Integer, Signed, FromPrimitive};

pub enum ComputerOutput<T: Signed + Integer> {
    Queue(VecDeque<T>),
    Channel(SyncSender<T>)
}

pub enum ComputerInput<T: Signed + Integer> {
    Queue(VecDeque<T>),
    Channel(Receiver<T>)
}

pub struct Computer<T: Signed + Integer> {
    memory: HashMap<T, T>,
    loc: T,
    running: bool,
    input: ComputerInput<T>,
    output: ComputerOutput<T>,
    alt_output: VecDeque<T>,
    relative_base: T,
}

fn convert<T: FromPrimitive>(value: usize) -> T {
    T::from_usize(value).unwrap()
}

impl<T: Signed + Integer + fmt::Debug> fmt::Debug for Computer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory: {:?} Location: {:?} Running: {:?}", self.memory, self.loc, self.running)
    }
}

impl<T> Computer<T> where T: Integer + Signed + Copy + FromPrimitive + Hash + fmt::Display {
    /// Creates a new Computer.
    ///
    /// The function takes the intial memory state for the computer, plus an optional input and
    /// output. An empty queue is used as the default for input and output, if no alternative is
    /// supplied.
    pub fn new(memory: Vec<T>, input: Option<ComputerInput<T>>, output: Option<ComputerOutput<T>>) -> Self {
        let mem_map = (0..).zip(memory).map(|(k,v)| (convert(k), v.clone())).collect();
        Computer { 
            memory: mem_map, 
            loc: convert(0), 
            running: true, 
            input: input.unwrap_or(ComputerInput::Queue(VecDeque::new())),
            output: output.unwrap_or(ComputerOutput::Queue(VecDeque::new())),
            alt_output: VecDeque::new(),
            relative_base: convert(0)
        }
    }

    pub fn run(&mut self) -> Result<T, String> {
        while self.running {
            self.step()?;
        }

        self.result()
    }

    fn step(&mut self) -> Result<(), String> {
        let current_mem_value = self.memory.get(&self.loc);
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

    fn read_instruction_code(code: T) -> Result<(T, Vec<ArgumentKind>), String> {
        let min_opcode = convert(1);
        let max_opcode = convert(99);
        let divisor = convert(100);

        if code < min_opcode {
            return Err(format!("Opcode must be positive, but got {}", code));
        }

        let abs_code = code.abs();
        if abs_code <= max_opcode {
            return Ok((code, vec![]));
        }

        let prefix = (abs_code / divisor).to_string();
        if !prefix.chars().all(|x| x == '0' || x == '1' || x == '2') {
            return Err(format!("Unrecognised opcode format: {}", code));
        }
        
        Ok((code % divisor, (code.abs() / divisor).to_string().chars().rfold(vec![], |mut acc, x| match x {
            '0' => { acc.push(ArgumentKind::Position); acc },
            '1' => { acc.push(ArgumentKind::Immediate); acc },
            _ => { acc.push(ArgumentKind::Relative); acc }
        })))
    }

    fn result(&self) -> Result<T, String> {
        let target = convert(0);
        match self.memory.get(&target) {
            Some(a) => Ok(a.clone()),
            _ => Err(String::from("Empty memory!"))
        }
    }

    pub fn output(&self) -> VecDeque<T> {
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
struct Argument<T> {
    value: T,
    kind: ArgumentKind,
    relative_base: T
}

impl<T> Argument<T> where T: Integer + Signed + Copy + Hash {
    fn new(value: T, kind: Option<&ArgumentKind>, relative_base: T) -> Self {
        Argument { value: value, kind: kind.cloned().unwrap_or(ArgumentKind::Position), relative_base: relative_base }
    }

    fn get<'a>(&self, memory: &'a HashMap<T, T>) -> Option<T> {
        match self.kind {
            ArgumentKind::Immediate => Some(self.value.clone()),
            ArgumentKind::Position => memory.get(&self.value).cloned(),
            ArgumentKind::Relative => memory.get(&(self.value + self.relative_base)).cloned()
        }
    }

    fn set(&self, memory: &mut HashMap<T,T>, new_value: T) -> Result<(), String> {
        match self.kind {
            ArgumentKind::Immediate => Err(String::from("Can't populate Immediate argument.")),
            ArgumentKind::Position => {
                memory.insert(self.value, new_value);
                Ok(())
            },
            ArgumentKind::Relative => {
                memory.insert(self.value + self.relative_base, new_value);
                Ok(())
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Instruction<T> {
    Add(Argument<T>, Argument<T>, Argument<T>),
    Multiply(Argument<T>, Argument<T>, Argument<T>),
    Input(Argument<T>),
    Output(Argument<T>),
    JumpIfTrue(Argument<T>, Argument<T>),
    JumpIfFalse(Argument<T>, Argument<T>),
    LessThan(Argument<T>, Argument<T>, Argument<T>),
    Equals(Argument<T>, Argument<T>, Argument<T>),
    AdjustRelativeBase(Argument<T>),
    Stop
}

#[derive(PartialEq, Eq, Debug)]
enum CallResult<T> {
    Step(T),
    Jump(T),
    Stop
}

impl<T> Instruction<T> where T: Integer + Signed + Copy + FromPrimitive + Hash + fmt::Display {
    fn new(code: T, base_location: T, argument_types: Vec<ArgumentKind>, memory: &HashMap<T,T>, relative_base: T) -> Result<Self, String> {
        let address = |x: T| *(memory.get(&x).unwrap());
        let add: T = convert(1);
        let multiply: T = convert(2);
        let input: T = convert(3);
        let output: T = convert(4);
        let jump_if_true: T = convert(5);
        let jump_if_false: T = convert(6);
        let less_than: T = convert(7);
        let equals: T = convert(8);
        let adjust_relative_base: T = convert(9);
        let stop: T = convert(99);
        match code {
            a if a == add => {
                Ok(Instruction::Add(Argument::new(address(base_location + convert(1)), argument_types.get(0), relative_base),
                                    Argument::new(address(base_location + convert(2)), argument_types.get(1), relative_base),
                                    Argument::new(address(base_location + convert(3)), argument_types.get(2), relative_base)))
            },
            a if a == multiply => {
                Ok(Instruction::Multiply(Argument::new(address(base_location + convert(1)), argument_types.get(0), relative_base),
                                         Argument::new(address(base_location + convert(2)), argument_types.get(1), relative_base),
                                         Argument::new(address(base_location + convert(3)), argument_types.get(2), relative_base)))
            },
            a if a == input => {
                Ok(Instruction::Input(Argument::new(address(base_location + convert(1)), argument_types.get(0), relative_base)))
            },
            a if a == output => {
                Ok(Instruction::Output(Argument::new(address(base_location + convert(1)), argument_types.get(0), relative_base)))
            },
            a if a == jump_if_true => {
                Ok(Instruction::JumpIfTrue(Argument::new(address(base_location + convert(1)), argument_types.get(0), relative_base),
                                           Argument::new(address(base_location + convert(2)), argument_types.get(1), relative_base)))
            },
            a if a == jump_if_false => {
                Ok(Instruction::JumpIfFalse(Argument::new(address(base_location + convert(1)), argument_types.get(0), relative_base),
                                            Argument::new(address(base_location + convert(2)), argument_types.get(1), relative_base)))
            },
            a if a == less_than => {
                Ok(Instruction::LessThan(Argument::new(address(base_location + convert(1)), argument_types.get(0), relative_base),
                                         Argument::new(address(base_location + convert(2)), argument_types.get(1), relative_base),
                                         Argument::new(address(base_location + convert(3)), argument_types.get(2), relative_base)))
            },
            a if a == equals => {
                Ok(Instruction::Equals(Argument::new(address(base_location + convert(1)), argument_types.get(0), relative_base),
                                       Argument::new(address(base_location + convert(2)), argument_types.get(1), relative_base),
                                       Argument::new(address(base_location + convert(3)), argument_types.get(2), relative_base)))
            },
            a if a == adjust_relative_base => {
                Ok(Instruction::AdjustRelativeBase(Argument::new(address(base_location + convert(1)), argument_types.get(0), relative_base)))
            },
            a if a == stop => Ok(Instruction::Stop),
            x => Err(format!("Unsupported instruction: {}", x))
        }
    }

    fn length(&self) -> T {
        match self {
            Instruction::Add(_,_,_) => convert(4),
            Instruction::Multiply(_,_,_) => convert(4),
            Instruction::Input(_) => convert(2),
            Instruction::Output(_) => convert(2),
            Instruction::JumpIfTrue(_,_) => convert(3),
            Instruction::JumpIfFalse(_,_) => convert(3),
            Instruction::LessThan(_,_,_) => convert(4),
            Instruction::Equals(_,_,_) => convert(4),
            Instruction::AdjustRelativeBase(_) => convert(2),
            Instruction::Stop => convert(0)
        }
    }

    fn call<'a>(&self, 
                memory: &mut HashMap<T,T>, 
                reader: &mut ComputerInput<T>, 
                writer: &mut ComputerOutput<T>, 
                alt_output: &mut VecDeque<T>,
                relative_base: &mut T) -> Result<CallResult<T>, String> {
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

    fn add(&self, input1: &Argument<T>, input2: &Argument<T>, output: &Argument<T>, memory: &mut HashMap<T,T>) -> Result<CallResult<T>, String> {
        let result = input1.get(memory).unwrap_or_else(|| convert(0)) + input2.get(memory).unwrap_or_else(|| convert(0));
        output.set(memory, result).and(Ok(CallResult::Step(self.length())))
    }

    fn multiply(&self, input1: &Argument<T>, input2: &Argument<T>, output: &Argument<T>, memory: &mut HashMap<T,T>) -> Result<CallResult<T>, String> {
        let result = input1.get(memory).unwrap_or_else(|| convert(0)) * input2.get(memory).unwrap_or_else(|| convert(0));
        output.set(memory, result).and(Ok(CallResult::Step(self.length())))
    }

    fn input<'a>(&self, destination: &Argument<T>, memory: &mut HashMap<T,T>, input: &mut ComputerInput<T>) -> Result<CallResult<T>, String> {
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

    fn output<'a>(&self, source: &Argument<T>, memory: &mut HashMap<T,T>, output: &mut ComputerOutput<T>, alt_output: &mut VecDeque<T>) -> Result<CallResult<T>, String> {
        let value = source.get(memory).unwrap_or_else(|| convert(0));
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
    }

    fn jump_if_true(&self, input: &Argument<T>, target: &Argument<T>, memory: &mut HashMap<T,T>) -> Result<CallResult<T>, String> {
        let test_val = input.get(memory).unwrap_or_else(|| convert(0));
        if test_val == convert(0) {
            Ok(CallResult::Step(self.length()))
        } else {
            Ok(CallResult::Jump(target.get(memory).unwrap_or_else(|| convert(0))))
        }
    }

    fn jump_if_false(&self, input: &Argument<T>, target: &Argument<T>, memory: &mut HashMap<T,T>) -> Result<CallResult<T>, String> {
        let test_val = input.get(memory).unwrap_or_else(|| convert(0));
        if test_val == convert(0) {
            Ok(CallResult::Jump(target.get(memory).unwrap_or_else(|| convert(0))))
        } else {
            Ok(CallResult::Step(self.length()))
        }
    }
    
    fn less_than(&self, input1: &Argument<T>, input2: &Argument<T>, output: &Argument<T>, memory: &mut HashMap<T,T>) -> Result<CallResult<T>, String> {
        let value1 = input1.get(memory).unwrap_or_else(|| convert(0));
        let value2 = input2.get(memory).unwrap_or_else(|| convert(0));
        output.set(memory, if value1 < value2 { convert(1) } else { convert(0) }).and(Ok(CallResult::Step(self.length())))
    }

    fn equals(&self, input1: &Argument<T>, input2: &Argument<T>, output: &Argument<T>, memory: &mut HashMap<T,T>) -> Result<CallResult<T>, String> {
        let value1 = input1.get(memory).unwrap_or_else(|| convert(0));
        let value2 = input2.get(memory).unwrap_or_else(|| convert(0));
        output.set(memory, if value1 == value2 { convert(1) } else { convert(0) }).and(Ok(CallResult::Step(self.length())))
    }

    fn adjust_relative_base(&self, input: &Argument<T>, memory: &mut HashMap<T,T>, relative_base: &mut T) -> Result<CallResult<T>, String> {
        *relative_base = *relative_base + input.get(memory).unwrap_or_else(|| convert(0));
        Ok(CallResult::Step(self.length()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::sync_channel;
    use std::thread;

    fn hash_with_indexes<T: Integer + Copy + FromPrimitive + Hash>(vec: Vec<T>) -> HashMap<T,T> {
        (0..).zip(vec).map(|(k,v)| (convert(k), v.clone())).collect()
    }

    #[test]
    fn test_new_instruction_add() {
        let instruction = Instruction::new(1, 0, vec![ArgumentKind::Position, ArgumentKind::Immediate], &hash_with_indexes(vec![1,2,3,4]), 0).unwrap();
        assert_eq!(Instruction::Add(Argument { value: 2, kind: ArgumentKind::Position, relative_base: 0},
                                    Argument { value: 3, kind: ArgumentKind::Immediate, relative_base: 0},
                                    Argument { value: 4, kind: ArgumentKind::Position, relative_base: 0 }),
                   instruction);
    }

    #[test]
    fn test_new_instruction_mutiply() {
        let instruction = Instruction::new(2, 1, vec![ArgumentKind::Position, ArgumentKind::Immediate], &hash_with_indexes(vec![3, 4, 5, 6, 7]), 0).unwrap();
        assert_eq!(Instruction::Multiply(Argument { value: 5, kind: ArgumentKind::Position, relative_base: 0 },
                                         Argument { value: 6, kind: ArgumentKind::Immediate, relative_base: 0 },
                                         Argument { value: 7, kind: ArgumentKind::Position, relative_base: 0 }),
                   instruction);
    }

    #[test]
    fn test_new_instruction_stop() {
        let instruction = Instruction::new(99, 0, vec![], &HashMap::new(), 0).unwrap();
        assert_eq!(Instruction::Stop,
                   instruction);
    }

    #[test]
    fn test_positional_argument_get() {
        let argument = Argument::new(3, Some(&ArgumentKind::Position), 0);
        let result = argument.get(&hash_with_indexes(vec![11, 12, 13, 14])).unwrap();
        assert_eq!(14, result);
    }

    #[test]
    fn test_positional_argument_set() {
        let argument = Argument::new(3, Some(&ArgumentKind::Position), 0);
        let mut memory = hash_with_indexes(vec![11, 12, 13, 14]);
        let expected = hash_with_indexes(vec![11, 12, 13, 42]);
        argument.set(&mut memory, 42).unwrap();
        assert_eq!(expected, memory);
    }

    #[test]
    fn test_immediate_argument_get() {
        let argument = Argument::new(3, Some(&ArgumentKind::Immediate), 0);
        let result = argument.get(&hash_with_indexes(vec![11, 12, 13, 14])).unwrap();
        assert_eq!(3, result);
    }

    #[test]
    fn test_relative_argument_get() {
        let argument = Argument::new(1, Some(&ArgumentKind::Relative), 1);
        let result = argument.get(&hash_with_indexes(vec![1, 2, 3, 4])).unwrap();
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
        computer.step().unwrap();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![2, 0, 0, 0, 99]), computer.memory);
    }

    #[test]
    fn test_step_single_mutiply() {
        let mut computer = Computer::new(vec![2, 3, 0, 3, 99], None, None);
        computer.step().unwrap();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![2, 3, 0, 6, 99]), computer.memory);
    }

    #[test]
    fn test_step_single_mutiply_long() {
        let mut computer = Computer::new(vec![2, 4, 4, 5, 99, 0], None, None);
        computer.step().unwrap();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![2, 4, 4, 5, 99, 9801]), computer.memory);
    }
    
    #[test]
    fn test_step_input() {
        let mut input = VecDeque::new();
        input.push_back(42);
        let mut computer = Computer::new(vec![3, 2, 0], Some(ComputerInput::Queue(input)), None);
        computer.step().unwrap();
        assert_eq!(2, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![3, 2, 42]), computer.memory);
    }

    #[test]
    fn test_step_input_with_channel() {
        let (tx, rx) = sync_channel(0);

        thread::spawn(move || {
            tx.send(42).unwrap();
        });

        let mut computer = Computer::new(vec![3, 2, 0], Some(ComputerInput::Channel(rx)), None);
        computer.step().unwrap();
        assert_eq!(2, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![3, 2, 42]), computer.memory);
    }

    #[test]
    fn test_step_output() {
        let mut computer = Computer::new(vec![4, 2, 42], None, None);
        computer.step().unwrap();
        assert_eq!(2, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![4, 2, 42]), computer.memory);
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
            computer.step().unwrap();
            assert_eq!(2, computer.loc);
            assert_eq!(hash_with_indexes(vec![4, 2, 42]), computer.memory);
        });

        assert_eq!(42, rx.recv().unwrap());
    }

    #[test]
    fn test_step_output_with_channel_no_receiver() {
        let (tx, rx) = sync_channel(0);
        drop(rx);
        let mut computer = Computer::new(vec![4, 2, 43], None, Some(ComputerOutput::Channel(tx)));
        computer.step().unwrap();
        assert_eq!(2, computer.loc);
        assert_eq!(hash_with_indexes(vec![4, 2, 43]), computer.memory);
        assert_eq!(43, computer.output().pop_front().unwrap());
    }

    #[test]
    fn test_step_jump_if_true_true() {
        let mut computer = Computer::new(vec![1105, 1, 20], None, None);
        computer.step().unwrap();
        assert_eq!(20, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![1105, 1, 20]), computer.memory);
    }

    #[test]
    fn test_step_jump_if_true_true_positional() {
        let mut computer = Computer::new(vec![5, 0, 2], None, None);
        computer.step().unwrap();
        assert_eq!(2, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![5, 0, 2]), computer.memory);
    }

    #[test]
    fn test_step_jump_if_true_false() {
        let mut computer = Computer::new(vec![1105, 0, 20], None, None);
        computer.step().unwrap();
        assert_eq!(3, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![1105, 0, 20]), computer.memory);
    }

    #[test]
    fn test_step_jump_if_true_false_positional() {
        let mut computer = Computer::new(vec![5, 3, 20, 0], None, None);
        computer.step().unwrap();
        assert_eq!(3, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![5, 3, 20, 0]), computer.memory);
    }

    #[test]
    fn test_step_jump_if_false_false() {
        let mut computer = Computer::new(vec![1106, 0, 20], None, None);
        computer.step().unwrap();
        assert_eq!(20, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![1106, 0, 20]), computer.memory);
    }

    #[test]
    fn test_step_jump_if_false_false_positional() {
        let mut computer = Computer::new(vec![6, 3, 3, 0], None, None);
        computer.step().unwrap();
        assert_eq!(0, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![6, 3, 3, 0]), computer.memory);
    }

    #[test]
    fn test_step_jump_if_false_true() {
        let mut computer = Computer::new(vec![1106, 1, 20], None, None);
        computer.step().unwrap();
        assert_eq!(3, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![1106, 1, 20]), computer.memory);
    }

    #[test]
    fn test_step_less_than_true() {
        let mut computer = Computer::new(vec![1107, -1, 0, 0], None, None);
        computer.step().unwrap();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![1, -1, 0, 0]), computer.memory);
    }

    #[test]
    fn test_step_less_than_false() {
        let mut computer = Computer::new(vec![1107, 10, 10, 0], None, None);
        computer.step().unwrap();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![0, 10, 10, 0]), computer.memory);
    }

    #[test]
    fn test_step_equals_true() {
        let mut computer = Computer::new(vec![1108, 42, 42, 0], None, None);
        computer.step().unwrap();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![1, 42, 42, 0]), computer.memory);
    }

    #[test]
    fn test_step_equals_false() {
        let mut computer = Computer::new(vec![1108, 42, 43, 0], None, None);
        computer.step().unwrap();
        assert_eq!(4, computer.loc);
        assert_eq!(true, computer.running);
        assert_eq!(hash_with_indexes(vec![0, 42, 43, 0]), computer.memory);
    }

    #[test]
    fn test_adjust_relative_base() {
        let mut computer = Computer::new(vec![109, -7], None, None);
        computer.step().unwrap();
        assert_eq!(-7, computer.relative_base);
    }

    #[test]
    fn test_run() {
        let mut computer = Computer::new(vec![1, 1, 1, 4, 99, 5, 6, 0, 99], None, None);
        assert_eq!(30, computer.run().unwrap());
    }

    #[test]
    fn test_64bit() {
        let mut computer: Computer<i64> = Computer::new(vec![104i64,1125899906842624i64,99i64], None, None);
        computer.run().unwrap();
        assert_eq!(vec![1125899906842624i64], Vec::from(match computer.output {
            ComputerOutput::Queue(q) => q,
            _ => VecDeque::new()
        }));
    }
}
