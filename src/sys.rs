pub const fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}
pub const fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c == '_')
}

pub const fn is_alpha_numeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}
