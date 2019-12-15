#[derive(Debug)]
pub struct Computer {
    memory: Vec<i32>,
    loc: i32,
    running: bool,
}

impl Computer {
    pub fn new(memory: Vec<i32>) -> Self {
        Computer { memory: memory, loc: 0, running: true }
    }

    pub fn run(&mut self) -> Result<i32, &'static str> {
        while self.running {
            self.step()?;
        }

        self.result()
    }

    fn step(&mut self) -> Result<(), &'static str> {
        let (target, value) = match self.memory.get(self.loc as usize) {
            Some(1) => 
                (self.memory[(self.loc + 3) as usize], self.mem_lookup(self.loc + 1) + self.mem_lookup(self.loc + 2)),
            Some(2) =>
                (self.memory[(self.loc + 3) as usize], self.mem_lookup(self.loc + 1) * self.mem_lookup(self.loc + 2)),
            Some(99) => {
                self.running = false;
                (0, 0)
            },
            _ => return Err("Unexpected register value!")
        };

        if self.running {
            self.loc = self.loc + 4;
            self.memory[target as usize] = value;
        }

        Ok(())
    }

    fn mem_lookup(&self, location: i32) -> i32 {
        self.memory[self.memory[location as usize] as usize]
    }

    fn result(&self) -> Result<i32, &'static str> {
        match self.memory.get(0) {
            Some(a) => Ok(a.clone()),
            _ => Err("Empty memory!")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_single_add() {
        let mut computer = Computer::new(vec![1, 0, 0, 0, 99]);
        computer.step().is_ok();
        assert_eq!(vec![2, 0, 0, 0, 99], computer.memory);
    }

    #[test]
    fn test_step_single_mutiply() {
        let mut computer = Computer::new(vec![2, 3, 0, 3, 99]);
        computer.step().is_ok();
        assert_eq!(vec![2, 3, 0, 6, 99], computer.memory);
    }

    #[test]
    fn test_step_single_mutiply_long() {
        let mut computer = Computer::new(vec![2, 4, 4, 5, 99, 0]);
        computer.step().is_ok();
        assert_eq!(vec![2, 4, 4, 5, 99, 9801], computer.memory);
    }

    #[test]
    fn test_run() {
        let mut computer = Computer::new(vec![1, 1, 1, 4, 99, 5, 6, 0, 99]);
        assert_eq!(30, computer.run().unwrap());
    }
}
