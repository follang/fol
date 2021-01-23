pub fn is_eof(ch: &char) -> bool {
    return *ch == '\0';
}

pub fn is_eol(ch: &char) -> bool {
    return *ch == '\n' || *ch == '\r';
}

pub fn is_space(ch: &char) -> bool {
    return *ch == ' ' || *ch == '\t';
}

pub fn is_digit(ch: &char) -> bool {
    return '0' <= *ch && *ch <= '9';
}

pub fn is_alpha(ch: &char) -> bool {
    return 'a' <= *ch && *ch <= 'z' || 'A' <= *ch && *ch <= 'Z' || *ch == '_';
}

pub fn is_bracket(ch: &char) -> bool {
    return *ch == '{' || *ch == '[' || *ch == '(' || *ch == ')' || *ch == ']' || *ch == '}';
}

pub fn is_symbol(ch: &char) -> bool {
    return '!' <= *ch && *ch <= '/'
        || ':' <= *ch && *ch <= '@'
        || '[' <= *ch && *ch <= '^'
        || '{' <= *ch && *ch <= '~';
}

pub fn is_oct_digit(ch: &char) -> bool {
    return '0' <= *ch && *ch <= '7' || *ch == '_';
}
pub fn is_hex_digit(ch: &char) -> bool {
    return '0' <= *ch && *ch <= '9'
        || 'a' <= *ch && *ch <= 'f'
        || 'A' <= *ch && *ch <= 'F'
        || *ch == '_';
}

pub fn is_alphanumeric(ch: &char) -> bool {
    return is_digit(ch) || is_alpha(ch);
}

pub fn is_void(ch: &char) -> bool {
    return is_eol(ch) || is_space(ch);
}
