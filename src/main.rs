use show_my_errors::{AnnotationList, Stylesheet};
use spli::{
    executor::Context,
    parser::{determine_error, program, token, Error, Span},
    ValueKind,
};
use std::{
    io::{self, Read},
    sync::Arc,
};
use rustyline::error::ReadlineError;
use typed_arena::Arena;

fn show_error(filename: &str, content: &str, err: &Error) -> io::Result<()> {
    let annotation = determine_error(content, err).unwrap();
    let mut annotation_list = AnnotationList::new(filename, content);
    annotation_list.add(annotation).unwrap();
    annotation_list.show_stderr(&Stylesheet::colored())
}

fn repl() -> io::Result<()> {
    let lines = Arena::new();
    let ctx = Context::new();
    ctx.add_prelude();
    let mut rl = rustyline::Editor::<()>::new();
    loop {
        let line = rl.readline("spli> ");
        let line = match line {
            Ok(line) => {
                lines.alloc(line)
            },
            Err(ReadlineError::Io(err)) => break Err(err),
            Err(ReadlineError::Eof) => break Ok(()),
            Err(ReadlineError::Interrupted) => continue,
            #[cfg(unix)]
            Err(ReadlineError::Utf8Error) => {
                eprintln!("Had problems decoding your input, probably something with Unicode");
                continue
            },
            #[cfg(unix)]
            Err(ReadlineError::Errno(err)) => {
                eprintln!("Error while reading line: {}", err);
                continue
            },
            #[cfg(windows)]
            Err(ReadlineError::Decode(err)) => {
                eprintln!("Had problems decoding your input, probably something with Unicode");
                continue
            },
            _ => continue
        };
        rl.add_history_entry(line.clone());

        if line.len() != 0 {
            match token(Span::new(line)) {
                Ok((rest, parsed)) => {
                    if rest.fragment().len() == 0 {
                        let result = ctx.evaluate(Arc::new(parsed));
                        println!("{} :: {}", result, result.kind.type_name());
                        if let ValueKind::Exception(exc) = &result.kind {
                            println!("{:#?}", exc);
                        }
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
    }
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
