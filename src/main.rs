use std::io::prelude::*;
use std::io;
use std::fs::File;

#[derive(PartialEq, Eq)]
enum ParseResult<T> {
    Nop,
    Eof,
    Node(T),
}

#[derive(PartialEq, Eq, Debug)]
enum LanguageNode {
    IncrementPointer,         // ☃ ->☃
    DecrementPointer,         // ☃ <-☃
    IncrementCounter,         // ❄
    DecrementCounter,         // ☀
    ReadFromStd,              // ☃ ? (no space but is difficult to read without it)
    WriteToStd,               // ☃ ! (ditto)
    Brackets( Box<LanguageNode> ), // unicode ... snowman
    Many( Vec<LanguageNode> ),  // multiple of the nodes above in sequence
}

fn optimise(n: LanguageNode) -> Option<LanguageNode> {
    use ::LanguageNode::*;

    match n {
        Brackets(inner) => {
            let i_opt = optimise(*inner);

            if let Some(o) = i_opt { Some(Brackets(Box::new(o))) } else { None }
        },
        Many(v) => {
            let mut c: usize = 0;
            return None;
            /* while let Some(next) = v.get(c) {
                //match 
            } */
        },
        _ => { None }
    }
}

fn parse<T: Iterator<Item=String>>(strs: &mut T) -> LanguageNode {
    use ::LanguageNode::*;
    use ::ParseResult::*;

    let mut out_vec = vec![];

    loop {
        let parse_result = parse_next(strs);
        if parse_result == ParseResult::Eof { break; }

        if let Node(p) = parse_result { out_vec.push(p); }
    }

    Many(out_vec)
}

fn parse_next<T: Iterator<Item=String>>(strs: &mut T) -> ParseResult<LanguageNode> {
    use ::LanguageNode::*;
    use ::ParseResult::*;

    let maybe_str = strs.next().clone();

    if let Some(n) = maybe_str.map(|a| a.to_string()) {
        let borrow: &str = &n;

        match borrow {
            ">" => Node(IncrementPointer),
            "<" => Node(DecrementPointer),
            "+" => Node(IncrementCounter),
            "-" => Node(DecrementCounter),
            "," => Node(ReadFromStd),
            "." => Node(WriteToStd),
            "[" => Node(Brackets(Box::new(parse(strs)))),
            "]" => Eof,
            _   => Nop,
        }
    } else {
        Eof
    }
}

fn run(program: &LanguageNode) -> () {
    let mut buffer = [0usize; 30000];
    run_params(program, &mut buffer, 0);
}

fn run_params(program: &LanguageNode, buffer: &mut [usize], pointer: usize) -> usize {
    use ::LanguageNode::*;

    match *program {
        IncrementPointer => pointer + 1,
        DecrementPointer => if pointer > 0 { pointer - 1 } else { pointer },
        IncrementCounter => {
            if let Some(counter) = buffer.get_mut(pointer) { *counter += 1 } else { panic!("Cannot read buffer at {}", pointer); }
            
            pointer
        },
        DecrementCounter => {
            if let Some(counter) = buffer.get_mut(pointer) { *counter -= 1 } else { panic!("Cannot read buffer at {}", pointer); }
            
            pointer
        },
        ReadFromStd      => {
            {
                let counter = buffer.get_mut(pointer).unwrap();
                let stdin = io::stdin();
                let mut handle = stdin.lock();
                
                *counter = handle.lines()
                    .next()
                    .unwrap()
                    .unwrap()
                    .trim()
                    .parse::<usize>()
                    .unwrap();
            }

            pointer
        },
        WriteToStd       => {
            if let Some(counter) = buffer.get(pointer) { print!("{} ", *counter); }

            pointer
        },
        Brackets(ref inner)  => {
            let mut point = pointer;

            'brackets: loop {
                let maybe_counter = buffer.get(point).map(|a| *a).clone();
                if let Some(counter) = maybe_counter {
                    if counter == 0 { break 'brackets; }

                    point = run_params(&*inner, buffer, point);
                } else {
                    break 'brackets;
                }
            }

            point
        },
        Many(ref inner)       => {
            let mut point = pointer;
            for node in inner.iter() {
                point = run_params(node, buffer, point);
            }

            point
        }
    }
}

fn main() -> () {
    let mut args = std::env::args().skip(1);
    
    if let Some(first_arg) = args.next() {
        if first_arg == "-l" ||
            first_arg.starts_with("--lit") {
            let mut program = args.next()
                .unwrap()
                .chars()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

            run(&parse(&mut program.into_iter()));
        } else {
            if let Ok(mut file) = File::open(first_arg.clone()) {
                let mut s = String::new();
                if let Ok(_) = file.read_to_string(&mut s) {
                    let mut program = s.chars().map(|s| s.to_string());
                    // let mut p2 = s.split_whitespace().map(|s| s.to_string());
                    // for c in p2 { println!("{}", c); }
                    
                    let prog = parse(&mut program);
                    run(&prog);
                } else {
                    panic!("File read failed for {}.", first_arg);
                }
            } else {
                panic!("File open failed for {}.", first_arg);
            }
        }
    } else { panic!() }
}
