macro_rules! next_and {
    ($it:expr, $pos:expr, $stmt:stmt) => {{
        $it.next();
        *$pos += 1;
        $stmt
    }};
}
