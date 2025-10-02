//! A markdown parser for Xrust

#![allow(dead_code)]

use std::rc::Rc;
use xrust::item::Node;
use xrust::parser::combinators::alt::{alt2, alt4};
use xrust::parser::combinators::list::separated_list0;
use xrust::parser::combinators::many::{many0, many1};
use xrust::parser::combinators::map::map;
use xrust::parser::combinators::tag::tag;
use xrust::parser::combinators::tuple::tuple3;
use xrust::parser::combinators::whitespace::whitespace0;
use xrust::parser::{ParseError, ParseInput, ParserState};
use xrust::trees::smite::RNode;
use xrust::value::Value;
use xrust::xdmerror::{Error, ErrorKind};

pub fn parse(input: &str) -> Result<RNode, Error> {
    let d = RNode::new_document();
    if input.is_empty() {
        return Ok(d);
    }
    let state = ParserState::new(Some(d.clone()), None, None);
    md_expr((input.trim(), state))
        .map(|_| d)
        .map_err(|e| match e {
            ParseError::Combinator => Error::new(
                ErrorKind::ParseError,
                String::from("unrecoverable parse error"),
            ),
            ParseError::NotWellFormed(f) => Error::new(
                ErrorKind::ParseError,
                format!("unrecognised extra characters \"{}\"", f),
            ),
            _ => Error::new(ErrorKind::Unknown, String::from("unknown error")),
        })
}

fn md_expr<N: Node>(input: ParseInput<N>) -> Result<(ParseInput<N>, N), ParseError> {
    match document()(input) {
        Err(e) => Err(e),
        Ok(((input1, state1), n)) => {
            if input1.is_empty() {
                Ok(((input1, state1), n))
            } else {
                Err(ParseError::NotWellFormed(input1.to_string()))
            }
        }
    }
}

fn document<N: Node>() -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, N), ParseError> {
    move |input| match separated_list0(map(many1(tag("\n")), |_| ()), block())(input) {
        Ok(((input1, state1), v)) => {
            let mut doc = state1.doc().unwrap();
            let mut top = doc
                .new_element(state1.get_qualified_name(
                    None,
                    None,
                    state1.get_value(&"article".to_string()),
                ))
                .expect("unable to create element");
            v.iter()
                .for_each(|c| top.push(c.clone()).expect("unable to add node"));
            doc.push(top.clone()).expect("unable to add node");
            Ok(((input1, state1.clone()), top))
        }
        Err(err) => Err(err),
    }
}

fn block<N: Node>() -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, N), ParseError> {
    alt2(heading(), para())
}

fn heading<N: Node>() -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, N), ParseError> {
    move |input| match tuple3(many1(tag("#")), whitespace0(), phrases())(input) {
        Ok(((input1, state1), (h, _, v))) => {
            let hd_name = format!("heading{}", h.len());
            let mut h = state1
                .doc()
                .unwrap()
                .new_element(state1.get_qualified_name(None, None, state1.get_value(&hd_name)))
                .expect("unable to create element");
            v.iter()
                .for_each(|c| h.push(c.clone()).expect("unable to add node"));
            Ok(((input1, state1), h))
        }
        Err(err) => Err(err),
    }
}

