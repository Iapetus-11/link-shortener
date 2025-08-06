use std::io::{self, Write, stdin, stdout};

pub fn take_input(prompt: &str) -> io::Result<String> {
    let mut input = String::new();

    print!("{}", prompt);
    stdout().flush()?;

    stdin().read_line(&mut input).unwrap();

    for char in ['\n', '\r'] {
        input = input.strip_suffix(char).unwrap_or(&input).to_string();
    }

    Ok(input)
}
