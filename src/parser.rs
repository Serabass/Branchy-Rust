use crate::ast::{BinOp, CallBlock, CharBlockCount, Event, EventMatcher, FunctionDef, Literal, Node, Program, SourceError, Span};
use crate::lexer::{tokenize_with_offsets, Token};

fn build_line_index(source: &str) -> Vec<usize> {
    let mut out = vec![0];
    for (i, c) in source.char_indices() {
        if c == '\n' {
            out.push(i + 1);
        }
    }
    out
}

fn offset_to_span(line_index: &[usize], start: usize, end: usize) -> Span {
    let start_line = (0..line_index.len()).rposition(|i| line_index[i] <= start).unwrap_or(0);
    let end_line = (0..line_index.len()).rposition(|i| line_index[i] <= end.saturating_sub(1)).unwrap_or(0);
    Span {
        start_line: (start_line + 1) as u32,
        start_column: (start - line_index[start_line] + 1) as u32,
        end_line: (end_line + 1) as u32,
        end_column: (end - line_index[end_line] + 1) as u32,
    }
}

struct SpanStream {
    tokens: Vec<(Token, usize, usize)>,
    line_index: Vec<usize>,
    index: usize,
    first: Option<Span>,
    last: Option<Span>,
}

impl SpanStream {
    fn new(tokens: Vec<(Token, usize, usize)>, source: &str) -> Self {
        SpanStream {
            line_index: build_line_index(source),
            tokens,
            index: 0,
            first: None,
            last: None,
        }
    }
    fn start_span(&mut self) {
        self.first = None;
    }
    fn get_span(&self) -> Option<Span> {
        match (self.first, self.last) {
            (Some(s), Some(e)) => Some(Span {
                start_line: s.start_line,
                start_column: s.start_column,
                end_line: e.end_line,
                end_column: e.end_column,
            }),
            _ => None,
        }
    }
    fn next(&mut self) -> Option<Token> {
        let (tok, start, end) = self.tokens.get(self.index)?.clone();
        self.index += 1;
        let span = offset_to_span(&self.line_index, start, end);
        if self.first.is_none() {
            self.first = Some(span);
        }
        self.last = Some(span);
        Some(tok)
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index).map(|(t, _, _)| t)
    }
    /// Span of the last token consumed by next().
    fn current_span(&self) -> Option<Span> {
        self.last
    }
    /// Span of the next token (if any).
    fn peek_span(&self) -> Option<Span> {
        self.tokens.get(self.index).map(|(_, start, end)| offset_to_span(&self.line_index, *start, *end))
    }
}

type TokenIter = SpanStream;

fn err_span(it: &TokenIter, message: impl Into<String>) -> SourceError {
    SourceError {
        message: message.into(),
        span: it.current_span().or_else(|| it.peek_span()),
    }
}

pub fn parse_program(input: &str) -> Result<Program, SourceError> {
    let tokens = tokenize_with_offsets(input)?;
    let mut it = SpanStream::new(tokens, input);
    let mut includes = Vec::new();
    while matches!(it.peek(), Some(Token::Include)) {
        it.next();
        let path = match it.next() {
            Some(Token::Str(s)) => s,
            _ => return Err(err_span(&it, "expected string path after include")),
        };
        includes.push(path);
        skip_semicolon(&mut it);
    }
    let mut functions = Vec::new();
    let mut events = Vec::new();
    loop {
        if let Some(f) = parse_function_def(&mut it)? {
            functions.push(f);
        } else if let Some(e) = parse_event_def(&mut it)? {
            events.push(e);
        } else {
            break;
        }
    }
    let mut main_branches = Vec::new();
    while matches!(it.peek(), Some(Token::LBrack)) {
        main_branches.push(parse_branch(&mut it)?);
        skip_semicolon(&mut it);
    }
    let main = match main_branches.len() {
        0 => return Err(err_span(&it, "expected at least one main branch")),
        1 => main_branches.into_iter().next().unwrap(),
        _ => Node::Branch {
            children: main_branches,
            span: None,
        },
    };
    if it.next().is_some() {
        return Err(err_span(&it, "unexpected tokens after main branch"));
    }
    Ok(Program {
        includes,
        functions,
        events,
        main,
    })
}

