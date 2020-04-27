use show_my_errors::{AnnotationList, Stylesheet};
use spli::parser::{determine_error, program, token, Error, Span};
use std::io::{self, BufRead, Read, Write};

fn show_error(filename: &str, content: &str, err: &Error) -> io::Result<()> {
    let annotation = determine_error(content, err).unwrap();
    let mut annotation_list = AnnotationList::new(filename, content);
    annotation_list.add(annotation).unwrap();
    annotation_list.show_stderr(&Stylesheet::colored())
}

fn repl() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    stdout.write(b"spli parser> ")?;
    stdout.flush()?;
    for line in stdin.lock().lines() {
        let line = line?;
        if line.len() != 0 {
            match token(Span::new(&line)) {
                Ok((rest, parsed)) => {
                    if rest.fragment().len() == 0 {
                        println!("{} :: {}", parsed, parsed.kind.type_name())
                    } else {
                        println!("Parsed: {}", parsed);
                        println!("Rest: {}", rest);
                    }
                }
                Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                    show_error("<stdin>", &line, &err)?;
                }
                Err(nom::Err::Incomplete(_)) => unreachable!(),
            }
        }
        stdout.write(b"spli parser> ")?;
        stdout.flush()?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    if let Some(filename) = std::env::args().skip(1).next() {
        let mut file = std::fs::File::open(&filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        match program(Span::new(&contents)) {
            Ok((rest, parsed)) => {
                if rest.fragment().len() == 0 {
                    println!("Syntax OK");
                } else {
                    println!("Parsed this: {}", parsed);
                    println!("Extra symbols at the end of program: {}", rest);
                }
            }
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                show_error(&filename, &contents, &err)?;
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
        Ok(())
    } else {
        repl()
    }
}
