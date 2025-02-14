use std::fs::File;
use std::borrow::Cow;
use std::ffi::CString;
use std::{io, mem, ptr};
use std::fmt::{self, Display};
use std::os::fd::{IntoRawFd, FromRawFd};

use crate::program::{Args, Program};

#[derive(Clone, Debug)]
pub struct Output<'a> {
    status: i32,
    stdout: Cow::<'a, str>,
    stderr: Cow::<'a, str>,
}

impl Display for Output<'_> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter::<'_>) -> fmt::Result {
        write!(f, "{out}", out = self.to_string())
    }
}

impl<'a> Output<'a> {
    #[allow(unused)]
    pub const EMPTY: Self = Self {
        status: 0,
        stdout: Cow::Borrowed(""),
        stderr: Cow::Borrowed("")
    };

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.stdout.is_empty() && self.stderr.is_empty()
    }

    #[inline]
    pub fn from_result<T, E>(result: Result::<T, E>) -> Self
    where
        E: Display,
        T: Into::<Cow::<'a, str>>
    {
        match result {
            Ok(stdout) => Self {
                status: 0,
                stdout: stdout.into(),
                stderr: Cow::Borrowed("")
            },
            Err(e) => Self::from_err(&e, 1)
        }
    }

    #[inline(always)]
    pub fn from_err<E>(err: &E, status: i32) -> Self
    where
        for<'b> &'b E: Display
    {
        Self {
            status,
            stdout: Cow::Borrowed(""),
            stderr: Cow::Owned(err.to_string())
        }
    }

    #[inline]
    pub fn to_string(&self) -> String {
        let Output { status, stdout, stderr } = self;
        match (stdout.is_empty(), stderr.is_empty()) {
            (false, true)  => format!("{stdout}\nstatus: {status}"),
            (true, false)  => format!("{stderr}\nstatus: {status}"),
            (false, false) => format!("{stdout}\n{stderr}\nstatus: {status}"),
            (true, true)   => const { String::new() }
        }
    }
}

pub struct Command;

impl Command {
    pub fn create_pipe() -> io::Result<(File, File)> {
        let mut fds = [0; 2];
        if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let r = unsafe { File::from_raw_fd(fds[0]) };
        let w = unsafe { File::from_raw_fd(fds[1]) };
        Ok((r, w))
    }

    pub fn execute(program: &Program) -> io::Result::<Output<'static>> {
        let (mut stdout_reader, stdout_writer) = Self::create_pipe()?;
        let (mut stderr_reader, stderr_writer) = Self::create_pipe()?;

        let stdout_writer_fd = stdout_writer.into_raw_fd();
        let stderr_writer_fd = stderr_writer.into_raw_fd();

        let c_cmd = CString::new(program.program.as_bytes())?;
        let mut c_args = vec![c_cmd.as_ptr()];
        if let Some(Args { args_nul_terminated, .. }) = &program.args {
            c_args.extend(args_nul_terminated.iter().map(|s| s.as_ptr()));
        }
        c_args.push(ptr::null());

        let mut file_actions = unsafe { mem::zeroed() };
        unsafe {
            _ = libc::posix_spawn_file_actions_init(&mut file_actions);
            _ = libc::posix_spawn_file_actions_adddup2(&mut file_actions, stdout_writer_fd, libc::STDOUT_FILENO);
            _ = libc::posix_spawn_file_actions_adddup2(&mut file_actions, stderr_writer_fd, libc::STDERR_FILENO)
        }

        let mut attr = unsafe { mem::zeroed() };
        unsafe {
            _ = libc::posix_spawnattr_init(&mut attr)
        }

        let env = [c"PATH=/usr/bin:/bin".as_ptr(), ptr::null()];

        let mut pid = 0;
        let ret = unsafe {
            libc::posix_spawn(
                &mut pid,
                c_cmd.as_ptr(),
                &file_actions,
                &attr,
                c_args.as_ptr() as *const _,
                env.as_ptr() as *const _
            )
        };

        if ret != 0 {
            return Err(io::Error::last_os_error())
        }

        unsafe {
            _ = libc::close(stdout_writer_fd);
            _ = libc::close(stderr_writer_fd)
        }

        let mut stdout = io::read_to_string(&mut stdout_reader)?;
        let mut stderr = io::read_to_string(&mut stderr_reader)?;

        #[inline]
        fn trim_in_place(input: &mut String) {
            let trimmed = input.trim();

            let start = trimmed.as_ptr() as usize - input.as_ptr() as usize;
            let end = start + trimmed.len();

            unsafe {
                let bytes = input.as_mut_vec();
                _ = bytes.drain(..start);
                _ = bytes.drain(end - start..)
            }
        }

        unsafe {
            _ = libc::posix_spawn_file_actions_destroy(&mut file_actions);
            _ = libc::posix_spawnattr_destroy(&mut attr)
        }

        let mut status = 0;
        unsafe {
            _ = libc::waitpid(pid, &mut status, 0)
        }

        trim_in_place(&mut stdout);
        trim_in_place(&mut stderr);

        let stdout = Cow::Owned(stdout);
        let stderr = Cow::Owned(stderr);

        Ok(Output {
            stdout,
            stderr,
            status: if libc::WIFEXITED(status) {
                libc::WEXITSTATUS(status)
            } else {
                -1
            }
        })
    }
}
