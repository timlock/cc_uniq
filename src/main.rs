use std::{
    env, error,
    io::{self},
};

use cc_uniq::{parse_args, Processor};

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut args = env::args().skip(1);
    run(
        &mut args,
        || Box::new(io::stdin()),
        || Box::new(io::stdout()),
    )
}

fn run<I, O>(
    args: &mut impl Iterator<Item = String>,
    stdin: I,
    stdout: O,
) -> Result<(), Box<dyn error::Error>>
where
    I: Fn() -> Box<dyn io::Read>,
    O: Fn() -> Box<dyn io::Write>,
{
    let (config, mut destination) = parse_args(args, stdin, stdout)?;

    let processor = Processor::new(config);
    for line in processor {
        let line = line?;

        write!(destination, "{}", line)?;
    }
    writeln!(destination)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::{
        cell::RefCell,
        error::Error,
        io::{self, Cursor, Read, Write},
        rc::Rc,
    };

    use crate::run;

    #[test]
    fn test_step_1() -> Result<(), Box<dyn Error>> {
        let path = "test.txt".to_owned();
        let args = vec![path];

        let stdin_fn = create_stdin_fn("");
        let out_buffer = IoRef::new(Vec::<u8>::new());
        let stdout_fn = create_stdout_fn(out_buffer.clone());

        run(&mut args.into_iter(), stdin_fn, stdout_fn)?;

        let output = String::from_utf8(out_buffer.inner.borrow().to_owned())?;

        assert_eq!("line1\nline2\nline3\nline4\n", output);
        Ok(())
    }

    #[test]
    fn test_step_2() -> Result<(), Box<dyn Error>> {
        let args = vec!["-".to_owned()];

        let stdin_fn = create_stdin_fn("line1\nline2\nline3\nline4");
        let out_buffer = IoRef::new(Vec::<u8>::new());
        let stdout_fn = create_stdout_fn(out_buffer.clone());

        run(&mut args.into_iter(), stdin_fn, stdout_fn)?;

        let output = String::from_utf8(out_buffer.inner.borrow().to_owned())?;

        assert_eq!("line1\nline2\nline3\nline4\n", output);
        Ok(())
    }


    #[test]
    fn test_step_3() -> Result<(), Box<dyn Error>> {
        let path = "test.txt".to_owned();
        let args = vec!["-c".to_owned(), path];

        let stdin_fn = create_stdin_fn("");
        let out_buffer = IoRef::new(Vec::<u8>::new());
        let stdout_fn = create_stdout_fn(out_buffer.clone());

        run(&mut args.into_iter(), stdin_fn, stdout_fn)?;

        let output = String::from_utf8(out_buffer.inner.borrow().to_owned())?;

        assert_eq!("1 line1\n2 line2\n1 line3\n1 line4\n", output);
        Ok(())
    }

    struct IoRef<T> {
        inner: Rc<RefCell<T>>,
    }
    impl<T> IoRef<T>
    where
        T: Write,
    {
        fn new(inner: T) -> Self {
            Self {
                inner: Rc::new(RefCell::new(inner)),
            }
        }

        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }

    impl<T> Write for IoRef<T>
    where
        T: Write,
    {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.inner.borrow_mut().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.inner.borrow_mut().flush()
        }
    }
    fn create_stdout_fn(out_buffer: IoRef<Vec<u8>>) -> impl Fn() -> Box<dyn Write> {
        move || {
            let boxed_cursor: Box<dyn Write> = Box::new(out_buffer.clone());
            boxed_cursor
        }
    }
    fn create_stdin_fn(input: &str) -> impl Fn() -> Box<dyn Read> {
        let input = input.to_owned();
        move || {
            let cursor = Cursor::new(input.clone().into_bytes());
            let boxed_cursor: Box<dyn Read> = Box::new(cursor);
            boxed_cursor
        }
    }
}
