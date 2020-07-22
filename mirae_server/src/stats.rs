extern crate rstring_builder;
extern crate char_stream;
use std::fs::File;
use std::io::Write;
use std::ops::Add;
use rstring_builder::StringBuilder;

use std::collections::HashMap;
use std::collections::HashSet;
use char_stream::CharStream;
use std::error::Error;
use crate::scanner;

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    LongString(StrBuilder),
    List(Vec<Value>),
    Box(Stats)
}

impl Value {
    pub fn as_int(&self) -> Result<i64, Box<dyn Error>> {
        if let Value::Int(a) = self {
            return Ok(a.clone());
        } else {
            return Err("value is not an int!".into());
        }
    }

    pub fn as_flt(&self) ->  Result<f64, Box<dyn Error>> {
        if let Value::Float(a) = self {
            return Ok(a.clone());
        } else {
            return Err("value is not a float!".into());
        }
    }

    pub fn as_string(&self) -> Result<String, Box<dyn Error>> {
        if let Value::String(a) = self {
            return Ok(a.clone());
        } else {
            return Err("value is not a string!".into());
        }
    }

    pub fn as_longstring(&self) -> Result<StringBuilder, Box<dyn Error>> {
        if let Value::LongString(a) = self {
            return Ok(a.clone().sb);
        } else {
            return Err("value is not a longstring!".into());
        }
    }

    pub fn as_vec(&self) -> Result<Vec<Value>, Box<dyn Error>> {
        if let Value::List(a) = self {
            return Ok(a.clone());
        } else {
            return Err("value is not a vector!".into());
        }
    }

    pub fn as_box(&self) -> Result<Stats, Box<dyn Error>> {
        if let Value::Box(a) = self {
            return Ok(a.clone());
        } else {
            return Err("value is not a box!".into());
        }
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Value {
        let ret : Value;
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => ret = Value::Int(a + b),
            (Value::Float(a), Value::Float(b)) => ret = Value::Float(a + b),
            _ => panic!("values must be of same type, and must be ints or floats!")
        }
        return ret;
    }
}

pub struct Stats {
    types : HashMap<String, String>,
    stats : HashMap<String, Value>,
    properties : HashSet<String>
}

impl Stats {
    pub fn new() -> Stats {
        return Stats {
            types: HashMap::new(),
            stats: HashMap::new(),
            properties: HashSet::new()
        }
    }
}

pub struct StrBuilder {
   pub sb: StringBuilder
}

impl StrBuilder {
    pub fn new(insb : StringBuilder) -> Self {
        return StrBuilder {
            sb: insb
        }
    }
}

impl Clone for StrBuilder {
    fn clone(&self) -> Self {
        let s = self.sb.string();
        let mut builder = StringBuilder::new();
        builder.append(s);
        return StrBuilder::new(builder);
    }
}

impl Clone for Stats {
    fn clone(&self) -> Self {
        let mut stats = Stats::new();
        for p in &self.properties {
            stats.properties.insert(p.to_string());
        }
        for key in self.stats.keys() {
            stats.types.insert(key.clone(), self.types.get(key).unwrap().clone());
            stats.stats.insert(key.clone(), self.stats.get(key).unwrap().clone());
        }
        return stats;
    }
}

fn get_next_value(s : &mut scanner::Scanner, var_type : &str) -> Option<Value> {
    if var_type == "int" {
        if !scanner::is_next_int(s) {
            return None;
        }
        return Some(Value::Int(scanner::get_next_as_int(s).unwrap()));
    } else if var_type == "float" {
        if !scanner::is_next_float(s) {
            return None;
        }
        return Some(Value::Float(scanner::get_next_as_float(s).unwrap()));
    } else if var_type == "string" {
        let string : Option<String> = scanner::get_next_string(s);
        if string.is_none() {
            return None;
        }
        return Some(Value::String(string.unwrap()));
    }
    panic!("type does not match Int, Float, or String!");
}

