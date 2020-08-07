extern crate rstring_builder;
use char_stream::CharStream;
use rstring_builder::StringBuilder;
use std::error::Error;

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
    pub fn as_int(&self) -> Result<i64, Box<dyn Error>> {
        if let Param::Int(a) = self {
            return Ok(a.clone());
        } else {
            return Err("Param is not int!".into());
        }
    }

    /* COULD BE USED IN THE FUTURE
    pub fn as_flt(&self) -> Result<f64, Box<dyn Error>> {
        if let Param::Float(a) = self {
            return Ok(a.clone());
        } else {
            return Err("Param is not float!".into());
        }
    }
    */

    pub fn as_string(&self) -> Result<String, Box<dyn Error>> {
        if let Param::String(a) = self {
            return Ok(a.clone());
        } else {
            return Err("Param is not string!".into());
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

pub fn next(scan : &mut Scanner) -> Result<String, Box<dyn Error>> {
    let current = peek_next(scan);
    update_token(scan);
    return current;
}

pub fn peek_next(scan : &Scanner) -> Result<String, Box<dyn Error>> {
    let chars = &scan.chars;
    if chars.len() == 0 {
        return Err("no next token!".into());
    } else {
        return Ok(chars.into_iter().collect());
    }
}

pub fn next_line(scan : &mut Scanner) -> Result<String, Box<dyn Error>> {
    if !has_next_line(scan) {
        return Err("no next line!".into());
    }
    update_line(scan);
    let chars = &scan.chars;
    return Ok(chars.into_iter().collect());
}

pub fn has_next_line(scan : &mut Scanner) -> bool {
    return (&mut scan.input_stream).peek().is_some();
}

fn update_token(scan : &mut Scanner) {
    scan.chars = vec![];
    scan.whitespace = vec![];
    let mut next : Option<char>;
    loop {
        next = scan.input_stream.next();
        if let Some(nxt) = next {
            if nxt.is_whitespace() {
                scan.whitespace.push(nxt);
                continue;
            } 
        } 
        break;
    }
    loop {
        if let Some(nxt) = next {
            if !nxt.is_whitespace() {                
                scan.chars.push(nxt);
                next = scan.input_stream.next();
                continue;
            }
        }
        break;
    }
}

fn update_line(scan : &mut Scanner) {
    scan.chars = vec![];
    let mut next : Option<char> = scan.input_stream.next();

    while let Some(nxt) = next {
        if nxt == '\n' {
            break;
        }
        scan.chars.push(nxt);
        next = scan.input_stream.next();
    }
}

pub fn is_next_int(scan : &mut Scanner) -> bool {
    match peek_next(scan) {
        Ok(val) => val.parse::<i64>().is_ok(),
        Err(_) => false
    }
}

pub fn is_next_float(scan : &mut Scanner) -> bool {
    match peek_next(scan) {
        Ok(val) => val.parse::<f64>().is_ok(),
        Err(_) => false
    }
}

pub fn get_next_as_int(scan : &mut Scanner) -> Option<i64> {
    if !is_next_int(scan) {
        return None;
    }
    match next(scan) {
        Ok(val) => {
            match val.parse::<i64>() {
                Ok(ok) => {
                    return Some(ok);
                }
                Err(_) => {
                    return None;
                }
            };            
        },
        Err(_) => None
    }
}

pub fn get_next_as_float(scan: &mut Scanner) -> Option<f64> {
    if !is_next_float(scan) {
        return None;
    }
    match next(scan) {
        Ok(val) => {
            match val.parse::<f64>() {
                Ok(ok) => {
                    return Some(ok);
                }
                Err(_) => {
                    return None;
                }
            };            
        },
        Err(_) => None
    }
}

pub fn get_next_string(scan: &mut Scanner) -> Option<String> {
    let mut s : StringBuilder = StringBuilder::new();
    let nxt = peek_next(scan);
    if let Ok(nxt) = nxt {
        if !nxt.starts_with('"') {
            return None;
        }
    } else {
        return None;
    }

    loop {
        let nxt = next(scan);
        if let Ok(nxt) = nxt {
            if nxt.ends_with("\"") {
                s.append(nxt);
                break;
            }
            s.append(nxt);
            s.append(" ");
        } else {
            return None;
        }
    }

    return Some(s.string()[1..(s.string().len()-1)].to_string());
}

pub fn get_params(scan: &mut Scanner) -> Vec<Param> {
    let mut params = vec![];
    while peek_next(scan).is_ok() {
        if let Some(i) = get_next_as_int(scan) {
            params.push(Param::Int(i));
        } else if let Some(f) = get_next_as_float(scan) {
            params.push(Param::Float(f));
        } else if let Ok(s) = next(scan) {
            params.push(Param::String(s));
        } else {
            break;
        }
    }
    return params;
}