fn strong<N: Node>() -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, N), ParseError> {
    move |input| match tuple3(
        tag("**"),
        map(many0(none_of("*")), |v| v.iter().collect::<String>()),
        tag("**"),
    )(input)
    {
        Ok(((input1, state1), (_, c, _))) => {
            let mut s = state1
                .doc()
                .unwrap()
                .new_element(state1.get_qualified_name(
                    None,
                    None,
                    state1.get_value(&"emph".to_string()),
                ))
                .expect("unable to create element");
            let att = state1
                .doc()
                .unwrap()
                .new_attribute(
                    state1.get_qualified_name(None, None, state1.get_value(&"role".to_string())),
                    state1.get_value(&"strong".to_string()),
                )
                .expect("unable to create attribute");
            s.add_attribute(att).expect("unable to add attribute");
            let content = state1
                .doc()
                .unwrap()
                .new_text(Rc::new(Value::from(c)))
                .expect("unable to create text node");
            s.push(content).expect("unable to add node");
            Ok(((input1, state1), s))
        }
        Err(err) => Err(err),
    }
}
fn underline<N: Node>() -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, N), ParseError> {
    move |input| match tuple3(
        tag("__"),
        map(many1(none_of("_")), |v| v.iter().collect::<String>()),
        tag("__"),
    )(input)
    {
        Ok(((input1, state1), (_, c, _))) => {
            let mut s = state1
                .doc()
                .unwrap()
                .new_element(state1.get_qualified_name(
                    None,
                    None,
                    state1.get_value(&"emph".to_string()),
                ))
                .expect("unable to create element");
            let att = state1
                .doc()
                .unwrap()
                .new_attribute(
                    state1.get_qualified_name(None, None, state1.get_value(&"role".to_string())),
                    state1.get_value(&"underline".to_string()),
                )
                .expect("unable to create attribute");
            s.add_attribute(att).expect("unable to add attribute");
            let content = state1
                .doc()
                .unwrap()
                .new_text(Rc::new(Value::from(c)))
                .expect("unable to create text node");
            s.push(content).expect("unable to add text node");
            Ok(((input1, state1), s))
        }
        Err(err) => Err(err),
    }
}
fn emphasis<N: Node>() -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, N), ParseError> {
    move |input| match tuple3(
        tag("//"),
        map(many1(none_of("/")), |v| v.iter().collect::<String>()),
        tag("//"),
    )(input)
    {
        Ok(((input1, state1), (_, c, _))) => {
            let mut s = state1
                .doc()
                .unwrap()
                .new_element(state1.get_qualified_name(
                    None,
                    None,
                    state1.get_value(&"emph".to_string()),
                ))
                .expect("unable to create element");
            let content = state1
                .doc()
                .unwrap()
                .new_text(Rc::new(Value::from(c)))
                .expect("unable to create text node");
            s.push(content).expect("unable to add text node");
            Ok(((input1, state1), s))
        }
        Err(err) => Err(err),
    }
}
fn phrases<N: Node>() -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, Vec<N>), ParseError> {
    many1(alt4(text_content(), strong(), emphasis(), underline()))
}

fn text_content<N: Node>() -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, N), ParseError> {
    move |input| match map(many1(none_of("*/_\n")), |v| v.iter().collect::<String>())(input) {
        Ok(((input1, state1), t)) => {
            let p = state1
                .doc()
                .unwrap()
                .new_text(Rc::new(Value::from(t)))
                .expect("unable to create text node");
            Ok(((input1, state1), p))
        }
        Err(err) => Err(err),
    }
}

fn para<N: Node>() -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, N), ParseError> {
    move |input| match phrases()(input) {
        Ok(((input1, state1), v)) => {
            let mut p = state1
                .doc()
                .unwrap()
                .new_element(state1.get_qualified_name(
                    None,
                    None,
                    state1.get_value(&"para".to_string()),
                ))
                .expect("unable to create element");
            v.iter()
                .for_each(|c| p.push(c.clone()).expect("unable to add node"));
            Ok(((input1, state1), p))
        }
        Err(err) => Err(err),
    }
}

#[allow(dead_code)]
fn eol<N: Node>() -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, ()), ParseError> {
    map(many1(tag("\n")), |_| ())
}

// This is copied from xrust::parser::xpath::support
// TODO: make it a public function exported by xrust
fn none_of<N: Node>(
    s: &str,
) -> impl Fn(ParseInput<N>) -> Result<(ParseInput<N>, char), ParseError> + '_ {
    move |(input, state)| {
        if input.is_empty() {
            Err(ParseError::Combinator)
        } else {
            let mut ch_it = input.char_indices();
            let (_, a) = ch_it.next().unwrap();
            match s.find(|b| a == b) {
                Some(_) => Err(ParseError::Combinator),
                None => {
                    if let Some((j, _)) = ch_it.next() {
                        Ok(((&input[j..], state), a))
                    } else {
                        Ok((("", state), a))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_1() {
        let d = parse(
            "# Heading
A paragraph
## Level 2
Some **strong** text
# Level 1
Some __underlined__ text
## Another Level 2
//Emphasised// text",
        )
        .expect("unable to parse MarkDown");
        assert_eq!(
            d.to_xml(),
            "<article><heading1>Heading</heading1><para>A paragraph</para><heading2>Level 2</heading2><para>Some <emph role='strong'>strong</emph> text</para><heading1>Level 1</heading1><para>Some <emph role='underline'>underlined</emph> text</para><heading2>Another Level 2</heading2><para><emph>Emphasised</emph> text</para></article>"
        )
    }

    #[test]
    fn headings_1() {
        let d = parse(
            "# Level 1 Heading
## Level 2 Heading
### Level 3 Heading
#### Level 4 Heading
",
        )
        .expect("unable to parse MarkDown");
        assert_eq!(
            d.to_xml(),
            "<article><heading1>Level 1 Heading</heading1><heading2>Level 2 Heading</heading2><heading3>Level 3 Heading</heading3><heading4>Level 4 Heading</heading4></article>"
        )
    }
}
