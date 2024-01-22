use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub enum Value<'x> {
    String(&'x str),
    List(Vec<Value<'x>>),
    Map(HashMap<&'x str, Value<'x>>),
}

impl<'x> Value<'x> {
    pub fn as_string(&self) -> &str {
        match self {
            Value::String(x) => x,
            _ => unreachable!("oof"),
        }
    }
    pub fn as_list(&self) -> &Vec<Value<'x>> {
        match self {
            Value::List(x) => x,
            _ => unreachable!("oof"),
        }
    }
    pub fn as_map(&self) -> &HashMap<&'x str, Value<'x>> {
        match self {
            Value::Map(x) => x,
            _ => unreachable!("oof"),
        }
    }
}

impl<'x> TryFrom<&Value<'x>> for &'x str {
    type Error = anyhow::Error;

    fn try_from(value: &Value<'x>) -> Result<Self, Self::Error> {
        match value {
            Value::String(x) => Ok(x),
            _ => Err(anyhow!("expected string, found {:?}", value)),
        }
    }
}
impl<'x> TryFrom<&Value<'x>> for u32 {
    type Error = anyhow::Error;

    fn try_from(value: &Value<'x>) -> Result<Self, Self::Error> {
        match value {
            Value::String(x) => {
                let r: u32 = x.parse()?;
                Ok(r)
            }
            _ => Err(anyhow!("expected number, found {:?}", value)),
        }
    }
}

fn is_valid_ident(x: u8) -> bool {
    x.is_ascii_alphabetic() || x == b'-'
}

fn lex(input_str: &str) -> Vec<Tok> {
    let mut tokens = Vec::new();

    let input = input_str.as_bytes();
    let length = input.len();
    let mut index = 0;

    macro_rules! simple {
        ($t:ident) => {
            tokens.push($t);
            index += 1;
        };
    }

    while index < length {
        let now = input[index];
        match now {
            b'{' => {
                simple!(LCURLY);
            }
            b'}' => {
                simple!(RCURLY);
            }
            b'[' => {
                simple!(LSQUARE);
            }
            b']' => {
                simple!(RSQUARE);
            }
            b'=' => {
                simple!(EQ);
            }
            b',' => {
                simple!(COMMA);
            }
            b'a'..=b'z' | b'A'..=b'Z' => {
                let start = index;
                while index < length && is_valid_ident(input[index]) {
                    index += 1;
                }
                tokens.push(IDENT(&input_str[start..index]));
            }
            b'"' => {
                index += 1;
                let start = index;
                while index < length && input[index] != b'"' {
                    index += 1;
                }
                assert_eq!(input[index], b'"');
                tokens.push(STRING(&input_str[start..index]));
                index += 1;
            }
            _ => todo!("{}", now as char),
        }
    }

    tokens.push(EOF);

    tokens
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Tok<'x> {
    EOF,

    LCURLY,
    RCURLY,
    LSQUARE,
    RSQUARE,
    EQ,
    COMMA,

    IDENT(&'x str),
    STRING(&'x str),
}
// impl<'x> Tok<'x> {
//     fn is_ident(&self) -> bool {
//         matches!(self, Tok::IDENT(_))
//     }
// }

use anyhow::anyhow;
use Tok::*;

struct Parser<'x> {
    input: Vec<Tok<'x>>,
    offset: usize,
}
impl<'x> Parser<'x> {
    fn len(&self) -> usize {
        self.input.len()
    }
    fn next(&mut self) -> Tok {
        let r = self.input[self.offset];
        self.offset += 1;
        r
    }
}

macro_rules! next {
    ($e:expr) => {{
        let r = $e.input[$e.offset];
        $e.offset += 1;
        r
    }};
}

macro_rules! peek {
    ($e:expr) => {{
        $e.input[$e.offset]
    }};
}

macro_rules! expect {
    ($e:expr, $t:expr) => {{
        let t = $t;
        let x = $e.next();
        assert_eq!(std::mem::discriminant(&x), std::mem::discriminant(&t));
        x
    }};
}

macro_rules! expect_ident {
    ($e:expr) => {{
        let tok = $e.input[$e.offset];
        $e.offset += 1;
        match tok {
            IDENT(s) => s,
            _ => panic!("expected ident, found {:?}", tok),
        }
    }};
}

fn parse_list<'x>(parser: &mut Parser<'x>) -> Value<'x> {
    let mut result = Vec::new();

    expect!(parser, LSQUARE);

    let mut first = true;
    while parser.offset < parser.len() && peek!(parser) != Tok::RSQUARE {
        if !first {
            if peek!(parser) != Tok::COMMA {
                break;
            }
            next!(parser);
        }

        // this should probably be parse_impl that should recognize what kind of token it is, but this is enough for now.. maybe
        let peek = peek!(parser);
        let value = match peek {
            STRING(s) => {
                next!(parser);
                Value::String(s)
            }
            _ => unreachable!("{:?}", peek),
        };

        result.push(value);

        first = false;
    }

    expect!(parser, RSQUARE);

    Value::List(result)
}

