fn main() {
    windows::build!(
        bindings::windows::win32::security::*,
        bindings::windows::win32::system_services::LARGE_INTEGER,
    );
}
