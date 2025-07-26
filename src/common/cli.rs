use std::io::{self, stdin, stdout, Write};

pub fn take_input(prompt: &str) -> io::Result<String> {
    let mut input = String::new();

    print!("{}", prompt);
    stdout().flush()?;

    stdin().read_line(&mut input).unwrap();

    Ok(input)
}