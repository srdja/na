extern crate gcc;

#[cfg(target_os = "linux")]
fn main() {
    gcc::compile_library("libip.a", &["src/ip_linux.c"]);
}

#[cfg(target_os = "win32")]
fn main() {
    gcc::compile_library("libip.a", &["src/ip_win.c"]);
}
