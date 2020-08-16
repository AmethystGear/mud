extern crate char_stream;
extern crate rstring_builder;

/* COULD BE USED IF WE USE SAVE IN THE FUTURE
use std::fs::File;
use std::io::Write;
*/
use rstring_builder::StringBuilder;

use crate::scanner;
use char_stream::CharStream;
use std::collections::HashMap;
use std::collections::HashSet;
use std::{error::Error, fs::File, io::Write};

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    LongString(StrBuilder),
    List(Vec<Value>),
    Box(Stats),
}

impl Value {
    pub fn as_int(&self) -> Result<i64, Box<dyn Error>> {
        if let Value::Int(a) = self {
            return Ok(a.clone());
        } else {
            return Err("value is not an int!".into());
        }
    }

    pub fn as_flt(&self) -> Result<f64, Box<dyn Error>> {
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
impl Value {
    fn add(self, rhs: Value) -> Result<Value, Box<dyn Error>> {
        let ret;
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => ret = Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => ret = Ok(Value::Float(a + b)),
            _ => panic!("values must be of same type, and must be ints or floats!"),
        }
        return ret;
    }
}
pub struct Stats {
    types: HashMap<String, String>,
    stats: HashMap<String, Value>,
    properties: HashSet<String>,
}

impl Stats {
    pub fn new() -> Stats {
        return Stats {
            types: HashMap::new(),
            stats: HashMap::new(),
            properties: HashSet::new(),
        };
    }
}

pub struct StrBuilder {
    pub sb: StringBuilder,
}

impl StrBuilder {
    pub fn new(insb: StringBuilder) -> Self {
        return StrBuilder { sb: insb };
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
        for (key, value) in &self.stats {
            stats.stats.insert(key.clone(), value.clone());
        }
        for (key, value) in &self.types {
            stats.types.insert(key.clone(), value.clone());
        }
        return stats;
    }
}

fn get_next_value(s: &mut scanner::Scanner, var_type: &str) -> Result<Value, Box<dyn Error>> {
    match var_type {
        "int" => {
            let val = scanner::get_next_as_int(s);
            if let Some(val) = val {
                return Ok(Value::Int(val));
            }
        }
        "float" => {
            let val = scanner::get_next_as_float(s);
            if let Some(val) = val {
                return Ok(Value::Float(val));
            }
        }
        "string" => {
            let val: Option<String> = scanner::get_next_string(s);
            if let Some(val) = val {
                return Ok(Value::String(val));
            }
        }
        _ => return Err(format!("bad var_type {}", var_type).into()),
    };
    return Err(format!("value conversion for {} failed!", var_type).into());
}

pub fn from(s: &mut scanner::Scanner) -> Result<Stats, Box<dyn Error>> {
    let mut stats = Stats {
        types: HashMap::new(),
        stats: HashMap::new(),
        properties: HashSet::new(),
    };
    while let Ok(line) = scanner::next_line(s) {
        let mut line_scan = scanner::from(CharStream::from_string(line));

        let token = scanner::next(&mut line_scan);
        if token.is_err() {
            continue;
        }
        let token = token?;

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
            let next = scanner::next(&mut line_scan)?;
            stats.properties.insert(next);
            continue;
        }

        let var_type = token.to_lowercase();
        let var_name = scanner::next(&mut line_scan)?;

        let var_value: Value;
        if var_type == "int" || var_type == "float" || var_type == "string" {
            var_value = get_next_value(&mut line_scan, &var_type)?;
        } else if var_type == "int[]" || var_type == "float[]" || var_type == "string[]" {
            let var_type = &var_type[0..(var_type.len() - 2)];
            let mut val: Vec<Value> = vec![];
            while scanner::peek_next(&mut line_scan).is_ok() {
                val.push(get_next_value(&mut line_scan, &var_type)?);
            }
            if val.len() == 0 {
                panic!("there's no values in the {} {} list!", var_type, var_name);
            }
            var_value = Value::List(val);
        } else if var_type == "longstring" {
            let mut long_string: StringBuilder = StringBuilder::new();
            while let Ok(long_string_line) = scanner::next_line(s) {
                let trim = long_string_line.trim();
                if trim == "}" {
                    break;
                }
                if trim != "{" {
                    long_string.append(long_string_line);
                    long_string.append('\n');
                }
            }
            var_value = Value::LongString(StrBuilder::new(long_string));
        } else if var_type == "box" {
            var_value = Value::Box(from(s)?);
        } else {
            panic!("unknown type: {}", var_type);
        }

        stats.types.insert(var_name.clone(), var_type);
        stats.stats.insert(var_name, var_value);
    }
    return Ok(stats);
}

