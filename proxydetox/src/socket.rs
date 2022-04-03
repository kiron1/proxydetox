extern "C" {
    // https://developer.apple.com/documentation/xpc/1505523-launch_activate_socket
    #[cfg(target_os = "macos")]
    fn launch_activate_socket(
        name: *const libc::c_char,
        fds: *mut *mut libc::c_int,
        cnt: *mut libc::size_t,
    ) -> libc::c_int;

    // https://man7.org/linux/man-pages/man3/sd_listen_fds.3.html
    // fn sd_listen_fds_with_names(
    //     unset_environment: libc::c_int,
    //     names: *mut *mut *mut libc::c_char,
    // ) -> libc::c_int;
}

/// Pass the name of a socket listed in a launchd.plist, receive `RawFd`s.
///
/// See `man launch` for usage of `launch_activate_socket`.
#[cfg(target_os = "macos")]
pub fn activate_socket(name: &str) -> std::io::Result<Vec<std::os::unix::io::RawFd>> {
    let name = std::ffi::CString::new(name)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let mut fds: *mut libc::c_int = std::ptr::null_mut();
    let mut cnt: libc::size_t = 0;

    let error = unsafe { launch_activate_socket(name.as_ptr(), &mut fds, &mut cnt) };
    if error != 0 {
        return Err(std::io::Error::from_raw_os_error(error));
    }

    let out = unsafe { std::slice::from_raw_parts(fds, cnt).to_vec() };

    unsafe {
        libc::free(fds as *mut _);
    }

    Ok(out)
}

#[cfg(not(target_os = "macos"))]
pub fn activate_socket(_name: &str) -> std::io::Result<Vec<std::os::unix::io::RawFd>> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "not implemented",
    ))
}