pub fn from (s : &mut scanner::Scanner) -> Stats {
    let mut stats = Stats {
        types: HashMap::new(),
        stats: HashMap::new(),
        properties: HashSet::new()
    };
    while scanner::has_next_line(s) {
        let line = scanner::next_line(s).unwrap();
        let mut line_scan = scanner::from(CharStream::from_string(line));
        if scanner::peek_next(&line_scan).is_none() {
            continue;
        }
        let token : String = scanner::next(&mut line_scan).unwrap();

        // for comments in the stats files
        if token.starts_with('#') {
            continue;
        }
        // the beginning token
        if token == "{" {
            continue;
        }
        // the ending token
        if token == "}" {
            break;
        }

        // properties are handled seperately
        if token == "prop" {
            if scanner::peek_next(&line_scan).is_none() {
                panic!("property doesn't have a name");
            }
            stats.properties.insert(scanner::next(&mut line_scan).unwrap());
            continue;
        }

        let var_type = token.to_lowercase();

        if scanner::peek_next(&s).is_none() {
            panic!("variable doesn't have a name");
        }
        let var_name = scanner::next(&mut line_scan).unwrap();
        let var_value : Value;
        if var_type == "int" || var_type == "float" || var_type == "string" {
            var_value = get_next_value(&mut line_scan, &var_type).unwrap();
        } else if  var_type == "int[]" || var_type == "float[]" || var_type == "string[]" {
            let var_type = &var_type[0..(var_type.len() - 2)];
            let mut val : Vec<Value> = vec![];
            let mut get = get_next_value(&mut line_scan, &var_type);
            while get.is_some() {
                val.push(get.unwrap());
                get = get_next_value(&mut line_scan, &var_type);
            }
            if val.len() == 0 {
                panic!("there's no values in the {} {} list!", var_type, var_name);
            }
            var_value = Value::List(val);
        } else if var_type == "longstring" {
            let mut long_string : StringBuilder = StringBuilder::new();
            let mut long_string_line = scanner::next_line(s);
            while long_string_line.is_some() {
                let uw = long_string_line.unwrap();
                let trim = uw.trim();
                if trim == "}" {
                    break;
                }
                if trim != "{" {
                    long_string.append(uw);
                    long_string.append('\n');
                }
                long_string_line = scanner::next_line(s);
            }
            var_value = Value::LongString(StrBuilder::new(long_string));
        } else if var_type == "box" {
            var_value = Value::Box(from(s));
        } else {
            panic!("unknown type: {}", var_type);
        }

        stats.types.insert(var_name.clone(), var_type);
        stats.stats.insert(var_name, var_value);
    }
    println!("here {:?}", stats.types);
    return stats;
}

pub fn append(a: Stats, b: Stats) -> Stats {
    let mut stats = Stats::new();
    for p in a.properties {
        stats.properties.insert(p);
    }
    for p in b.properties {
        stats.properties.insert(p);
    }
    for key in b.stats.keys() {
        stats.types.insert(key.to_string(), b.types.get(key).unwrap().to_string());
        stats.stats.insert(key.to_string(), b.stats.get(key).unwrap().clone());
    }
    for key in a.stats.keys() {
        stats.types.insert(key.to_string(), a.types.get(key).unwrap().to_string());
        stats.stats.insert(key.to_string(), a.stats.get(key).unwrap().clone());
    }
    return stats;
}

pub fn add(a: Stats, b: Stats) -> Stats {
    let mut stats = Stats::new();
    for key in a.stats.keys() {
        if b.stats.contains_key(key) && b.types.get(key).unwrap() == a.types.get(key).unwrap() {
            stats.types.insert(key.to_string(), a.types.get(key).unwrap().to_string());
            stats.stats.insert(key.to_string(), a.stats.get(key).unwrap().clone() + b.stats.get(key).unwrap().clone());
        } else {
            stats.types.insert(key.to_string(), a.types.get(key).unwrap().to_string());
            stats.stats.insert(key.to_string(), a.stats.get(key).unwrap().clone());
        }
    }
    for key in b.stats.keys() {
        if a.stats.contains_key(key) && a.types.get(key).unwrap() == b.types.get(key).unwrap() {
            stats.types.insert(key.to_string(), b.types.get(key).unwrap().to_string());
            stats.stats.insert(key.to_string(), b.stats.get(key).unwrap().clone() + a.stats.get(key).unwrap().clone());
        } else {
            stats.types.insert(key.to_string(), b.types.get(key).unwrap().to_string());
            stats.stats.insert(key.to_string(), b.stats.get(key).unwrap().clone());
        }
    }
    return stats;
}

