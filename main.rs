use clap::Parser;
use std::string::String;
use std::vec::Vec;
use std::fs::File;
use std::io::{self, BufRead, Error, ErrorKind};
use std::path::Path;
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// the file to interpret
   input: Option<String>,
}

struct BfProgram {
    program: Vec::<char>,
    data: Vec::<u8>,
    inst_ptr: usize,
    data_ptr: usize
}

impl BfProgram {
    fn new(instructions: Vec::<char>) -> Self {
        Self {
            program: instructions,
            data: vec![0u8],
            inst_ptr: 0,
            data_ptr: 0,
        }
    }

    fn is_finished(&self) -> bool {
        return self.inst_ptr == self.program.len();
    }

    fn run(&mut self) -> io::Result<()> {
        while !self.is_finished() {
            self.perform_instruction()?;
        }

        return Ok(());
    }

    fn perform_instruction(&mut self) -> io::Result<()> {
        let inst = self.program[self.inst_ptr];

        match inst {
            '>' => { self.data_ptr += 1;  },
            '<' => {
                if self.data_ptr == 0 {
                    return Err(Error::new(ErrorKind::Other, "Data pointer out of bounds!"));
                }
                
                self.data_ptr -= 1; 
            },
            '+' => { self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_add(1); },
            '-' => { self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_sub(1); },
            '.' => { print!("{}", self.data[self.data_ptr] as char) },
            ',' => {    
                let mut input = String::new();
                match io::stdin().read_line(&mut input) { 

                    Ok(_) => { if input.len() > 0 { self.data[self.data_ptr] = input.as_bytes()[0]; } },
                    _ => {}
                }
            },
            '[' => {
                if self.data[self.data_ptr] == 0 {
                    self.inst_ptr = self._find_loop_end()?;
                }
            },
            ']' => {
                if self.data[self.data_ptr] != 0 {
                    self.inst_ptr = self._find_loop_beg()?;
                }
            },
            _ => {}
        }

        if self.data_ptr >= self.data.len() {
            self.data.push(0);
        }

        self.inst_ptr += 1;

        return Ok(());
    }

    fn _find_loop_beg(&self) -> io::Result::<usize> {
        let mut ctr = 1;
        let mut pos = self.inst_ptr;

        while 0 < ctr && 0 < pos {
            pos -= 1;
            
            if self.program[pos] == ']' {
                ctr += 1;
            }
            else if self.program[pos] == '[' {
                ctr -= 1;
            }
        }

        if pos == 0 && ctr != 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Unmatched ] in program"));
        }

        return Ok(pos);
    }

    fn _find_loop_end(&self) -> io::Result::<usize> {
        let mut ctr = 1;
        let mut pos = self.inst_ptr;

        while ctr > 0 && pos < (self.program.len() - 1) {
            pos += 1;
            
            if self.program[pos] == '[' {
                ctr += 1;
            }
            else if self.program[pos] == ']' {
                ctr -= 1;
            }
        }

        if pos == self.program.len() {
            return Err(Error::new(ErrorKind::InvalidInput, "Unmatched [ in program"));
        }

        return Ok(pos);
    }
}

fn is_valid_bf_inst(c: char) -> bool {
    if c == '>' || c == '<' || c == '+' || c == '-' || c == '.' || c == ',' || c == '[' || c == ']' {
        return true;
    }
    else {
        return false;
    }
}

fn read_program_from_stdin() -> io::Result<Vec<char>> {
    let mut input = String::new();
    let mut loop_level = 0;

    io::stdin().read_line(&mut input)?;
            

    for c in input.chars() {
        if c == '[' {
            loop_level += 1;
        }
        else if c == ']' {
            loop_level -= 1;
        }
    }   

    if loop_level < 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "Unmatched ] in program"));
    }
    else if loop_level > 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "Unmatched [ in program"));
    }

    return Ok(input.chars().collect::<Vec<char>>());
}

fn load_program(file_name: &String) -> io::Result<Vec<char>> {
    let path = Path::new(&file_name);
    let file = File::open(&path)?;
    
    let mut program: Vec<char> = Vec::<char>::new();
    let mut loop_level = 0; // keep track of open and closed loops

    for line in io::BufReader::new(file).lines() {
        if let Ok(l) = line {
            for c in l.chars() {
                if is_valid_bf_inst(c) {
                    program.push(c);

                    if c == '[' {
                        loop_level += 1;
                    }
                    else if c == ']' {
                        loop_level -= 1;
                    }
                }   
            }
        }
    }

    if loop_level < 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "Unmatched ] in file"));
    }
    else if loop_level > 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "Unmatched [ in file"));
    }

    return Ok(program);
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let program = match args.input {
        Some(file) => load_program(&file).unwrap_or_else(|err| {
            eprintln!("Error reading file: {err}!");
            process::exit(1);
        }),
        None => read_program_from_stdin().unwrap_or_else(|err| {
            eprintln!("Error reading from stdin: {err}!");
            process::exit(1);
        })
    };

    let mut program = BfProgram::new(program);

    match program.run() {
        Ok(_) => { println!(); },
        Err(err) => { 
            eprintln!("Error running program: {err}!");
            process::exit(1);
        }
    }

    return Ok(());
}
