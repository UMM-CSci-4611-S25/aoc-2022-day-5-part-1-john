use std::{char::ParseCharError, fs, num::ParseIntError, str::FromStr};

static INPUT_FILE: &str = "input.txt";

fn main() {
    let expected_string =format!("Failed to open file '{INPUT_FILE}'"); 
    let contents = 
        fs::read_to_string(INPUT_FILE).expect(&expected_string);

    // This splits the input into two parts, the text before the blank
    // line (`stack_config`) and the part after the blank line (`instructions`).
    let (stack_config, instructions) = contents
        .split_once("\n\n")
        .expect("There was no blank line in the input");

    // The `.parse()` call actually calls the appropriate `from_str()`, which
    // in this case is in the `impl FromStr for Stacks` block.
    let stacks: Stacks = stack_config
        .parse()
        .expect("Failed to parse stack configuration");

    // This `.parse()` call uses the implementation of `from_str()`
    // in the `impl FromStr for CraneInstructions` block.
    let instructions: CraneInstructions = instructions
        .parse()
        .expect("Failed to parse crane instructions");

    // Run all the instructions, returning the final `Stacks` state.
    let final_state = stacks
        .apply_instructions(&instructions)
        .expect("Applying an instruction set failed");

    // Get the top of the stacks and print that out.
    println!(
        "The top of the stacks is {}",
        final_state
            .tops_string()
            .expect("Tried to take the top of an empty stack")
    );
}

#[derive(Debug)]
pub enum ParseError {
    // Add different variants as you discover different kinds of parsing errors.
    // This could include things like too many stacks, illegal strings on a stack, etc.
    InvalidId(String),
    InvalidChar(String),
    InvalidInstruction(String),
}

#[derive(Debug, Default)]
pub struct Stacks {
    stacks: Vec<Stack>
}

#[derive(Debug)]
enum CraneError {
    // Add different variants as you discover different kinds of errors
    // that can occur when applying a crane instruction.
    // This could include things like trying to move from an empty stack,
    // trying to get the top of an empty stack, etc.
    EmptyStack(String),
    IndexError(String)
}

impl Stacks {
    /// Apply a single instruction to the set of stacks in `self`.
    /// Return the new set of stacks, or a `CraneError` if the instruction
    /// is invalid.
    fn apply_instruction(mut self, instruction: &CraneInstruction) -> Result<Self, CraneError> {
        let mut tmp_stack: Vec<char> = Vec::new();

        //Find the indeces of the relevant stacks considering 0 vs 1 based indexing.
        let start_index: usize = instruction.from_stack - 1;
        let end_index: usize = instruction.to_stack - 1;
        let move_num: usize = instruction.num_to_move;
        

        //Process the starting stack as needed.
        let mut active_stack: &mut Stack = match self.stacks.get_mut(start_index) {
            Some(s) => s,
            None => {
                return Err(CraneError::IndexError(std::format!("Could not find a stack at index {start_index}.")))
            }
        };

        if move_num > active_stack.len() {
            return Err(CraneError::EmptyStack(std::format!(
                "Cannot take {count} items from stack {start}, since stack {start} only has {start_count} items.\n
                From stack: {from_stack:?}", 
                count=move_num, 
                start=instruction.from_stack, 
                start_count=active_stack.len(),
                from_stack=active_stack
            )));
        }

        for _ in 0..move_num {
            tmp_stack.push(active_stack.pop());
        }

        //Process the end stacks.
        active_stack = match self.stacks.get_mut(end_index) {
            Some(s) => s,
            None => {
                return Err(CraneError::IndexError(std::format!("Could not find stack at index {end_index}.")));
            }
        };

        for _ in 0..move_num {
            active_stack.push(tmp_stack.pop().unwrap());
        }

        Ok(self)
    }

    /// Perform each of these instructions in order on the set of stacks
    /// in `self`. Return the new set of stacks, or a `CraneError` if
    /// any of the instructions are invalid.
    fn apply_instructions(self, instructions: &CraneInstructions) -> Result<Self, CraneError> {
        let mut output: Result<Self, CraneError> = Ok(self);

        for instr in &instructions.instructions {
            let tmp = match output {
                Ok(s) => s,
                Err(err) => { return Err(err); }
            };
            output = tmp.apply_instruction(instr);

        }

        output
    }

    /// Return a string containing the top character of each stack in order.
    /// The stacks should all be non-empty; if any is empty return a `CraneError`.
    fn tops_string(&self) -> Result<String, CraneError> {
        //Check for empty stacks, and if found, return an error.
        for stack in &self.stacks {
            if stack.is_empty() {
                return Err(CraneError::EmptyStack("Found empty stack.".to_string()));
            }
        }
        Ok(self.stacks.iter().map(Stack::get_last).collect())
    }
}

impl FromStr for Stacks {
    type Err = ParseError;

