use std::str;

use super::transform::*;
use edn::Value;
use nom::*;

fn keyword_to_get_op(keyword: &[u8]) -> Box<dyn Operation> {
    Box::new(
        GetOperation {
        key: Value::Keyword(String::from(str::from_utf8(keyword).unwrap())),
    })
}

fn is_whitespace(c: u8) -> bool {
    c.is_ascii_whitespace() || c == b','
}

fn isnt_whitespace(c: u8) -> bool {
    println!("is whitespace? {:?} -> {}", c, is_whitespace(c));
    !is_whitespace(c)
}

named!(identity<&[u8], Box<dyn Operation> >,
    value!(Box::new(IdentityOperation {}), char!('.'))
);

named!(keyword<&[u8], Box<dyn Operation> >,
    map!(preceded!(char!(':'), take_while1!(isnt_whitespace)), keyword_to_get_op)
);

named!(
    map,
    delimited!(tag!("map("), take_while1!(isnt_whitespace), char!(')'))
);

named!(expr<&[u8], Box<dyn Operation> >, alt!(
    identity | keyword
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_identity() {
        assert_eq!(identity(b"."), Ok((&b""[..], IdentityOperation {})));
        assert_eq!(identity(b".."), Ok((&b"."[..], IdentityOperation {})));
    }

    #[test]
    pub fn test_keyword() {
        assert_eq!(
            keyword(b":abc\n"),
            Ok((
                &b"\n"[..],
                GetOperation {
                    key: Value::Keyword("abc".to_owned())
                }
            ))
        );
        assert_eq!(
            keyword(b":abc def"),
            Ok((
                &b" def"[..],
                GetOperation {
                    key: Value::Keyword("abc".to_owned())
                }
            ))
        );
    }
}

/*named!(pub expr, alt!(
    identity | keyword
));*/
