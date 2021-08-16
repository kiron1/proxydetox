use std::process::Command;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Installation of service is not supported")]
    NotSupported,
}

type Result<T> = std::result::Result<T, Error>;

#[cfg(target_os = "linux")]
pub fn install() -> Result<()> {
    use std::io::Write;

    let service_name = "proxydetox.service";
    let exe = std::env::current_exe()?;
    let ini = format!(
        "[Unit]
Description=Proxydetox Daemon
Documentation=https://github.com/kiron1/proxydetox
After=network-online.target
Wants=network-online.target

[Service]
ExecStart={exe}
KillMode=process
RestartSec=5s
Restart=on-failure

[Install]
WantedBy=default.target
",
        exe = exe.display()
    );

    let service_path = {
        let mut p = dirs::home_dir()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not found"))?;
        p.push(".config");
        p.push("systemd");
        p.push("user");
        p.push(&service_name);
        p
    };
    let mut service_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(service_path)?;

    service_file.write_all(ini.as_bytes())?;

    Command::new("systemctl")
        .args(&["--user", "daemon-reload"])
        .status()?;
    Command::new("systemctl")
        .args(&["--user", "enable", &service_name])
        .status()?;
    Command::new("systemctl")
        .args(&["--user", "start", &service_name])
        .status()?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn install() -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let _add = Command::new("reg")
        .args(&[
            "HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v",
            "Proxydetox",
            "/t",
            "REG_SZ",
            "/d",
            current_exe,
            "/f",
        ])
        .status()?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn install() -> Result<()> {
    Err(NotSupported)
}
