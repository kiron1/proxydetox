fn main() {
    #[cfg(windows)]
    windows::build!(
        Windows::Win32::Security::Authentication::Identity::Core::*,
        Windows::Win32::Security::Credentials::*,
        Windows::Win32::Foundation::*,
    );
}