fn get_type(v: &Value) -> String {
    match v {
        Value::Int(_) => {
            return "int".to_string();
        }
        Value::Float(_) => {
            return "float".to_string();
        }
        Value::String(_) => {
            return "string".to_string();
        }
        Value::List(l) => {
            let val : &Value = &l[0];
            return format!("{}[]",get_type(val));
        }
        Value::LongString(_) => {
            return "longstring".to_string();
        }
        Value::Box(_) => {
            return "box".to_string();
        }
    }
}

pub fn set(stats: &mut Stats, var_name: &str, v: Value) {
    stats.types.insert(var_name.to_string(), get_type(&v));
    stats.stats.insert(var_name.to_string(), v);
}

pub fn rm(stats: &mut Stats, var_name: &str) {
    if !stats.types.contains_key(&var_name.to_string()) {
        panic!("that variable doesn't exist!");
    }
    stats.types.remove(&var_name.to_string());
    stats.stats.remove(&var_name.to_string());
}

pub fn get<'a>(stats: &'a Stats, var_name: &str) -> Option<&'a Value> {
    return stats.stats.get(&var_name.to_string());
}

pub fn get_or_else<'a>(stats: &'a Stats, var_name : &str, val : &'a Value) -> &'a Value {
    let v = get(stats, var_name);
    if v.is_none() {
        return &val;
    } else {
        return v.unwrap();
    }
}

pub fn has_var(stats: &Stats, var_name: &str) -> bool {
    return stats.stats.contains_key(&var_name.to_string());
}

pub fn has_prop(stats: &Stats, prop_name: &str) -> bool {
    return stats.properties.contains(&prop_name.to_string());
}

pub fn add_ids_to_boxes(stats: &mut Stats, start_id : i64) -> i64 {
    let clone = stats.clone();
    let mut id = start_id;
    for (key, val) in clone.stats {
        if let Value::Box(b) = val {
            let mut new_box = b.clone();
            set(&mut new_box, "id", Value::Int(id));
            set(stats, &key, Value::Box(new_box));
            id += 1;
        }
    }
    return id;
}

pub fn get_var_names(stats: &Stats) -> Vec<String> {
    let mut v = vec![];
    for var in stats.stats.keys() {
        v.push(var.to_string());
    }
    return v;
}

fn get_string_rep(v : &Value) -> String {
    match v {
        Value::Int(i) => {
            return i.to_string();
        }
        Value::Float(f) => {
            return f.to_string();
        }
        Value::String(s) => {
            return format!("\"{}\"", s);
        }
        Value::List(l) => {
            let mut s : StringBuilder = StringBuilder::new();
            for v in l {
                s.append(get_string_rep(v));
                s.append(" ");
            }
            s.delete(s.len() - 1, s.len());
            return s.string();
        }
        Value::LongString(s) => {
            return format!("\n{}\n{}{}","{",s.sb.string(),"}");
        }
        Value::Box(b) => {
            return string(&b);
        }
    }
}

pub fn string(stats : &Stats) -> String {
    let mut s = StringBuilder::new();
    s.append("{\n");
    for key in stats.stats.keys() {
        let var_name = stats.types.get(key).unwrap().to_string();
        s.append(var_name.clone());
        s.append(' ');
        s.append(key.to_string());
        s.append(' ');
        s.append(get_string_rep(stats.stats.get(key).unwrap()));
        s.append('\n');
    }
    s.append("}\n");
    return s.string();
}

pub fn save_to(stats: &Stats, mut file : File) {
    file.write_all(string(stats).as_bytes());
}