fn parse_event_def(it: &mut TokenIter) -> Result<Option<Event>, SourceError> {
    let (matcher, body) = match it.peek() {
        Some(Token::At) => {
            it.next();
            let name = expect_ident(it)?;
            expect(it, Token::Equals)?;
            let body = parse_branch(it)?;
            (EventMatcher::ByName(name), body)
        }
        Some(Token::Str(s)) => {
            let s = s.clone();
            it.next();
            expect(it, Token::Equals)?;
            let body = parse_branch(it)?;
            (EventMatcher::ByStr(s), body)
        }
        Some(Token::Tilde) => {
            it.next();
            let pattern = match it.next() {
                Some(Token::Str(s)) => s,
                other => return Err(err_span(it, format!("expected regex string after ~, got {:?}", other))),
            };
            expect(it, Token::Equals)?;
            let body = parse_branch(it)?;
            (EventMatcher::ByRegex(pattern), body)
        }
        _ => return Ok(None),
    };
    skip_semicolon(it);
    Ok(Some(Event { matcher, body }))
}

fn parse_function_def(it: &mut TokenIter) -> Result<Option<FunctionDef>, SourceError> {
    match it.peek() {
        Some(Token::Bang) => {}
        _ => return Ok(None),
    }
    it.next();
    let name = expect_ident(it)?;
    expect(it, Token::LParen)?;
    let mut params = Vec::new();
    loop {
        if !matches!(it.peek(), Some(Token::Param(_))) {
            break;
        }
        if let Some(Token::Param(p)) = it.next() {
            params.push(p);
        }
        if !matches!(it.peek(), Some(Token::Comma)) {
            break;
        }
        it.next();
    }
    expect(it, Token::RParen)?;
    expect(it, Token::Equals)?;
    let body = parse_value(it)?;
    skip_semicolon(it);
    Ok(Some(FunctionDef { name, params, body }))
}

fn parse_branch(it: &mut TokenIter) -> Result<Node, SourceError> {
    it.start_span();
    expect(it, Token::LBrack)?;
    let mut elements = Vec::new();
    loop {
        if matches!(it.peek(), Some(Token::RBrack)) {
            it.next();
            return Ok(Node::Branch {
                children: elements,
                span: it.get_span(),
            });
        }
        elements.push(parse_expression(it)?);
        skip_semicolon(it);
    }
}

fn node_span(n: &Node) -> Option<Span> {
    match n {
        Node::Branch { span, .. }
        | Node::Leaf { span, .. }
        | Node::BinaryOp { span, .. }
        | Node::Call { span, .. }
        | Node::InlineCall { span, .. }
        | Node::FuncCall { span, .. }
        | Node::SpreadParam { span, .. }
        | Node::SpreadInclude { span, .. }
        | Node::CharBlock { span, .. } => *span,
    }
}

fn merge_span(a: Option<Span>, b: Option<Span>) -> Option<Span> {
    match (a, b) {
        (Some(s1), Some(s2)) => {
            let (start_ln, start_col) = if (s1.start_line, s1.start_column) <= (s2.start_line, s2.start_column) {
                (s1.start_line, s1.start_column)
            } else {
                (s2.start_line, s2.start_column)
            };
            let (end_ln, end_col) = if (s1.end_line, s1.end_column) >= (s2.end_line, s2.end_column) {
                (s1.end_line, s1.end_column)
            } else {
                (s2.end_line, s2.end_column)
            };
            Some(Span {
                start_line: start_ln,
                start_column: start_col,
                end_line: end_ln,
                end_column: end_col,
            })
        }
        (s, None) | (None, s) => s,
    }
}

fn parse_expression(it: &mut TokenIter) -> Result<Node, SourceError> {
    let mut left = parse_element(it)?;
    loop {
        let op = match it.peek() {
            Some(Token::Plus) => BinOp::Plus,
            Some(Token::Star) => BinOp::Star,
            _ => break,
        };
        it.next();
        let right = parse_element(it)?;
        let left_span = node_span(&left);
        let right_span = node_span(&right);
        left = Node::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span: merge_span(left_span, right_span),
        };
    }
    Ok(left)
}

