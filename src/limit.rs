struct Rlimit {
    cur: libc::rlim_t,
    max: libc::rlim_t,
}

impl Rlimit {
    fn maxfiles() -> std::io::Result<Rlimit> {
        let mut limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        let rc = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) };
        if rc == 0 {
            Ok(Rlimit {
                cur: limit.rlim_cur,
                max: limit.rlim_max,
            })
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    fn set_maxfiles(new_limit: &Rlimit) -> std::io::Result<()> {
        let limit = libc::rlimit {
            rlim_cur: new_limit.cur,
            rlim_max: new_limit.max,
        };
        let rc = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limit) };
        if rc == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

pub fn update_limits() {
    let mut maxfiles_limit = Rlimit::maxfiles().expect("getrlimit");
    tracing::info!("Currnet number of files limit: {}", maxfiles_limit.cur);
    maxfiles_limit.cur = maxfiles_limit.cur.max(4096);
    Rlimit::set_maxfiles(&maxfiles_limit).expect("setrlimit");
    maxfiles_limit = Rlimit::maxfiles().expect("getrlimit");
    tracing::info!("New number of files limit: {}", maxfiles_limit.cur);
}