pub fn add(a: Stats, b: Stats) -> Result<Stats, Box<dyn Error>> {
    let mut stats = Stats::new();
    let err = "add stats failure";
    for (key, value) in &a.stats {
        if b.stats.contains_key(key)
            && b.types.get(key).ok_or(err)? == a.types.get(key).ok_or(err)?
        {
            stats
                .types
                .insert(key.to_string(), a.types.get(key).ok_or(err)?.to_string());
            stats.stats.insert(
                key.to_string(),
                value.clone().add(b.stats.get(key).ok_or(err)?.clone())?,
            );
        } else {
            stats
                .types
                .insert(key.to_string(), a.types.get(key).ok_or(err)?.to_string());
            stats.stats.insert(key.to_string(), value.clone());
        }
    }
    for (key, value) in &b.stats {
        if a.stats.contains_key(key)
            && a.types.get(key).ok_or(err)? == b.types.get(key).ok_or(err)?
        {
            stats
                .types
                .insert(key.to_string(), b.types.get(key).ok_or(err)?.to_string());
            stats.stats.insert(
                key.to_string(),
                value.clone().add(a.stats.get(key).ok_or(err)?.clone())?,
            );
        } else {
            stats
                .types
                .insert(key.to_string(), b.types.get(key).ok_or(err)?.to_string());
            stats.stats.insert(key.to_string(), value.clone());
        }
    }
    return Ok(stats);
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
            let val: &Value = &l[0];
            return format!("{}[]", get_type(val));
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

pub fn get<'a>(stats: &'a Stats, var_name: &str) -> Result<&'a Value, Box<dyn Error>> {
    let res = stats.stats.get(&var_name.to_string());
    match res {
        Some(v) => {
            return Ok(v);
        }
        None => {
            return Err(format!(
                "Nothing with the var_name {} exists in this stats object",
                var_name
            )
            .into())
        }
    }
}

pub fn get_or_else<'a>(stats: &'a Stats, var_name: &str, val: &'a Value) -> &'a Value {
    let v = get(stats, var_name);
    match v {
        Ok(v) => {
            return v;
        }
        Err(_) => {
            return val;
        }
    }
}

pub fn has_var(stats: &Stats, var_name: &str) -> bool {
    return stats.stats.contains_key(&var_name.to_string());
}

pub fn has_prop(stats: &Stats, prop_name: &str) -> bool {
    return stats.properties.contains(&prop_name.to_string());
}

pub fn add_ids_to_boxes(stats: &mut Stats, start_id: i64) -> i64 {
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

fn get_string_rep(v: &Value) -> Result<String, Box<dyn Error>> {
    match v {
        Value::Int(i) => {
            return Ok(i.to_string());
        }
        Value::Float(f) => {
            return Ok(f.to_string());
        }
        Value::String(s) => {
            return Ok(format!("\"{}\"", s));
        }
        Value::List(l) => {
            let mut s: StringBuilder = StringBuilder::new();
            for v in l {
                s.append(get_string_rep(v)?);
                s.append(" ");
            }
            s.delete(s.len() - 1, s.len());
            return Ok(s.string());
        }
        Value::LongString(s) => {
            return Ok(format!("\n{}\n{}{}", "{", s.sb.string(), "}"));
        }
        Value::Box(b) => {
            return string(&b);
        }
    }
}

pub fn string(stats: &Stats) -> Result<String, Box<dyn Error>> {
    let mut s = StringBuilder::new();
    s.append("{\n");
    for (key, value) in &stats.stats {
        let var_name = stats
            .types
            .get(key)
            .ok_or("types is not consistent with stats")?
            .to_string();
        s.append(var_name.clone());
        s.append(' ');
        s.append(key.to_string());
        s.append(' ');
        s.append(get_string_rep(&value)?);
        s.append('\n');
    }
    for val in &stats.properties {
        s.append("prop ");
        s.append(val.to_string());
        s.append('\n');
    }
    s.append("}\n");
    return Ok(s.string());
}

pub fn save_to(stats: &Stats, file: &mut File) -> Result<(), Box<dyn Error>> {
    file.write_all(string(stats)?.as_bytes())?;
    return Ok(());
}