fn parse_element(it: &mut TokenIter) -> Result<Node, SourceError> {
    it.start_span();
    match it.peek() {
        Some(Token::Spread) => {
            it.next();
            match it.peek() {
                Some(Token::Param(p)) => {
                    let p = p.clone();
                    it.next();
                    Ok(Node::SpreadParam {
                        param: p,
                        span: it.get_span(),
                    })
                }
                Some(Token::Include) => {
                    it.next();
                    let path = match it.next() {
                        Some(Token::Str(s)) => s,
                        _ => return Err(err_span(it, "expected string after ...include")),
                    };
                    Ok(Node::SpreadInclude {
                        path,
                        span: it.get_span(),
                    })
                }
                _ => Err(err_span(it, "expected :param or include after ...")),
            }
        }
        Some(Token::Bang) => parse_func_call(it),
        Some(Token::Param(p)) => {
            let p = p.clone();
            it.next();
            Ok(Node::Leaf {
                lit: Literal::Param(p),
                span: it.get_span(),
            })
        }
        Some(Token::OptionalParam(p)) => {
            let p = p.clone();
            it.next();
            Ok(Node::Leaf {
                lit: Literal::OptionalParam(p),
                span: it.get_span(),
            })
        }
        Some(Token::Ident(_)) => parse_ident_start(it),
        Some(Token::Num(lo)) => {
            let lo = *lo;
            it.next();
            if matches!(it.peek(), Some(Token::RangeSep)) {
                it.next();
                match it.next() {
                    Some(Token::Num(hi)) => Ok(Node::Leaf {
                        lit: Literal::Range(lo, hi),
                        span: it.get_span(),
                    }),
                    _ => Err(err_span(it, "expected number after .. in range (e.g. 1..3)")),
                }
            } else {
                Ok(Node::Leaf {
                    lit: Literal::Num(lo),
                    span: it.get_span(),
                })
            }
        }
        Some(Token::Str(s)) => {
            let s = s.clone();
            it.next();
            Ok(Node::Leaf {
                lit: Literal::Str(s),
                span: it.get_span(),
            })
        }
        Some(Token::LBrack) => parse_branch(it),
        Some(Token::CharBlock(s)) => {
            let content = s.clone();
            it.next();
            let (ranges, count) = parse_char_block_content(&content)
                .map_err(|msg| SourceError { message: msg, span: it.get_span() })?;
            Ok(Node::CharBlock {
                ranges,
                count,
                span: it.get_span(),
            })
        }
        _ => Err(err_span(it, "expected element")),
    }
}

fn parse_char_block_content(s: &str) -> Result<(Vec<(char, char)>, CharBlockCount), String> {
    let (set_part, count_part) = match s.find(':') {
        Some(i) => {
            let set = s[..i].trim();
            let count_str = s[i + 1..].trim();
            (set, Some(count_str))
        }
        None => (s.trim(), None),
    };
    let ranges = parse_char_set(set_part)?;
    if ranges.is_empty() {
        return Err("char block set is empty".into());
    }
    let count = match count_part {
        None => CharBlockCount::One,
        Some(cs) => {
            if let Some(dot) = cs.find("..") {
                let lo: i64 = cs[..dot].trim().parse().map_err(|_| "invalid range start in char block")?;
                let hi: i64 = cs[dot + 2..].trim().parse().map_err(|_| "invalid range end in char block")?;
                CharBlockCount::Range(lo, hi)
            } else {
                let n: i64 = cs.parse().map_err(|_| "invalid count in char block")?;
                CharBlockCount::Fixed(n)
            }
        }
    };
    Ok((ranges, count))
}

fn parse_char_set(s: &str) -> Result<Vec<(char, char)>, String> {
    let mut ranges = Vec::new();
    let mut it = s.chars();
    let mut prev: Option<char> = None;
    while let Some(c) = it.next() {
        if c == '-' {
            let c1 = prev.take().ok_or("char set: '-' at start or after another '-'")?;
            let c2 = it.next().ok_or("char set: expected character after '-'")?;
            if c1 as u32 <= c2 as u32 {
                ranges.push((c1, c2));
            } else {
                ranges.push((c2, c1));
            }
        } else {
            if let Some(p) = prev {
                ranges.push((p, p));
            }
            prev = Some(c);
        }
    }
    if let Some(p) = prev {
        ranges.push((p, p));
    }
    Ok(ranges)
}

