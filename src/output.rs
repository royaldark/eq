use colored::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io;

use clap::{_clap_count_exprs, arg_enum};
use edn::Value as EdnValue;
use serde_json::Value as JsonValue;

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

crate struct ColorTheme {
    nil: Color,
    boolean: Color,
    keyword: Color,
    char: Color,
    string: Color,
    number: Color,
    tag: Color,
    symbol: Color,
    vector: Color,
    list: Color,
}

static DEFAULT_THEME: ColorTheme = ColorTheme {
    nil: Color::BrightBlue,
    symbol: Color::Cyan,
    boolean: Color::Magenta,
    char: Color::BrightRed,
    string: Color::Yellow,
    number: Color::Green,
    keyword: Color::Red,
    tag: Color::BrightGreen,
    vector: Color::BrightYellow,
    list: Color::BrightYellow,
};

trait EdnFormatter {
    fn write_nil<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        write!(writer, "{}", "nil".color(DEFAULT_THEME.nil))
    }

    fn write_boolean<W: io::Write>(&mut self, writer: &mut W, value: bool) -> io::Result<()> {
        let as_str = if value { "true" } else { "false" };
        write!(writer, "{}", as_str.color(DEFAULT_THEME.boolean))
    }

    fn write_char<W: io::Write>(&mut self, writer: &mut W, value: char) -> io::Result<()> {
        try!(write!(writer, "{}", "\\".color(DEFAULT_THEME.char)));
        try!(write!(
            writer,
            "{}",
            value.encode_utf8(&mut [0; 4]).color(DEFAULT_THEME.char)
        ));
        Ok(())
    }

    fn write_symbol<W: io::Write>(&mut self, writer: &mut W, value: String) -> io::Result<()> {
        write!(writer, "{}", value.color(DEFAULT_THEME.symbol))
    }

    fn write_float<W: io::Write>(&mut self, writer: &mut W, value: f64) -> io::Result<()> {
        write!(writer, "{}", value.to_string().color(DEFAULT_THEME.number))
    }

    fn write_integer<W: io::Write>(&mut self, writer: &mut W, value: i64) -> io::Result<()> {
        write!(writer, "{}", value.to_string().color(DEFAULT_THEME.number))
    }

    fn write_string<W: io::Write>(&mut self, writer: &mut W, value: String) -> io::Result<()> {
        try!(self.begin_string(writer));
        try!(write!(writer, "{}", value.color(DEFAULT_THEME.string)));
        self.end_string(writer)
    }

    fn write_keyword<W: io::Write>(&mut self, writer: &mut W, value: String) -> io::Result<()> {
        try!(write!(writer, "{}", ":".color(DEFAULT_THEME.keyword)));
        try!(write!(writer, "{}", value.color(DEFAULT_THEME.keyword)));
        Ok(())
    }

    fn write_vector<W: io::Write>(
        &mut self,
        writer: &mut W,
        value: Vec<EdnValue>,
    ) -> io::Result<()> {
        try!(self.begin_vector(writer));

        for (idx, item) in value.into_iter().enumerate() {
            try!(self.begin_vector_item(writer, idx == 0));
            try!(self.write_form(writer, item));
            try!(self.end_vector_item(writer));
        }

        try!(self.end_vector(writer));
        Ok(())
    }

    fn write_list<W: io::Write>(&mut self, writer: &mut W, value: Vec<EdnValue>) -> io::Result<()> {
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
        write!(writer, "{}", "[".color(DEFAULT_THEME.vector))
    }

    fn end_vector<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        write!(writer, "{}", "]".color(DEFAULT_THEME.vector))
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
        write!(writer, "{}", "(".color(DEFAULT_THEME.list))
    }

    fn end_list<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        write!(writer, "{}", ")".color(DEFAULT_THEME.list))
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
        value: BTreeMap<EdnValue, EdnValue>,
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
        value: BTreeSet<EdnValue>,
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
        y: Box<EdnValue>,
    ) -> io::Result<()> {
        try!(write!(writer, "{}", "#".color(DEFAULT_THEME.tag)));
        try!(write!(writer, "{}", x.color(DEFAULT_THEME.tag)));
        try!(write!(writer, "{}", " ".color(DEFAULT_THEME.tag)));
        try!(self.write_form(writer, *y));

        Ok(())
    }

    fn write_form<W: io::Write>(&mut self, writer: &mut W, form: EdnValue) -> io::Result<()> {
        match form {
            EdnValue::Nil => self.write_nil(writer),
            EdnValue::Boolean(b) => self.write_boolean(writer, b),
            EdnValue::String(s) => self.write_string(writer, s),
            EdnValue::Char(c) => self.write_char(writer, c),
            EdnValue::Symbol(s) => self.write_symbol(writer, s),
            EdnValue::Keyword(k) => self.write_keyword(writer, k),
            EdnValue::Integer(i) => self.write_integer(writer, i),
            EdnValue::Float(f) => self.write_float(writer, f.into()),
            EdnValue::List(l) => self.write_list(writer, l),
            EdnValue::Vector(v) => self.write_vector(writer, v),
            EdnValue::Map(m) => self.write_map(writer, m),
            EdnValue::Set(s) => self.write_set(writer, s),
            EdnValue::Tagged(x, y) => self.write_tagged(writer, x, y),
        }
    }
}

