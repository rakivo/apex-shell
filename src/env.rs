use std::env;
use std::path::PathBuf;
use std::os::unix::fs::MetadataExt;

use nix::unistd;

pub struct Env {
    pub cwd: PathBuf,
    pub user: String,
    pub hostname: String,
    pub path_vars: Vec::<String>,
}

impl Env {
    pub fn new() -> anyhow::Result::<Self> {
        let user = env::var("USER")?;
        let path = env::var("PATH")?;
        let path_vars = path.split(':')
            .map(ToOwned::to_owned)
            .collect::<Vec::<_>>();
        let hostname = unistd::gethostname()?
            .into_string()
            .unwrap();
        let cwd = env::current_dir()?;
        Ok(Self { cwd, user, hostname, path_vars })
    }

    #[inline]
    pub fn find_executable(&self, path: &str) -> Option::<PathBuf> {
        #[inline(always)]
        fn is_exe(mode: u32) -> bool {
            mode & 0o111 != 0
        }

        for path_var in self.path_vars.iter() {
            let mut path_buf = PathBuf::from(path_var);
            path_buf.push(path);
            let Ok(m) = path_buf.metadata() else { continue };
            if is_exe(m.mode()) {
                return Some(path_buf)
            }
        } None
    }

    #[inline(always)]
    pub fn ps1(&self) -> String {
        let Self { cwd, user, hostname, .. } = self;
        format!("{user}@{hostname} {cwd}", cwd = cwd.display())
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn pretty_cwd(&self) -> String {
        // let cwd_ = p.to_str().unwrap();
        // let cwd_ = cwd_.strip_prefix("/home/");
        // let cwd_ = cwd_.strip_prefix(&user).unwrap();
        let cwd = "~".to_owned();
        // cwd.push_str(cwd_);
        cwd
    }
}
