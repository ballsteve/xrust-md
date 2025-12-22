//! A markdown parser for Xrust

#![allow(dead_code)]

use qualname::{NamespacePrefix, NamespaceUri, NcName, QName};
use std::rc::Rc;
use xrust::item::Node;
use xrust::parser::combinators::alt::{alt2, alt4};
use xrust::parser::combinators::list::separated_list0;
use xrust::parser::combinators::many::{many0, many1};
use xrust::parser::combinators::map::map;
use xrust::parser::combinators::tag::tag;
use xrust::parser::combinators::tuple::tuple3;
use xrust::parser::combinators::whitespace::whitespace0;
use xrust::parser::{ParseError, ParseInput, ParserStateBuilder, StaticState, StaticStateBuilder};
use xrust::trees::smite::RNode;
use xrust::value::Value;
use xrust::xdmerror::{Error, ErrorKind};

pub fn parse(input: &str) -> Result<RNode, Error> {
    let d = RNode::new_document();
    if input.is_empty() {
        return Ok(d);
    }
    let state = ParserStateBuilder::new().doc(d.clone()).build();
    let mut static_state = StaticStateBuilder::new()
        .namespace(|_| Err(ParseError::Notimplemented))
        .build();
    md_expr((input.trim(), state), &mut static_state)
        .map(|_| d)
        .map_err(|e| match e {
            ParseError::Combinator(_s) => Error::new(
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

fn md_expr<'a, N: Node, L>(
    input: ParseInput<'a, N>,
    ss: &mut StaticState<L>,
) -> Result<(ParseInput<'a, N>, N), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    match document()(input, ss) {
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

fn document<'a, N: Node, L>()
-> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, N), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    move |input, ss| match separated_list0(map(many1(tag("\n")), |_| ()), block())(input, ss) {
        Ok(((input1, state1), v)) => {
            let mut doc = state1.doc().unwrap();
            let mut top = doc
                .new_element(QName::from_local_name(NcName::try_from("article").unwrap()))
                .expect("unable to create element");
            v.iter()
                .for_each(|c| top.push(c.clone()).expect("unable to add node"));
            doc.push(top.clone()).expect("unable to add node");
            Ok(((input1, state1.clone()), top))
        }
        Err(err) => Err(err),
    }
}

fn block<'a, N: Node, L>()
-> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, N), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    alt2(heading(), para())
}

fn heading<'a, N: Node, L>()
-> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, N), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    move |input, ss| match tuple3(many1(tag("#")), whitespace0(), phrases())(input, ss) {
        Ok(((input1, state1), (h, _, v))) => {
            let hd_name = format!("heading{}", h.len());
            let mut h = state1
                .doc()
                .unwrap()
                .new_element(QName::from_local_name(
                    NcName::try_from(hd_name.as_str()).unwrap(),
                ))
                .expect("unable to create element");
            v.iter()
                .for_each(|c| h.push(c.clone()).expect("unable to add node"));
            Ok(((input1, state1), h))
        }
        Err(err) => Err(err),
    }
}

fn strong<'a, N: Node, L>()
-> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, N), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    move |input, ss| match tuple3(
        tag("**"),
        map(many0(none_of("*")), |v| v.iter().collect::<String>()),
        tag("**"),
    )(input, ss)
    {
        Ok(((input1, state1), (_, c, _))) => {
            let mut s = state1
                .doc()
                .unwrap()
                .new_element(QName::from_local_name(NcName::try_from("emph").unwrap()))
                .expect("unable to create element");
            let att = state1
                .doc()
                .unwrap()
                .new_attribute(
                    QName::from_local_name(NcName::try_from("role").unwrap()),
                    Rc::new(Value::from("strong")),
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
fn underline<'a, N: Node, L>()
-> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, N), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    move |input, ss| match tuple3(
        tag("__"),
        map(many1(none_of("_")), |v| v.iter().collect::<String>()),
        tag("__"),
    )(input, ss)
    {
        Ok(((input1, state1), (_, c, _))) => {
            let mut s = state1
                .doc()
                .unwrap()
                .new_element(QName::from_local_name(NcName::try_from("emph").unwrap()))
                .expect("unable to create element");
            let att = state1
                .doc()
                .unwrap()
                .new_attribute(
                    QName::from_local_name(NcName::try_from("role").unwrap()),
                    Rc::new(Value::from("underline")),
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
fn emphasis<'a, N: Node, L>()
-> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, N), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    move |input, ss| match tuple3(
        tag("//"),
        map(many1(none_of("/")), |v| v.iter().collect::<String>()),
        tag("//"),
    )(input, ss)
    {
        Ok(((input1, state1), (_, c, _))) => {
            let mut s = state1
                .doc()
                .unwrap()
                .new_element(QName::from_local_name(NcName::try_from("emph").unwrap()))
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
fn phrases<'a, N: Node, L>()
-> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, Vec<N>), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    many1(alt4(text_content(), strong(), emphasis(), underline()))
}

fn text_content<'a, N: Node, L>()
-> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, N), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    move |input, ss| match map(many1(none_of("*/_\n")), |v| v.iter().collect::<String>())(input, ss)
    {
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

fn para<'a, N: Node, L>()
-> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, N), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    move |input, ss| match phrases()(input, ss) {
        Ok(((input1, state1), v)) => {
            let mut p = state1
                .doc()
                .unwrap()
                .new_element(QName::from_local_name(NcName::try_from("para").unwrap()))
                .expect("unable to create element");
            v.iter()
                .for_each(|c| p.push(c.clone()).expect("unable to add node"));
            Ok(((input1, state1), p))
        }
        Err(err) => Err(err),
    }
}

#[allow(dead_code)]
fn eol<'a, N: Node, L>()
-> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, ()), ParseError>
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    map(many1(tag("\n")), |_| ())
}

// This is copied from xrust::parser::xpath::support
// TODO: make it a public function exported by xrust
fn none_of<'a, N: Node, L>(
    s: &str,
) -> impl Fn(ParseInput<'a, N>, &mut StaticState<L>) -> Result<(ParseInput<'a, N>, char), ParseError> + '_
where
    L: FnMut(&NamespacePrefix) -> Result<NamespaceUri, ParseError>,
{
    move |(input, state), _ss| {
        if input.is_empty() {
            Err(ParseError::Combinator(String::from("no input")))
        } else {
            let mut ch_it = input.char_indices();
            let (_, a) = ch_it.next().unwrap();
            match s.find(|b| a == b) {
                Some(_) => Err(ParseError::Combinator(String::from("found char"))),
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
