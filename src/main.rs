#![allow(arithmetic_overflow)]

use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, Write}, path::Path, process::Command,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Please provide a option for running the code");
        println!("--jit run code in just in time compiling");
        println!("--inter run code in vm");

        return;
    }

    //really stupid way to do this, but it works
    let mut allow_run = false;

    if args[2] != "--jit" {
        allow_run = true;
    }
    if args[2] != "--inter" {
        allow_run = true;
    }

    if !allow_run {
        println!("Please provide a option for running the code");
        println!("--jit run code in just in time compiling");
        println!("--inter run code in vm");
    }

    let filepath = args[1].clone();

    let program: Vec<char> = fs::read_to_string(filepath).unwrap().chars().collect();

    if args[2] == "--jit" {
        //Now get start the jit instead of the interpreter
        jit_compiling(program);
        return; //Takes ownershit so we need to tell the compiler to stop before we create pointer hell( Best rust feature :))
    }

    let mut index = 0;
    let mut loop_stack: Vec<usize> = Vec::new();

    let mut memory: [u8; 65536] = [0; 65536];
    let mut memory_index = 0;

    let brackets: HashMap<usize, usize> = {
        let mut m = HashMap::new();
        let mut scope_stack = Vec::new();
        for (idx, ch) in program.iter().enumerate() {
            match ch {
                &'[' => {
                    scope_stack.push(idx);
                }
                &']' => {
                    m.insert(scope_stack.pop().unwrap(), idx);
                }
                _ => { /* ignore */ }
            }
        }

        m
    };

    loop {
        if (program.len() <= index) {
            break;
        }

        match program[index] {
            '+' => {
                memory[memory_index] = memory[memory_index] + 1;
            }
            '-' => {
                memory[memory_index] = memory[memory_index] - 1;
            }
            '<' => {
                memory_index = memory_index - 1;
            }
            '>' => {
                memory_index = memory_index + 1;
            }
            ',' => {
                let mut input = String::new();
                let string = std::io::stdin()
                    .read_line(&mut input)
                    .ok()
                    .expect("Failed to read line");
                let bytes = input.bytes().nth(0).expect("no byte read");

                memory[memory_index] = bytes;
            }
            '.' => {
                print!("{}", memory[memory_index] as u8 as char)
            }
            '[' => {
                if memory[memory_index] == 0 {
                    index = brackets[&index];
                } else {
                    loop_stack.push(index);
                }
            }
            ']' => {
                let matching_bracket = loop_stack.pop().unwrap();
                if memory[memory_index] != 0 {
                    index = matching_bracket - 1;
                }
            }

            _ => {}
        }
        index = index + 1;
    }
}

fn jit_compiling(program: Vec<char>) {
    let mut raw_code: Vec<char> = Vec::new();

    //remove all comments
    for n in program {
        match n {
            '+' => raw_code.push('+'),
            '-' => raw_code.push('-'),
            '<' => raw_code.push('<'),
            '>' => raw_code.push('>'),
            '.' => raw_code.push('.'),
            ',' => raw_code.push(','),
            '[' => raw_code.push('['),
            ']' => raw_code.push(']'),
            _ => { //do not add comments to code
            }
        }
    }

    let c_header = include_str!("jitheader.c");

    let mut c_jit_code = String::from(c_header);

    for opcode in raw_code{
        match opcode {
            '+' => c_jit_code.push_str("++*ptr;\n"),
            '-' => c_jit_code.push_str("--*ptr;\n"),
            '<' => c_jit_code.push_str("--ptr;\n"),
            '>' => c_jit_code.push_str("++ptr;\n"),
            '.' => c_jit_code.push_str("putchar(*ptr);\n"),
            ',' => c_jit_code.push_str("*ptr = getchar();\n"),
            '[' => c_jit_code.push_str("while (*ptr) {\n"),
            ']' => c_jit_code.push_str("}"),
            _ => { //do not add comments to code
            }
        }
    }
    c_jit_code.push_str("}");
    let path = Path::new("./temp.c");
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't jit buffer {}: {}", display, why),
        Ok(file) => file,
    };
    
    match file.write_all(c_jit_code.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }




}