trait JsonFormatter {
    fn write_null<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        write!(writer, "{}", "nil".color(DEFAULT_THEME.nil))
    }

    fn write_undefined<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        write!(writer, "{}", "nil".color(DEFAULT_THEME.nil))
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

crate fn format_output(forms: Vec<EdnValue>, opts: &OutputOptions) -> io::Result<()> {
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

#[cfg(test)]
mod format_tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::iter::FromIterator;

    #[test]
    fn test_nil() {
        assert_eq!(format_form(EdnValue::Nil), "nil");
    }

    #[test]
    fn test_bool() {
        assert_eq!(format_form(EdnValue::Boolean(true)), "true");
        assert_eq!(format_form(EdnValue::Boolean(false)), "false");
    }

    #[test]
    fn test_string() {
        assert_eq!(
            format_form(EdnValue::String("I'm Joe".to_owned())),
            "\"I'm Joe\""
        );
        assert_eq!(
            format_form(EdnValue::String("hello world".to_owned())),
            "\"hello world\""
        );
    }

    #[test]
    fn test_char() {
        assert_eq!(format_form(EdnValue::Char('c')), "\\c");
        assert_eq!(format_form(EdnValue::Char('\\')), "\\\\");
    }

    #[test]
    fn test_list() {
        assert_eq!(
            format_form(EdnValue::List(vec![
                EdnValue::Integer(2),
                EdnValue::String("hello there".to_owned()),
                EdnValue::Keyword("ayo".to_owned()),
            ])),
            "(2 \"hello there\" :ayo)"
        );
        assert_eq!(
            format_form(EdnValue::List(vec![
                EdnValue::Symbol("defn".to_owned()),
                EdnValue::Symbol("square".to_owned()),
                EdnValue::Vector(vec![EdnValue::Symbol("x".to_owned())]),
                EdnValue::List(vec![
                    EdnValue::Symbol("*".to_owned()),
                    EdnValue::Symbol("x".to_owned()),
                    EdnValue::Symbol("x".to_owned()),
                ]),
            ])),
            "(defn square [x] (* x x))"
        );
    }

    #[test]
    fn test_vector() {
        assert_eq!(
            format_form(EdnValue::Vector(vec![
                EdnValue::Integer(2),
                EdnValue::String("hello there".to_owned()),
                EdnValue::Keyword("ayo".to_owned()),
            ])),
            "[2 \"hello there\" :ayo]"
        );
        assert_eq!(
            format_form(EdnValue::Vector(vec![
                EdnValue::Symbol("hi".to_owned()),
                EdnValue::Vector(vec![
                    EdnValue::Char('k'),
                    EdnValue::Nil,
                    EdnValue::Vector(vec![]),
                ]),
            ])),
            "[hi [\\k nil []]]"
        );
    }

    #[test]
    fn test_set() {
        assert_eq!(
            format_form(EdnValue::Set(BTreeSet::from_iter(vec![]))),
            "#{}"
        );
        assert_eq!(
            format_form(EdnValue::Set(BTreeSet::from_iter(vec![EdnValue::Integer(
                2,
            )]))),
            "#{2}"
        );
        assert_eq!(
            format_form(EdnValue::Set(BTreeSet::from_iter(vec![EdnValue::Boolean(
                true,
            )]))),
            "#{true}"
        );
    }
}