fn parse_map<'x>(parser: &mut Parser<'x>, needs_braces: bool) -> Value<'x> {
    if needs_braces {
        expect!(parser, LCURLY);
    }

    let mut result = HashMap::new();

    let mut first = true;
    while parser.offset < parser.len() {
        if !first {
            if peek!(parser) != Tok::COMMA {
                break;
            }
            next!(parser);
        }
        let name = expect_ident!(parser);
        expect!(parser, EQ);

        let value = match peek!(parser) {
            STRING(s) => {
                next!(parser);
                Value::String(s)
            }
            LCURLY => parse_map(parser, true),
            LSQUARE => parse_list(parser),
            _ => todo!(),
        };

        result.insert(name, value);

        first = false;
    }

    if needs_braces {
        expect!(parser, RCURLY);
    }

    Value::Map(result)
}
fn parse_impl<'x>(parser: &mut Parser<'x>) -> Value<'x> {
    parse_map(parser, false)
}
pub fn parse(input: &str) -> Value {
    let tokens = lex(input);
    let mut parser = Parser {
        input: tokens,
        offset: 0,
    };
    parse_impl(&mut parser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_ident() {
        let tokens = lex("abc");
        assert_eq!(tokens.as_slice(), &[IDENT("abc"), EOF]);
    }

    #[test]
    fn lex_eq() {
        let tokens = lex(r#"abc-def="5""#);
        assert_eq!(tokens.as_slice(), &[IDENT("abc-def"), EQ, STRING("5"), EOF]);
    }

    #[test]
    fn parse_simple() {
        let expected = Value::Map(HashMap::from([("abc-def", Value::String("5"))]));
        let found = parse(r#"abc-def="5""#);
        assert_eq!(expected, found);
    }

    #[test]
    fn parse_double() {
        let expected = Value::Map(HashMap::from([
            ("abc-def", Value::String("5")),
            ("bkptno", Value::String("1")),
        ]));
        let found = parse(r#"abc-def="5",bkptno="1""#);
        assert_eq!(expected, found);
    }

    #[test]
    fn parse_submap() {
        let expected = Value::Map(HashMap::from([
            ("bkptno", Value::String("1")),
            (
                "frame",
                Value::Map(HashMap::from([(
                    "addr",
                    Value::String("0x0000000000401000"),
                )])),
            ),
        ]));
        let found = parse(r#"bkptno="1",frame={addr="0x0000000000401000"}"#);
        assert_eq!(expected, found);
    }

    #[test]
    fn parse_list() {
        let expected = Value::Map(HashMap::from([(
            "register-names",
            Value::List(Vec::from([Value::String("rax"), Value::String("rbx")])),
        )]));
        let found = parse(r#"register-names=["rax","rbx"]"#);
        assert_eq!(expected, found);
    }
}

/*

reason="breakpoint-hit",disp="keep",bkptno="1",frame={addr="0x0000000000401000",func="_start",args=[],file="tmp/now.s",fullname="working/tmp/now.s",line="5",arch="i386:x86-64"},thread-id="1",stopped-threads="all",core="4"

*/
