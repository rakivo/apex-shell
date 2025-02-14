use std::ffi::CString;
use std::io::{self, Error, ErrorKind};

use crate::env::Env;

#[derive(Debug)]
pub struct Args<'a> {
    #[allow(unused)]
    pub args: Vec::<&'a str>,
    pub args_nul_terminated: Vec::<CString>
}

#[derive(Debug)]
pub struct Program<'a> {
    pub args: Option::<Args<'a>>,
    pub program: Box::<str>
}

impl<'a> Program<'a> {
    #[inline(always)]
    pub fn from_str_as_str(s: &'a str) -> (&'a str, Vec::<&'a str>) {
        if let Some(space) = s.find(|c: char| c.is_ascii_whitespace()) {
            let program = s[..space].trim();
            let args = s[space..].trim();
            let args = if args.is_empty() {
                const { Vec::new() }
            } else {
                args.split_ascii_whitespace().collect()
            }; (program, args)
        } else {
            (s.trim(), const { Vec::new() })
        }
    }

    #[inline]
    pub fn from_as_str(program: &'a str, args: Vec::<&'a str>, env: &Env) -> io::Result::<Self> {
        #[inline(always)]
        fn find_executable(path: &str, env: &Env) -> io::Result::<Box::<str>> {
            env.find_executable(path)
                .map(|s| s.to_string_lossy().into_owned().into_boxed_str())
                .ok_or_else(|| {
                    let e = format!("{path} is not found");
                    Error::new(ErrorKind::NotFound, e)
                })
        }

        let program = find_executable(program, env)?;
        let args = if args.is_empty() {
            None
        } else {
            let args_nul_terminated = args.iter()
                .cloned()
                .map(CString::new)
                .filter_map(Result::ok)
                .collect();
            Some(Args {args, args_nul_terminated})
        };
        Ok(Self {program, args})
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn nth_arg(&self, n: usize) -> Option::<&'a str> {
        self.args.as_ref().and_then(|args| args.args.get(n)).cloned()
    }
}