fn parse_func_call(it: &mut TokenIter) -> Result<Node, SourceError> {
    it.start_span();
    it.next();
    let name = expect_ident(it)?;
    expect(it, Token::LParen)?;
    let mut args = Vec::new();
    loop {
        if matches!(it.peek(), Some(Token::RParen)) {
            break;
        }
        args.push(parse_expression(it)?);
        if !matches!(it.peek(), Some(Token::Comma)) {
            break;
        }
        it.next();
    }
    expect(it, Token::RParen)?;
    Ok(Node::FuncCall {
        name,
        args,
        span: it.get_span(),
    })
}

fn parse_ident_start(it: &mut TokenIter) -> Result<Node, SourceError> {
    it.start_span();
    let name = expect_ident(it)?;
    match it.peek() {
        Some(Token::LAngle) => {
            it.next();
            let mut options = Vec::new();
            loop {
                options.push(parse_expression(it)?);
                if !matches!(it.peek(), Some(Token::Pipe)) {
                    break;
                }
                it.next();
            }
            expect(it, Token::RAngle)?;
            Ok(Node::InlineCall {
                name,
                options,
                span: it.get_span(),
            })
        }
        Some(Token::Param(_)) | Some(Token::LBrace) => {
            let mut params = Vec::new();
            while matches!(it.peek(), Some(Token::Param(_))) {
                if let Some(Token::Param(p)) = it.next() {
                    params.push(p);
                }
            }
            let block = if matches!(it.peek(), Some(Token::LBrace)) {
                Some(parse_block(it)?)
            } else {
                None
            };
            Ok(Node::Call {
                name,
                params,
                block,
                span: it.get_span(),
            })
        }
        _ => Ok(Node::Leaf {
            lit: Literal::Ident(name),
            span: it.get_span(),
        }),
    }
}

fn parse_block(it: &mut TokenIter) -> Result<CallBlock, SourceError> {
    expect(it, Token::LBrace)?;
    let mut bindings = Vec::new();
    loop {
        skip_semicolon(it);
        if matches!(it.peek(), Some(Token::RBrace)) {
            it.next();
            return Ok(CallBlock { bindings });
        }
        let param = match it.next() {
            Some(Token::Param(p)) => p,
            _ => return Err(err_span(it, "expected :param in block")),
        };
        expect(it, Token::Equals)?;
        let value = parse_value(it)?;
        bindings.push((param, value));
        skip_semicolon(it);
    }
}

fn parse_value(it: &mut TokenIter) -> Result<Node, SourceError> {
    match it.peek() {
        Some(Token::LBrack) => parse_branch(it),
        _ => parse_expression(it),
    }
}

fn expect(it: &mut TokenIter, want: Token) -> Result<(), SourceError> {
    match it.next() {
        Some(got) if got == want => Ok(()),
        got => Err(err_span(it, format!("expected {:?}, got {:?}", want, got))),
    }
}

fn expect_ident(it: &mut TokenIter) -> Result<String, SourceError> {
    match it.next() {
        Some(Token::Ident(s)) => Ok(s),
        other => Err(err_span(it, format!("expected identifier, got {:?}", other))),
    }
}

