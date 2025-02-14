use std::io::{self, Write};

mod env;
mod program;
mod builtin;
mod command;

use env::Env;
use program::Program;
use builtin::Builtin;
use command::{Output, Command};

type Result<T> = std::result::Result::<T, ()>;

// stolen from: <https://docs.rs/crate/likely_stable/latest/source/src/lib.rs>
#[inline(always)]
pub const fn unlikely(b: bool) -> bool {
    if (1i32).checked_div(if b { 0 } else { 1 }).is_none() {
        true
    } else {
        false
    }
}

fn main() -> anyhow::Result::<()> {
    let mut env = Env::new()?;
    loop {
        print!("{ps1}: ", ps1 = env.ps1());
        _ = io::stdout().flush()?;

        let mut buf = String::new();
        let n = io::stdin().read_line(&mut buf)?;

        if unlikely(buf.len() != n) {
            eprintln!("could not read input");
            continue
        }

        let (program, args) = Program::from_str_as_str(&buf);

        if let Ok(builtin) = Builtin::try_from_program(program, &args) {
            let out = Output::from_result(builtin.run(&mut env));
            if !out.is_empty() {
                println!("{out}");
            } continue
        }

        let program = match Program::from_as_str(program, args, &env) {
            Ok(ok) => ok,
            Err(e) => {
                println!("{e}");
                continue
            }
        };

        match Command::execute(&program) {
            Ok(out) => println!("{out}"),
            Err(e) => eprintln!("error: {e}")
        }
    }
}