    // You probably want to use `s.lines()` to create an iterator over the lines (one per stack).
    // Then for each line:
    //   (a) extract the number at the front as the stack number
    //   (b) extract the following characters as the stack contents
    // The function `split_ascii_whitespace()` should prove useful.
    // Note that the stack numbers start at 1 and you'll need the indices
    // in `Stacks::stacks` to start at 0.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut output: Vec<Stack> = Vec::new();
        for line in s.lines() {
            let parse_result: Result<Stack, ParseError> = Stack::from_str(line);
            match parse_result {
                Ok(s) => { output.push(s); },
                Err(err) => { return Err(err); }
            }
        }
        Ok(Self { stacks: output }) 
    }
}

#[derive(Debug, Default)]
pub struct Stack {
    stack: Vec<char>,
}

impl Stack {
    #[must_use]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stack.len() == 0
    }

    pub fn pop(&mut self) -> char {
        let result = self.stack.pop();
        result.unwrap_or(' ')
    }

    pub fn push(&mut self, v: char) {
        self.stack.push(v);
    }

    #[must_use]
    pub fn get_last(&self) -> char {
        *self.stack.last().unwrap_or(&' ')
    }
}

impl FromStr for Stack {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut items: Vec<char> = Vec::new();
        let mut _stack_id: usize = 0;
        for (i, v) in s.split_ascii_whitespace().enumerate() {
            if i == 0 {
                let id_parse_result: Result<usize, ParseIntError> = v.parse();
                _stack_id = match id_parse_result {
                    Ok(s) => s,
                    Err(_err) => { 
                        return Err(ParseError::InvalidId(std::format!("Invalid character for stack id: {v}, expected single digit [0-9]")));
                    }
                }; 
            } else {
                let char_result: Result<char, ParseCharError> = v.parse();
                let stack_char = match char_result {
                    Ok(c) => c,
                    Err(_err) => {
                        return Err(ParseError::InvalidChar(std::format!("Invalid character for stack element: {v}, expected single character [A-Z].")));
                    }
                };
                items.push(stack_char);
            }
        }
        Ok(Self {stack: items })
    }
}

// Implementing `PartialEq<Vec<char>> for Stack` here allows us to
// say things like `vec!['A', 'B', 'C'] == stack`. This is useful
// for testing, where we might want to compare a `Stack` to a `Vec<char>`
// using something like ``assert_eq!(stack, vec!['A', 'B', 'C'])`.
impl PartialEq<Vec<char>> for Stack {
    fn eq(&self, other: &Vec<char>) -> bool {
        &self.stack == other
    }
}

struct CraneInstruction {
    num_to_move: usize,
    from_stack: usize,
    to_stack: usize,
}

impl FromStr for CraneInstruction {
    type Err = ParseError;

    // The instruction specification lines have the form
    //     move 13 from 8 to 7
    // All we need to capture are the three numbers, which happen to
    // be in the odd positions in the input line. I used a `filter` statement
    // to extract those three items from the list, which I could
    // then parse into `usize` using a `map` statement. You could also just
    // "reach" into the split string directly if you find that easier.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let slices: Vec<&str> = s.split_ascii_whitespace().collect();
        let mut vals: Vec<usize> = Vec::new();

        if slices.len() != 6 {
            return Err(ParseError::InvalidInstruction("Invalid crane instruction length. Expected form: \"move <a> from <b> to <c>.\"".to_string()));
        }

        for (i, s) in slices.iter().enumerate() {
            if i % 2 == 1 {
                let parse_result: Result<usize, ParseIntError> = (*s).parse();
                let v = match parse_result {
                    Ok(v) => v,
                    Err(_err) => {
                        return Err(ParseError::InvalidInstruction(std::format!("Error parsing symbol {s}, expected single digit.")));
                    }
                };
                vals.push(v); 
            }
        } 

        let output: Self = Self { 
            num_to_move: *vals.first().unwrap(), 
            from_stack: *vals.get(1).unwrap(), 
            to_stack: *vals.get(2).unwrap(), 
        };
        Ok(output)
    }
}

struct CraneInstructions {
    instructions: Vec<CraneInstruction>,
}

impl FromStr for CraneInstructions {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut instructions: Vec<CraneInstruction> = Vec::new();
        for line in s.lines() {
            let l = line.to_string();
            let line_parts: Vec<&str> = l.split_ascii_whitespace().collect();
            if line_parts.len() != 6 {
                return Err(ParseError::InvalidInstruction("Invalid instruction format.".to_string()))
            }

            let move_count = match line_parts.get(1) {
                Some(s) => s.parse::<usize>(),
                None => {return Err(ParseError::InvalidInstruction("Invalid move count.".to_string())); }
            };

            let orig_stack = match line_parts.get(3) {
                Some(s) => s.parse::<usize>(),
                None => {return Err(ParseError::InvalidInstruction("Invalid source stack.".to_string()));}
            };

            let target_stack = match line_parts.get(5) {
                Some(s) => s.parse::<usize>(),
                None => {return Err(ParseError::InvalidInstruction("Invalid target stack.".to_string()));}
            };

            instructions.push(CraneInstruction { 
                num_to_move: move_count.unwrap(), 
                from_stack: orig_stack.unwrap(), 
                to_stack: target_stack.unwrap(), 
            });
        }
        Ok(Self { instructions })
    }
}

