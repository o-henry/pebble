fn main() {
    if let Err(error) = screenpebble_lib::run() {
        eprintln!("ScreenPebble failed to start: {error}");
        std::process::exit(1);
    }
}
