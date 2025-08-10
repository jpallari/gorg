pub fn is_punctuation(ch: char) -> bool {
    ch.is_whitespace()
        || (ch >= '!' && ch <= '/')
        || (ch >= ':' && ch <= '@')
        || (ch >= '[' && ch <= '`')
        || (ch >= '{' && ch <= '~')
}
