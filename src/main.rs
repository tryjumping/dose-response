#![windows_subsystem = "windows"]

fn main() {
    match dose_response::run() {
        Ok(_) => {
            log::info!("Quitting the program.");
        }
        Err(err) => {
            log::error!("Reached a top-level error: {}", err);
        }
    };
}
