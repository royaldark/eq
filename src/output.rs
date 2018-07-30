use std::collections::{BTreeMap, BTreeSet};
use std::io;

use clap::{_clap_count_exprs, arg_enum};
use edn::Value;

arg_enum! {
    pub enum OutputFormat {
        EDN,
        JSON,
    }
}

arg_enum! {
    pub enum OutputStyle {
        Compact,
        Pretty,
    }
}

crate enum OutputDestination {
    Stdout,
    File(String),
}

crate struct OutputOptions {
    crate format: OutputFormat,
    crate style: OutputStyle,
    crate destination: OutputDestination,
}

trait EdnFormatter {
    fn write_nil<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"nil")
    }

    fn write_boolean<W: io::Write>(&mut self, writer: &mut W, value: bool) -> io::Result<()> {
        writer.write_all(if value { b"true" } else { b"false" })
    }

    fn write_char<W: io::Write>(&mut self, writer: &mut W, value: char) -> io::Result<()> {
        writer.write_all(value.encode_utf8(&mut [0; 4]).as_ref())
    }

    fn write_symbol<W: io::Write>(&mut self, writer: &mut W, value: String) -> io::Result<()> {
        writer.write_all(value.as_ref())
    }

    fn write_float<W: io::Write>(&mut self, writer: &mut W, value: f64) -> io::Result<()> {
        writer.write_all(format!("{}", value).as_ref())
    }

    fn write_integer<W: io::Write>(&mut self, writer: &mut W, value: i64) -> io::Result<()> {
        writer.write_all(format!("{}", value).as_ref())
    }

    fn write_string<W: io::Write>(&mut self, writer: &mut W, value: String) -> io::Result<()> {
        try!(self.begin_string(writer));
        try!(writer.write_all(value.as_ref()));
        self.end_string(writer)
    }

    fn write_keyword<W: io::Write>(&mut self, writer: &mut W, value: String) -> io::Result<()> {
        try!(writer.write_all(b":"));
        writer.write_all(value.as_ref())
    }

    fn write_vector<W: io::Write>(&mut self, writer: &mut W, value: Vec<Value>) -> io::Result<()> {
        try!(self.begin_vector(writer));

        for (idx, item) in value.into_iter().enumerate() {
            try!(self.begin_vector_item(writer, idx == 0));
            try!(self.write_form(writer, item));
            try!(self.end_vector_item(writer));
        }

        try!(self.end_vector(writer));
        Ok(())
    }

    fn write_list<W: io::Write>(&mut self, writer: &mut W, value: Vec<Value>) -> io::Result<()> {
        try!(self.begin_list(writer));

        for (idx, item) in value.into_iter().enumerate() {
            try!(self.begin_list_item(writer, idx == 0));
            try!(self.write_form(writer, item));
            try!(self.end_list_item(writer));
        }

        try!(self.end_list(writer));
        Ok(())
    }

    fn begin_vector<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"[")
    }

    fn end_vector<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"]")
    }

    fn begin_vector_item<W: io::Write>(&mut self, writer: &mut W, first: bool) -> io::Result<()> {
        if !first {
            try!(writer.write_all(b" "));
        }
        Ok(())
    }

    fn end_vector_item<W: io::Write>(&mut self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn begin_list<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"(")
    }

    fn end_list<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b")")
    }

    fn begin_list_item<W: io::Write>(&mut self, writer: &mut W, first: bool) -> io::Result<()> {
        if !first {
            try!(writer.write_all(b" "));
        }
        Ok(())
    }

    fn end_list_item<W: io::Write>(&mut self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn begin_string<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"\"")
    }

    fn end_string<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"\"")
    }

    fn begin_map<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"{")
    }

    fn end_map<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"}")
    }

    fn begin_map_key<W: io::Write>(&mut self, writer: &mut W, first: bool) -> io::Result<()> {
        if !first {
            try!(writer.write_all(b" "));
        }
        Ok(())
    }

    fn end_map_key<W: io::Write>(&mut self, writer: &mut W, _first: bool) -> io::Result<()> {
        Ok(())
    }

    fn begin_map_value<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        try!(writer.write_all(b" "));
        Ok(())
    }

    fn end_map_value<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn write_map<W: io::Write>(
        &mut self,
        writer: &mut W,
        value: BTreeMap<Value, Value>,
    ) -> io::Result<()> {
        try!(self.begin_map(writer));
        for (idx, (k, v)) in value.into_iter().enumerate() {
            try!(self.begin_map_key(writer, idx == 0));
            try!(self.write_form(writer, k));
            try!(self.end_map_key(writer, idx == 0));

            try!(self.begin_map_value(writer));
            try!(self.write_form(writer, v));
            try!(self.end_map_value(writer));
        }
        try!(self.end_map(writer));
        Ok(())
    }

    fn begin_set<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        try!(writer.write_all(b"#{"));
        Ok(())
    }

    fn end_set<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        try!(writer.write_all(b"}"));
        Ok(())
    }

    fn begin_set_item<W: io::Write>(&mut self, writer: &mut W, first: bool) -> io::Result<()> {
        if !first {
            try!(writer.write_all(b" "));
        }
        Ok(())
    }

    fn end_set_item<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn write_set<W: io::Write>(
        &mut self,
        writer: &mut W,
        value: BTreeSet<Value>,
    ) -> io::Result<()> {
        try!(self.begin_set(writer));
        for (idx, item) in value.into_iter().enumerate() {
            try!(self.begin_set_item(writer, idx == 0));
            try!(self.write_form(writer, item));
            try!(self.end_set_item(writer));
        }
        try!(self.end_set(writer));
        Ok(())
    }

    fn write_tagged<W: io::Write>(
        &mut self,
        writer: &mut W,
        x: String,
        y: Box<Value>,
    ) -> io::Result<()> {
        try!(writer.write_all(b"#"));
        try!(writer.write_all(x.as_ref()));
        try!(writer.write_all(b" "));
        try!(self.write_form(writer, *y));

        Ok(())
    }

    fn write_form<W: io::Write>(&mut self, writer: &mut W, form: Value) -> io::Result<()> {
        match form {
            Value::Nil => self.write_nil(writer),
            Value::Boolean(b) => self.write_boolean(writer, b),
            Value::String(s) => self.write_string(writer, s),
            Value::Char(c) => self.write_char(writer, c),
            Value::Symbol(s) => self.write_symbol(writer, s),
            Value::Keyword(k) => self.write_keyword(writer, k),
            Value::Integer(i) => self.write_integer(writer, i),
            Value::Float(f) => self.write_float(writer, f.into()),
            Value::List(l) => self.write_list(writer, l),
            Value::Vector(v) => self.write_vector(writer, v),
            Value::Map(m) => self.write_map(writer, m),
            Value::Set(s) => self.write_set(writer, s),
            Value::Tagged(x, y) => self.write_tagged(writer, x, y),
        }
    }
}

