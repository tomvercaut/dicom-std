pub(crate) fn replace_char_at(s: &str, index: usize, c: char) -> String {
    let n = s.len();
    if index >= n {
        return s.to_string();
    }
    let mut t = String::with_capacity(n);
    if index > 0 {
        t.push_str(&s[0..index]);
    }
    t.push(c);
    let m = index + 1;
    if m < n {
        t.push_str(&s[m..]);
    }
    t
}

#[cfg(test)]
mod tests {
    use crate::replace_char_at;

    #[test]
    fn str_replace_char_at() {
        let s = "hello".to_string();
        assert_eq!(replace_char_at(s.as_str(), 0, 'H').as_str(), "Hello");
        assert_eq!(replace_char_at(s.as_str(), 1, 'E').as_str(), "hEllo");
        assert_eq!(replace_char_at(s.as_str(), 2, 'L').as_str(), "heLlo");
        assert_eq!(replace_char_at(s.as_str(), 3, 'L').as_str(), "helLo");
        assert_eq!(replace_char_at(s.as_str(), 4, 'O').as_str(), "hellO");
    }
}