fn skip_semicolon(it: &mut TokenIter) {
    while matches!(it.peek(), Some(Token::Semicolon)) {
        it.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinOp, Literal, Node};

    #[test]
    fn parse_simple_branch() {
        let p = parse_program("[ a; b; c; ]").unwrap();
        assert!(p.functions.is_empty());
        let Node::Branch { children, .. } = &p.main else { panic!("expected branch") };
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn parse_literals() {
        let p = parse_program("[ hello; 42; ]").unwrap();
        let Node::Branch { children, .. } = &p.main else { panic!("branch") };
        assert!(matches!(&children[0], Node::Leaf { lit: Literal::Ident(s), .. } if s == "hello"));
        assert!(matches!(&children[1], Node::Leaf { lit: Literal::Num(42), .. }));
    }

    #[test]
    fn parse_nested_branch() {
        let p = parse_program("[ [ a; b ]; c; ]").unwrap();
        let Node::Branch { children, .. } = &p.main else { panic!("branch") };
        assert_eq!(children.len(), 2);
        let Node::Branch { children: inner, .. } = &children[0] else { panic!("inner branch") };
        assert_eq!(inner.len(), 2);
    }

    #[test]
    fn parse_function_def() {
        let p = parse_program("!f(:a) = [ x; ]; [ y; ]").unwrap();
        assert_eq!(p.functions.len(), 1);
        assert_eq!(p.functions[0].name, "f");
        assert_eq!(p.functions[0].params, ["a"]);
    }

    #[test]
    fn parse_inline_call() {
        let p = parse_program("[ hello <a|b|c> ]").unwrap();
        let Node::Branch { children, .. } = &p.main else { panic!("branch") };
        let Node::InlineCall { name, options, .. } = &children[0] else { panic!("inline") };
        assert_eq!(name, "hello");
        assert_eq!(options.len(), 3);
    }

    #[test]
    fn parse_func_call() {
        let p = parse_program("[ !f(мир); ]").unwrap();
        let Node::Branch { children, .. } = &p.main else { panic!("branch") };
        let Node::FuncCall { name, args, .. } = &children[0] else { panic!("funccall") };
        assert_eq!(name, "f");
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn parse_binary_op_concat() {
        let p = parse_program(r#"[ "a" + "b"; ]"#).unwrap();
        let Node::Branch { children, .. } = &p.main else { panic!("branch") };
        let Node::BinaryOp { op: BinOp::Plus, left, right, .. } = &children[0] else { panic!("binary op") };
        assert!(matches!(left.as_ref(), Node::Leaf { lit: Literal::Str(s), .. } if s == "a"));
        assert!(matches!(right.as_ref(), Node::Leaf { lit: Literal::Str(s), .. } if s == "b"));
    }

    #[test]
    fn parse_binary_op_repeat() {
        let p = parse_program(r#"[ "x" * 2; ]"#).unwrap();
        let Node::Branch { children, .. } = &p.main else { panic!("branch") };
        let Node::BinaryOp { op: BinOp::Star, left, right, .. } = &children[0] else { panic!("binary op") };
        assert!(matches!(left.as_ref(), Node::Leaf { lit: Literal::Str(s), .. } if s == "x"));
        assert!(matches!(right.as_ref(), Node::Leaf { lit: Literal::Num(2), .. }));
    }

    #[test]
    fn parse_include() {
        let p = parse_program(r#"include "lib.branchy"; [ x; ]"#).unwrap();
        assert_eq!(p.includes, ["lib.branchy"]);
        let Node::Branch { children, .. } = &p.main else { panic!("branch") };
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn parse_spread_param() {
        let p = parse_program(r#"[ a; ...:x; b; ]"#).unwrap();
        let Node::Branch { children, .. } = &p.main else { panic!("branch") };
        assert_eq!(children.len(), 3);
        assert!(matches!(&children[0], Node::Leaf { lit: Literal::Ident(s), .. } if s == "a"));
        assert!(matches!(&children[1], Node::SpreadParam { param: s, .. } if s == "x"));
        assert!(matches!(&children[2], Node::Leaf { lit: Literal::Ident(s), .. } if s == "b"));
    }

    #[test]
    fn parse_spread_include() {
        let p = parse_program(r#"[ ...include "mix.branchy"; ]"#).unwrap();
        let Node::Branch { children, .. } = &p.main else { panic!("branch") };
        assert_eq!(children.len(), 1);
        assert!(matches!(&children[0], Node::SpreadInclude { path: s, .. } if s == "mix.branchy"));
    }

    #[test]
    fn parse_two_adjacent_branches() {
        let p = parse_program("[ a; b; ] [ a; b; ];").unwrap();
        let Node::Branch { children, .. } = &p.main else { panic!("expected outer branch") };
        assert_eq!(children.len(), 2);
        let Node::Branch { children: c1, .. } = &children[0] else { panic!("first child branch") };
        let Node::Branch { children: c2, .. } = &children[1] else { panic!("second child branch") };
        assert_eq!(c1.len(), 2);
        assert_eq!(c2.len(), 2);
    }
}
