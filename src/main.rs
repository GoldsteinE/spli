use std::io::{self, Write, BufRead};
use spli::parser::token;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    stdout.write(b"spli parser> ")?;
    stdout.flush()?;
    for line in stdin.lock().lines() {
        let line = line?;
        if line.len() != 0 {
            match token(&line) {
                Ok(("", parsed)) => {
                    println!("{}", parsed)
                },
                Ok((rest, _)) => {
                    eprintln!("Not full: {}", rest)
                },
                Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                    eprintln!("Error: {}", nom::error::convert_error(&line, err))
                },
                Err(nom::Err::Incomplete(_)) => { unreachable!() }
            }
        }
        stdout.write(b"spli parser> ")?;
        stdout.flush()?;
    }
    Ok(())
}
