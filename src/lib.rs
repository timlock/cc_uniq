use std::{
    cmp::min,
    fmt,
    fs::File,
    io::{self, BufRead},
};
pub struct Config {
    pub source: Box<dyn io::BufRead>,
    pub count: bool,
    pub mode: Mode,
}

pub fn parse_args<I, O>(
    args: &mut impl Iterator<Item = String>,
    stdin_fn: I,
    stdout_fn: O,
) -> io::Result<(Config, Box<dyn io::Write>)>
where
    I: Fn() -> Box<dyn io::Read>,
    O: Fn() -> Box<dyn io::Write>,
{
    let args = args.collect::<Vec<_>>();

    let (flags, args) = parse_flags(args.as_slice());
    let mut mode = Mode::All;
    let mut count = false;
    for flag in flags {
        match flag {
            Flag::Repeated => mode = Mode::Repeated,
            Flag::Unqiue => mode = Mode::Unique,
            Flag::Count => count = true,
        }
    }

    let (source, args) = parse_source(args, stdin_fn)?;
    let (destination, args) = parse_destination(args, stdout_fn)?;

    Ok((
        Config {
            source,
            count,
            mode,
        },
        destination,
    ))
}
#[derive(Debug)]
pub enum Mode {
    Repeated,
    Unique,
    All,
}

enum Flag {
    Repeated,
    Unqiue,
    Count,
}

impl Flag {
    fn parse(value: &str) -> Option<Flag> {
        match value {
            "-c" | "--count" => Some(Flag::Count),
            "-d" | "--repeated" => Some(Flag::Repeated),
            "-u" => Some(Flag::Unqiue),
            _ => None,
        }
    }
}

fn parse_flags(args: &[String]) -> (Vec<Flag>, &[String]) {
    let mut flags = Vec::new();

    for i in 0..args.len() {
        let arg = &args[i];
        match Flag::parse(arg) {
            Some(flag) => flags.push(flag),
            None => return (flags, &args[i..]),
        }
    }

    (flags, args)
}

fn parse_source<I: Fn() -> Box<dyn io::Read>>(
    args: &[String],
    stdin_fn: I,
) -> io::Result<(Box<dyn io::BufRead>, &[String])> {
    let source: Box<dyn io::BufRead> = match args.first() {
        Some(path) => {
            if path == "-" {
                Box::new(io::BufReader::new(stdin_fn()))
            } else {
                let file = File::open(path)?;
                Box::new(io::BufReader::new(file))
            }
        }
        None => Box::new(io::BufReader::new(stdin_fn())),
    };

    let begin = min(args.len(), 1);

    Ok((source, &args[begin..]))
}

fn parse_destination<O: Fn() -> Box<dyn io::Write>>(
    args: &[String],
    stdout_fn: O,
) -> io::Result<(Box<dyn io::Write>, &[String])> {
    let destination = match args.first() {
        Some(path) => {
            if path == "-" {
                stdout_fn()
            } else {
                let file = File::create(path)?;
                Box::new(file)
            }
        }
        None => stdout_fn(),
    };

    let begin = min(args.len(), 1);

    Ok((destination, &args[begin..]))
}

pub struct Processor {
    last_line: Option<String>,
    reader: Box<dyn BufRead>,
    mode: Mode,
    count: Option<u32>,
}

impl Processor {
    pub fn new(config: Config) -> Self {
        let count = match config.count {
            true => Some(1),
            false => None,
        };

        Self {
            last_line: None,
            reader: config.source,
            mode: config.mode,
            count,
        }
    }

    fn create_output(&mut self) -> Option<String> {
        match self.last_line.take() {
            Some(last_line) => {
                let output = match self.count.take() {
                    Some(count) => format!("{count} {last_line}"),
                    None => last_line,
                };

                Some(output)
            }
            None => None,
        }
    }
}

impl Iterator for Processor {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut current_line = String::new();
            match self.reader.read_line(&mut current_line) {
                Ok(0) => match self.create_output() {
                    Some(output) => return Some(Ok(output)),
                    None => return None,
                },
                Ok(_) => {}
                Err(err) => return Some(Err(err)),
            }

            match self.last_line {
                Some(ref last_line) => {
                    if current_line != *last_line {
                        let output =  self.create_output().unwrap();
                        self.last_line = Some(current_line);
                        self.count = Some(1);

                        return Some(Ok(output));
                    } else {
                        match self.count {
                            Some(count) => self.count = Some(count + 1),
                            None => panic!("count should be some"),
                        }
                    }
                }
                None => {
                    self.last_line = Some(current_line);
                    self.count = Some(1);
                }
            }
        }
    }
}