// Don't consider these tests complete or comprehensive. They're just a starting point,
// and you should add more tests to make sure your code works as expected. They all
// start out with the `#[ignore]` attribute, so you'll need to remove that to run them
// as you implement them.
#[cfg(test)]
mod tests {
    use super::*;

    // Test that we can parse stacks correctly.
    #[test]
    fn test_from_str() {
        // The `\` at the end of the line escapes the newline and all following whitespace.
        let input = "1 Z N\n\
                           2 M C D\n\
                           3 P";
        println!("{input}");
        #[allow(clippy::unwrap_used)]
        let stacks: Stacks = input.parse().unwrap();
        assert_eq!(2, stacks.stacks[0].len());
        // The implementation of `PartialEq<Vec<char>>` above is what allows
        // us to compare a `Stack` to a `Vec<char>` here and in other tests.
        assert_eq!(stacks.stacks[0], vec!['Z', 'N']);
        assert_eq!(3, stacks.stacks[1].len());
        assert_eq!(stacks.stacks[1], vec!['M', 'C', 'D']);
        assert_eq!(1, stacks.stacks[2].len());
        assert_eq!(stacks.stacks[2], vec!['P']);
    }

    // Test that we can parse instructions correctly.
    #[test]
    fn test_instruction_parsing() {
        let input = "move 1 from 2 to 1\nmove 3 from 1 to 3";
        let instructions: CraneInstructions = input.parse().unwrap();
        assert_eq!(2, instructions.instructions.len());
        assert_eq!(1, instructions.instructions[0].num_to_move);
        assert_eq!(1, instructions.instructions[0].to_stack);
        assert_eq!(2, instructions.instructions[0].from_stack);
        assert_eq!(3, instructions.instructions[1].num_to_move);
        assert_eq!(3, instructions.instructions[1].to_stack);
        assert_eq!(1, instructions.instructions[1].from_stack);
    }

    // You probably want some tests that check that `apply_instruction` works as expected.
    // You might want to test that it moves the right number of items, that it moves them
    // from the right stack, that it moves them to the right stack, and that it doesn't
    // move items from an empty stack. Below is a simple test that checks that the
    // instruction `move 2 from 0 to 1` moves two items from stack 0 to stack 1, but you
    // probably want more than that.

    // Test that the instruction `move 2 from 0 to 1` works as expected with non-empty
    // stacks.
    #[test]
    fn test_apply_instruction() {
        let stacks = Stacks {
            stacks: vec![
                Stack {
                    stack: vec!['A', 'B', 'C'],
                },
                Stack {
                    stack: vec!['D', 'E', 'F'],
                },
                Stack {
                    stack: vec!['G', 'H', 'I'],
                },
                Stack {stack: Vec::new() },
                Stack {stack: Vec::new() },
                Stack {stack: Vec::new() },
                Stack {stack: Vec::new() },
                Stack {stack: Vec::new() },
                Stack {stack: Vec::new() },
            ],
        };

        let instruction = CraneInstruction {
            num_to_move: 2,
            from_stack: 1,
            to_stack: 2,
        };

        let new_stacks = stacks
            .apply_instruction(&instruction)
            .expect("Failed to apply instruction");

        assert_eq!(new_stacks.stacks[0], vec!['A']);
        assert_eq!(new_stacks.stacks[1], vec!['D', 'E', 'F', 'C', 'B']);
    }

    // This essentially runs `main()` and checks that the results are correct for part 1.
    #[test]
    fn test_part_2() {
        let expected_string = format!("Failed to open file '{INPUT_FILE}'");
        let contents =
            fs::read_to_string(INPUT_FILE).expect(&expected_string);

        let (stack_config, instructions) = contents
            .split_once("\n\n")
            .expect("There was no blank line in the input");

        let stacks: Stacks = stack_config
            .parse()
            .expect("Failed to parse stack configuration");

        let instructions: CraneInstructions = instructions
            .parse()
            .expect("Failed to parse crane instructions");

        let final_state = stacks
            .apply_instructions(&instructions)
            .expect("Applying an instruction set failed");

        let stack_tops = final_state
            .tops_string()
            .expect("Tried to take the top of an empty stack");

        assert_eq!("SBPQRSCDF", stack_tops);
    }
}
