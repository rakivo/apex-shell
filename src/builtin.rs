use std::io::{self, Error, ErrorKind};

use crate::env::Env;

pub enum Builtin<'a> {
    Exit,
    Cd(&'a str)
}

impl<'a> Builtin<'a> {
    pub fn try_from_program(program: &str, args: &[&'a str]) -> crate::Result::<Self> {
        match program {
            "cd" => Ok(Self::Cd(args.get(0).cloned().unwrap_or_default())),
            "exit" => Ok(Self::Exit),
            _ => Err(())
        }
    }

    pub fn run(&self, env: &mut Env) -> io::Result::<String> {
        match self {
            Self::Exit => std::process::exit(0),
            Self::Cd(dir) => {
                let mut cwd = env.cwd.to_owned();
                cwd.push(dir);

                let m = cwd.metadata()?;
                if !m.is_dir() {
                    let e = format!("{dir} is not a directory");
                    let e = Error::new(ErrorKind::InvalidInput, e);
                    return Err(e)
                }

                _ = std::env::set_current_dir(&cwd)?;

                #[inline(always)]
                fn is_dots(bytes: &[u8]) -> bool {
                    if bytes.get(0).map_or(false, |b| *b == b'.') {
                        if bytes.len() == 1 {
                            return true
                        } bytes[1] == b'.'
                    } else {
                        false
                    }
                }

                let cwd = if is_dots(dir.as_bytes()) {
                    // keep the internal cwd pretty
                    cwd.canonicalize()?
                } else {
                    cwd
                };

                env.cwd = cwd;

                const { Ok(String::new() ) }
            }
        }
    }
}
