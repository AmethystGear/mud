extern crate rstring_builder;
use char_stream::CharStream;
use rstring_builder::StringBuilder;

pub struct Scanner {
    input_stream : CharStream,
    chars : Vec<char>,
    whitespace : Vec<char>
}
#[derive(Clone)]
pub enum Param {
    Int(i64),
    Float(f64),
    String(String)
}

impl Param {
    pub fn as_int(&self) -> Option<i64> {
        if let Param::Int(a) = self {
            return Some(a.clone());
        } else {
            return None;
        }
    }

    pub fn as_flt(&self) -> Option<f64> {
        if let Param::Float(a) = self {
            return Some(a.clone());
        } else {
            return None;
        }
    }

    pub fn as_string(&self) -> Option<String> {
        if let Param::String(a) = self {
            return Some(a.clone());
        } else {
            return None;
        }
    }
}

pub fn from(input : CharStream) -> Scanner {
    let mut scan =  Scanner {
        input_stream : input,
        chars : Vec::new(),
        whitespace : Vec::new()
    };
    update_token(&mut scan);
    return scan;
}

pub fn next(scan : &mut Scanner) -> Option<String> {
    let current = peek_next(scan);
    update_token(scan);
    return current;
}

pub fn peek_next(scan : &Scanner) -> Option<String>{
    let chars = &scan.chars;
    if chars.len() == 0 {
        return None;
    } else {
        return Some(chars.into_iter().collect());
    }
}

pub fn next_line(scan : &mut Scanner) -> Option<String> {
    if !has_next_line(scan) {
        return None;
    }
    update_line(scan);
    let chars = &scan.chars;
    return Some(chars.into_iter().collect());
}

pub fn has_next_line(scan : &mut Scanner) -> bool {
    return (&mut scan.input_stream).peek().is_some();
}

fn update_token(scan : &mut Scanner) {
    scan.chars = vec![];
    scan.whitespace = vec![];
    let mut next : Option<char> = Some(' ');
    while next.is_some() && next.unwrap().is_whitespace() {
        next = scan.input_stream.next();
        if next.is_some() && next.unwrap().is_whitespace() {
            scan.whitespace.push(next.unwrap());
        }
    }
    while next.is_some() && !next.unwrap().is_whitespace() {
        scan.chars.push(next.unwrap());
        next = scan.input_stream.next();
    }
    if peek_next(scan).is_none() {
        return;
    }
}

fn update_line(scan : &mut Scanner) {
    scan.chars = vec![];
    let mut next : Option<char> = scan.input_stream.next();
    while next.is_some() && next.unwrap() != '\n' {
        scan.chars.push(next.unwrap());
        next = scan.input_stream.next();
    }
}

pub fn is_next_int(scan : &mut Scanner) -> bool {
    match peek_next(scan) {
        Option::Some(val) => val.parse::<i64>().is_ok(),
        Option::None => false
    }
}

pub fn is_next_float(scan : &mut Scanner) -> bool {
    match peek_next(scan) {
        Option::Some(val) => val.parse::<f64>().is_ok(),
        Option::None => false
    }
}

pub fn get_next_as_int(scan : &mut Scanner) -> Option<i64> {
    if !is_next_int(scan) {
        return None;
    }
    match next(scan) {
        Option::Some(val) => Some(val.parse::<i64>().unwrap()),
        Option::None => None
    }
}

pub fn get_next_as_float(scan: &mut Scanner) -> Option<f64> {
    if !is_next_float(scan) {
        return None;
    }
    match next(scan) {
        Option::Some(val) => Some(val.parse::<f64>().unwrap()),
        Option::None => None
    }
}

pub fn get_next_string(scan: &mut Scanner) -> Option<String> {
    let mut s : StringBuilder = StringBuilder::new();
    let nxt = peek_next(scan);
    if nxt.is_none() || !nxt.unwrap().starts_with('"') {
        return None;
    }
    loop {
        let nxt = next(scan);
        if nxt.is_none() {
            return None;
        }
        let nxt = nxt.unwrap();
        if nxt.ends_with("\"") {
            s.append(nxt);
            break;
        }
        s.append(nxt);
        s.append(" ");
    }

    return Some(s.string()[1..(s.string().len()-1)].to_string());
}

pub fn get_params(scan: &mut Scanner) -> Vec<Param> {
    let mut params = vec![];
    while peek_next(scan).is_some() {
        if is_next_int(scan) {
            params.push(Param::Int(get_next_as_int(scan).unwrap()));
        } else if is_next_float(scan) {
            params.push(Param::Float(get_next_as_float(scan).unwrap()));
        } else {
            params.push(Param::String(next(scan).unwrap()));
        }
    }
    return params;
}
