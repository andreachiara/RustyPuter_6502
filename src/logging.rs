
pub fn log_error(tag: &str, error: &str) {
    println!("(ERROR) {}: {}", tag, error);
}
pub fn log_warning(tag: &str, error: &str) {
    println!("(WARNING) {}: {}", tag, error);
}
pub fn log_info(tag: &str, error: &str) {
    println!("(INFO) {}: {}", tag, error);
}
