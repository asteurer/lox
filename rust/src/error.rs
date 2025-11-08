pub fn lox_error(line: usize, msg: String) {
    report(line, "".to_string(), msg);
}

pub fn report(line: usize, location: String, msg: String) {
    eprintln!("[line {line}] Error({location}): {msg}");
}