crate struct CompactEdnFormatter {}

impl EdnFormatter for CompactEdnFormatter {}

#[derive(Debug)]
crate struct PrettyEdnFormatter {
    current_indent: usize,
    has_value: bool,
    indent: String,
}

impl PrettyEdnFormatter {
    fn new(indent: String) -> Self {
        PrettyEdnFormatter {
            current_indent: 0,
            has_value: false,
            indent,
        }
    }
}

fn indent<W: io::Write>(writer: &mut W, n: usize, s: &[u8]) -> io::Result<()> {
    for _ in 0..n {
        try!(writer.write_all(s));
    }

    Ok(())
}

impl EdnFormatter for PrettyEdnFormatter {
    fn begin_vector<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.current_indent += 1;
        self.has_value = false;
        writer.write_all(b"[")
    }

    fn end_vector<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.current_indent -= 1;

        if self.has_value {
            try!(writer.write_all(b"\n"));
            try!(indent(writer, self.current_indent, self.indent.as_ref()));
        }

        writer.write_all(b"]")
    }

    fn begin_vector_item<W: io::Write>(&mut self, writer: &mut W, first: bool) -> io::Result<()> {
        if first {
            try!(writer.write_all(b"\n"));
        } else {
            try!(writer.write_all(b"\n,"));
        }

        try!(indent(writer, self.current_indent, self.indent.as_ref()));
        Ok(())
    }

    fn end_vector_item<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.has_value = true;
        Ok(())
    }

    fn begin_list<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.current_indent += 1;
        self.has_value = false;
        writer.write_all(b"(")
    }

    fn end_list<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.current_indent -= 1;

        if self.has_value {
            try!(writer.write_all(b"\n"));
            try!(indent(writer, self.current_indent, self.indent.as_ref()));
        }

        writer.write_all(b")")
    }

    fn begin_list_item<W: io::Write>(&mut self, writer: &mut W, first: bool) -> io::Result<()> {
        if first {
            try!(writer.write_all(b"\n"));
        } else {
            try!(writer.write_all(b"\n,"));
        }

        try!(indent(writer, self.current_indent, self.indent.as_ref()));
        Ok(())
    }

    fn end_list_item<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.has_value = true;
        Ok(())
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::iter::FromIterator;

    #[test]
    fn test_nil() {
        assert_eq!(format_form(Value::Nil), "nil");
    }

    #[test]
    fn test_bool() {
        assert_eq!(format_form(Value::Boolean(true)), "true");
        assert_eq!(format_form(Value::Boolean(false)), "false");
    }

    #[test]
    fn test_string() {
        assert_eq!(
            format_form(Value::String("I'm Joe".to_owned())),
            "\"I'm Joe\""
        );
        assert_eq!(
            format_form(Value::String("hello world".to_owned())),
            "\"hello world\""
        );
    }

    #[test]
    fn test_char() {
        assert_eq!(format_form(Value::Char('c')), "\\c");
        assert_eq!(format_form(Value::Char('\\')), "\\\\");
    }

    #[test]
    fn test_list() {
        assert_eq!(
            format_form(Value::List(vec![
                Value::Integer(2),
                Value::String("hello there".to_owned()),
                Value::Keyword("ayo".to_owned()),
            ])),
            "(2 \"hello there\" :ayo)"
        );
        assert_eq!(
            format_form(Value::List(vec![
                Value::Symbol("defn".to_owned()),
                Value::Symbol("square".to_owned()),
                Value::Vector(vec![Value::Symbol("x".to_owned())]),
                Value::List(vec![
                    Value::Symbol("*".to_owned()),
                    Value::Symbol("x".to_owned()),
                    Value::Symbol("x".to_owned()),
                ]),
            ])),
            "(defn square [x] (* x x))"
        );
    }

    #[test]
    fn test_vector() {
        assert_eq!(
            format_form(Value::Vector(vec![
                Value::Integer(2),
                Value::String("hello there".to_owned()),
                Value::Keyword("ayo".to_owned()),
            ])),
            "[2 \"hello there\" :ayo]"
        );
        assert_eq!(
            format_form(Value::Vector(vec![
                Value::Symbol("hi".to_owned()),
                Value::Vector(vec![Value::Char('k'), Value::Nil, Value::Vector(vec![])]),
            ])),
            "[hi [\\k nil []]]"
        );
    }

    #[test]
    fn test_set() {
        assert_eq!(format_form(Value::Set(BTreeSet::from_iter(vec![]))), "#{}");
        assert_eq!(
            format_form(Value::Set(BTreeSet::from_iter(vec![Value::Integer(2)]))),
            "#{2}"
        );
        assert_eq!(
            format_form(Value::Set(BTreeSet::from_iter(vec![Value::Boolean(true)]))),
            "#{true}"
        );
    }
}

crate fn format_output(forms: Vec<Value>, opts: &OutputOptions) -> io::Result<()> {
    let mut writer = match &opts.destination {
        OutputDestination::Stdout => io::stdout(),
        OutputDestination::File(_path) => io::stdout(),
    };

    for form in forms {
        match (&opts.format, &opts.style) {
            (OutputFormat::EDN, OutputStyle::Compact) => {
                try!((CompactEdnFormatter {}).write_form(&mut writer, form))
            }
            (OutputFormat::EDN, OutputStyle::Pretty) => {
                try!(PrettyEdnFormatter::new("  ".into()).write_form(&mut writer, form))
            }
            (OutputFormat::JSON, OutputStyle::Compact) => {
                try!((CompactEdnFormatter {}).write_form(&mut writer, form))
            }
            (OutputFormat::JSON, OutputStyle::Pretty) => {
                try!(PrettyEdnFormatter::new("  ".into()).write_form(&mut writer, form))
            }
        };
    }

    Ok(())
}
