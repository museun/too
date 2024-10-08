use std::fmt::{Debug, Formatter, Result};

pub const fn str(s: &str) -> impl Debug + '_ {
    struct NoQuote<'a>(&'a str);
    impl<'a> Debug for NoQuote<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.write_str(self.0)
        }
    }
    NoQuote(s)
}

pub fn short_name(name: &str) -> String {
    const fn is_special(c: char) -> bool {
        matches!(c, ' ' | '<' | '>' | '(' | ')' | '[' | ']' | ',' | ';')
    }

    fn collapse(s: &str) -> &str {
        s.split("::").last().unwrap()
    }

    let mut index = 0;
    let end = name.len();
    let mut out = String::new();

    while index < end {
        let rest = &name[index..end];
        if let Some(mut p) = rest.find(is_special) {
            out.push_str(collapse(&rest[0..p]));

            let ch = &rest[p..=p];
            out.push_str(ch);

            if matches!(ch, ">" | ")" | "]" if rest[p + 1..].starts_with("::")) {
                out.push_str("::");
                p += 2;
            }
            index += p + 1;
        } else {
            out.push_str(collapse(rest));
            index = end;
        }
    }
    out
}
