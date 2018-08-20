use colored::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::io::Write;
use std::str;

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
    map: Color,
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
    map: Color::White,
};

trait EdnFormatter {
    fn write_nil(&mut self) -> io::Result<()>;
    fn write_boolean(&mut self, value: bool) -> io::Result<()>;
    fn write_char(&mut self, value: char) -> io::Result<()>;
    fn write_symbol(&mut self, value: String) -> io::Result<()>;
    fn write_float(&mut self, value: f64) -> io::Result<()>;
    fn write_integer(&mut self, value: i64) -> io::Result<()>;
    fn write_string(&mut self, value: String) -> io::Result<()>;
    fn write_keyword(&mut self, value: String) -> io::Result<()>;
    fn write_list(&mut self, value: Vec<EdnValue>) -> io::Result<()>;
    fn write_vector(&mut self, value: Vec<EdnValue>) -> io::Result<()>;
    fn write_map(&mut self, value: BTreeMap<EdnValue, EdnValue>) -> io::Result<()>;
    fn write_set(&mut self, value: BTreeSet<EdnValue>) -> io::Result<()>;
    fn write_tagged(&mut self, x: String, y: Box<EdnValue>) -> io::Result<()>;

    fn begin_vector(&mut self) -> io::Result<()>;
    fn end_vector(&mut self) -> io::Result<()>;
    fn begin_vector_item(&mut self, first: bool) -> io::Result<()>;
    fn end_vector_item(&mut self) -> io::Result<()>;
    fn begin_list(&mut self) -> io::Result<()>;
    fn end_list(&mut self) -> io::Result<()>;
    fn begin_list_item(&mut self, first: bool) -> io::Result<()>;
    fn end_list_item(&mut self) -> io::Result<()>;
    fn begin_string(&mut self) -> io::Result<()>;
    fn end_string(&mut self) -> io::Result<()>;
    fn begin_map(&mut self) -> io::Result<()>;
    fn end_map(&mut self) -> io::Result<()>;
    fn begin_map_key(&mut self, first: bool) -> io::Result<()>;
    fn end_map_key(&mut self, _first: bool) -> io::Result<()>;
    fn begin_map_value(&mut self) -> io::Result<()>;
    fn end_map_value(&mut self) -> io::Result<()>;
    fn begin_set(&mut self) -> io::Result<()>;
    fn end_set(&mut self) -> io::Result<()>;
    fn begin_set_item(&mut self, first: bool) -> io::Result<()>;
    fn end_set_item(&mut self) -> io::Result<()>;

    fn write_form(&mut self, form: EdnValue) -> io::Result<()> {
        match form {
            EdnValue::Nil => self.write_nil(),
            EdnValue::Boolean(b) => self.write_boolean(b),
            EdnValue::String(s) => self.write_string(s),
            EdnValue::Char(c) => self.write_char(c),
            EdnValue::Symbol(s) => self.write_symbol(s),
            EdnValue::Keyword(k) => self.write_keyword(k),
            EdnValue::Integer(i) => self.write_integer(i),
            EdnValue::Float(f) => self.write_float(f.into()),
            EdnValue::List(l) => self.write_list(l),
            EdnValue::Vector(v) => self.write_vector(v),
            EdnValue::Map(m) => self.write_map(m),
            EdnValue::Set(s) => self.write_set(s),
            EdnValue::Tagged(x, y) => self.write_tagged(x, y),
        }
    }

    fn reset(&mut self) {
        ()
    }

    fn write_forms(&mut self, forms: Vec<EdnValue>) -> io::Result<()> {
        for form in forms {
            try!(self.write_form(form));
            self.reset();
        }

        Ok(())
    }
}

struct CompactEdnFormatter<W> {
    writer: W,
}

impl<W: Write> CompactEdnFormatter<W> {
    fn new(writer: W) -> CompactEdnFormatter<W> {
        CompactEdnFormatter { writer }
    }
}

impl<W: Write> Write for CompactEdnFormatter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writer.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> EdnFormatter for CompactEdnFormatter<W> {
    fn write_nil(&mut self) -> io::Result<()> {
        write!(self, "{}", "nil".color(DEFAULT_THEME.nil))
    }

    fn write_boolean(&mut self, value: bool) -> io::Result<()> {
        let as_str = if value { "true" } else { "false" };
        write!(self, "{}", as_str.color(DEFAULT_THEME.boolean))
    }

    fn write_char(&mut self, value: char) -> io::Result<()> {
        try!(write!(self, "{}", "\\".color(DEFAULT_THEME.char)));
        try!(write!(
            self.writer,
            "{}",
            value.encode_utf8(&mut [0; 4]).color(DEFAULT_THEME.char)
        ));
        Ok(())
    }

    fn write_symbol(&mut self, value: String) -> io::Result<()> {
        write!(self, "{}", value.color(DEFAULT_THEME.symbol))
    }

    fn write_float(&mut self, value: f64) -> io::Result<()> {
        write!(self, "{}", value.to_string().color(DEFAULT_THEME.number))
    }

    fn write_integer(&mut self, value: i64) -> io::Result<()> {
        write!(self, "{}", value.to_string().color(DEFAULT_THEME.number))
    }

    fn write_string(&mut self, value: String) -> io::Result<()> {
        try!(self.begin_string());
        try!(write!(self, "{}", value.color(DEFAULT_THEME.string)));
        self.end_string()
    }

    fn write_keyword(&mut self, value: String) -> io::Result<()> {
        try!(write!(self, "{}", ":".color(DEFAULT_THEME.keyword)));
        try!(write!(self, "{}", value.color(DEFAULT_THEME.keyword)));
        Ok(())
    }

    fn write_vector(&mut self, value: Vec<EdnValue>) -> io::Result<()> {
        try!(self.begin_vector());

        for (idx, item) in value.into_iter().enumerate() {
            try!(self.begin_vector_item(idx == 0));
            try!(self.write_form(item));
            try!(self.end_vector_item());
        }

        try!(self.end_vector());
        Ok(())
    }

    fn write_list(&mut self, value: Vec<EdnValue>) -> io::Result<()> {
        try!(self.begin_list());

        for (idx, item) in value.into_iter().enumerate() {
            try!(self.begin_list_item(idx == 0));
            try!(self.write_form(item));
            try!(self.end_list_item());
        }

        try!(self.end_list());
        Ok(())
    }

    fn begin_vector(&mut self) -> io::Result<()> {
        write!(self, "{}", "[".color(DEFAULT_THEME.vector))
    }

    fn end_vector(&mut self) -> io::Result<()> {
        write!(self, "{}", "]".color(DEFAULT_THEME.vector))
    }

    fn begin_vector_item(&mut self, first: bool) -> io::Result<()> {
        if !first {
            try!(self.writer.write_all(b" "));
        }
        Ok(())
    }

    fn end_vector_item(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn begin_list(&mut self) -> io::Result<()> {
        write!(self, "{}", "(".color(DEFAULT_THEME.list))
    }

    fn end_list(&mut self) -> io::Result<()> {
        write!(self, "{}", ")".color(DEFAULT_THEME.list))
    }

    fn begin_list_item(&mut self, first: bool) -> io::Result<()> {
        if !first {
            try!(self.writer.write_all(b" "));
        }
        Ok(())
    }

    fn end_list_item(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn begin_string(&mut self) -> io::Result<()> {
        self.writer.write_all(b"\"")
    }

    fn end_string(&mut self) -> io::Result<()> {
        self.writer.write_all(b"\"")
    }

    fn begin_map(&mut self) -> io::Result<()> {
        self.writer.write_all(b"{")
    }

    fn end_map(&mut self) -> io::Result<()> {
        self.writer.write_all(b"}")
    }

    fn begin_map_key(&mut self, first: bool) -> io::Result<()> {
        if !first {
            try!(self.writer.write_all(b" "));
        }
        Ok(())
    }

    fn end_map_key(&mut self, _first: bool) -> io::Result<()> {
        Ok(())
    }

    fn begin_map_value(&mut self) -> io::Result<()> {
        try!(self.writer.write_all(b" "));
        Ok(())
    }

    fn end_map_value(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_map(&mut self, value: BTreeMap<EdnValue, EdnValue>) -> io::Result<()> {
        try!(self.begin_map());
        for (idx, (k, v)) in value.into_iter().enumerate() {
            try!(self.begin_map_key(idx == 0));
            try!(self.write_form(k));
            try!(self.end_map_key(idx == 0));

            try!(self.begin_map_value());
            try!(self.write_form(v));
            try!(self.end_map_value());
        }
        try!(self.end_map());
        Ok(())
    }

    fn begin_set(&mut self) -> io::Result<()> {
        try!(self.writer.write_all(b"#{"));
        Ok(())
    }

    fn end_set(&mut self) -> io::Result<()> {
        try!(self.writer.write_all(b"}"));
        Ok(())
    }

    fn begin_set_item(&mut self, first: bool) -> io::Result<()> {
        if !first {
            try!(self.writer.write_all(b" "));
        }
        Ok(())
    }

    fn end_set_item(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_set(&mut self, value: BTreeSet<EdnValue>) -> io::Result<()> {
        try!(self.begin_set());
        for (idx, item) in value.into_iter().enumerate() {
            try!(self.begin_set_item(idx == 0));
            try!(self.write_form(item));
            try!(self.end_set_item());
        }
        try!(self.end_set());
        Ok(())
    }

    fn write_tagged(&mut self, x: String, y: Box<EdnValue>) -> io::Result<()> {
        try!(write!(self, "{}", "#".color(DEFAULT_THEME.tag)));
        try!(write!(self, "{}", x.color(DEFAULT_THEME.tag)));
        try!(write!(self, "{}", " ".color(DEFAULT_THEME.tag)));
        try!(self.write_form(*y));

        Ok(())
    }
}

trait JsonFormatter {
    /*fn write_null(&mut self) -> io::Result<()> {
        write!(self, "{}", "nil".color(DEFAULT_THEME.nil))
    }

    fn write_undefined(&mut self) -> io::Result<()> {
        write!(self, "{}", "nil".color(DEFAULT_THEME.nil))
    }*/
}

crate struct PrettyEdnFormatter<W: Write> {
    current_column: usize,
    offsets: Vec<usize>,
    has_value: bool,
    writer: W,
}

impl<W: Write> PrettyEdnFormatter<W> {
    fn new(writer: W) -> Self {
        PrettyEdnFormatter {
            current_column: 0,
            offsets: vec![],
            has_value: false,
            writer,
        }
    }

    fn write(&mut self, s: ColoredString) -> io::Result<()> {
        for mut c in s.chars() {
            //println!("{:?}", '\u{001b}' as usize);
            //println!("{:?}", c as usize);
            match c {
                '\n' => self.current_column = 0,
                '\t' => {
                    c = ' ';
                    self.current_column += 1
                }
                '\u{001b}' => {}
                _ if c.is_control() => {}
                _ => self.current_column += 1,
            }

            try!(self.writer.write_all(c.encode_utf8(&mut [0; 4]).as_bytes()));
        }

        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    fn indent(&mut self) -> io::Result<()> {
        match self.offsets.last() {
            Some(&n) => {
                for _ in 0..n {
                    try!(self.write(ColoredString::from(" ")));
                }
            }
            None => {}
        };

        Ok(())
    }
}

impl<W: Write> EdnFormatter for PrettyEdnFormatter<W> {
    fn reset(&mut self) {
        self.current_column = 0;
        self.offsets = vec![];
    }

    fn write_nil(&mut self) -> io::Result<()> {
        self.write("nil".color(DEFAULT_THEME.nil))
    }

    fn write_boolean(&mut self, value: bool) -> io::Result<()> {
        let as_str = if value { "true" } else { "false" };
        self.write(as_str.color(DEFAULT_THEME.boolean))
    }

    fn write_char(&mut self, value: char) -> io::Result<()> {
        try!(self.write("\\".color(DEFAULT_THEME.char)));
        try!(self.write(value.encode_utf8(&mut [0; 4]).color(DEFAULT_THEME.char)));
        Ok(())
    }

    fn write_symbol(&mut self, value: String) -> io::Result<()> {
        self.write(value.color(DEFAULT_THEME.symbol))
    }

    fn write_float(&mut self, value: f64) -> io::Result<()> {
        self.write(value.to_string().color(DEFAULT_THEME.number))
    }

    fn write_integer(&mut self, value: i64) -> io::Result<()> {
        self.write(value.to_string().color(DEFAULT_THEME.number))
    }

    fn write_string(&mut self, value: String) -> io::Result<()> {
        try!(self.begin_string());
        try!(self.write(value.color(DEFAULT_THEME.string)));
        self.end_string()
    }

    fn begin_string(&mut self) -> io::Result<()> {
        self.write(ColoredString::from("\""))
    }

    fn end_string(&mut self) -> io::Result<()> {
        self.write(ColoredString::from("\""))
    }

    fn write_keyword(&mut self, value: String) -> io::Result<()> {
        try!(self.write(":".color(DEFAULT_THEME.keyword)));
        try!(self.write(value.color(DEFAULT_THEME.keyword)));
        Ok(())
    }

    fn write_vector(&mut self, value: Vec<EdnValue>) -> io::Result<()> {
        try!(self.begin_vector());

        for (idx, item) in value.into_iter().enumerate() {
            try!(self.begin_vector_item(idx == 0));
            try!(self.write_form(item));
            try!(self.end_vector_item());
        }

        try!(self.end_vector());
        Ok(())
    }

    fn write_list(&mut self, value: Vec<EdnValue>) -> io::Result<()> {
        try!(self.begin_list());

        for (idx, item) in value.into_iter().enumerate() {
            try!(self.begin_list_item(idx == 0));
            try!(self.write_form(item));
            try!(self.end_list_item());
        }

        try!(self.end_list());
        Ok(())
    }

    fn begin_vector(&mut self) -> io::Result<()> {
        try!(self.write(ColoredString::from("[")));

        self.has_value = false;
        self.offsets.push(self.current_column);

        Ok(())
    }

    fn end_vector(&mut self) -> io::Result<()> {
        self.offsets.pop();
        self.write(ColoredString::from("]"))
    }

    fn begin_vector_item(&mut self, first: bool) -> io::Result<()> {
        if !first {
            try!(self.write(ColoredString::from("\n")));
            try!(self.indent());
        }

        Ok(())
    }

    fn end_vector_item(&mut self) -> io::Result<()> {
        self.has_value = true;
        Ok(())
    }

    fn begin_map(&mut self) -> io::Result<()> {
        try!(self.write(ColoredString::from("{")));
        self.offsets.push(self.current_column);
        self.has_value = false;
        Ok(())
    }

    fn end_map(&mut self) -> io::Result<()> {
        self.offsets.pop();
        self.write(ColoredString::from("}"))
    }

    fn begin_map_key(&mut self, first: bool) -> io::Result<()> {
        if !first {
            try!(self.write(ColoredString::from("\n")));
            try!(self.indent());
        }

        Ok(())
    }

    fn begin_list(&mut self) -> io::Result<()> {
        try!(self.write(ColoredString::from("(")));
        self.has_value = false;
        self.offsets.push(self.current_column);
        Ok(())
    }

    fn end_list(&mut self) -> io::Result<()> {
        self.offsets.pop();

        self.write(ColoredString::from(")"))
    }

    fn begin_list_item(&mut self, first: bool) -> io::Result<()> {
        if !first {
            try!(self.write(ColoredString::from("\n")));
            try!(self.indent());
        }

        Ok(())
    }

    fn end_list_item(&mut self) -> io::Result<()> {
        self.has_value = true;
        Ok(())
    }

    fn end_map_key(&mut self, _first: bool) -> io::Result<()> {
        Ok(())
    }

    fn begin_map_value(&mut self) -> io::Result<()> {
        self.write(ColoredString::from(" "))
    }

    fn end_map_value(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_map(&mut self, value: BTreeMap<EdnValue, EdnValue>) -> io::Result<()> {
        try!(self.begin_map());
        for (idx, (k, v)) in value.into_iter().enumerate() {
            try!(self.begin_map_key(idx == 0));
            try!(self.write_form(k));
            try!(self.end_map_key(idx == 0));

            try!(self.begin_map_value());
            try!(self.write_form(v));
            try!(self.end_map_value());
        }
        try!(self.end_map());
        Ok(())
    }

    fn begin_set(&mut self) -> io::Result<()> {
        try!(self.write(ColoredString::from("#{")));
        self.offsets.push(self.current_column);
        Ok(())
    }

    fn end_set(&mut self) -> io::Result<()> {
        self.offsets.pop();
        self.write(ColoredString::from("}"))
    }

    fn begin_set_item(&mut self, first: bool) -> io::Result<()> {
        if !first {
        try!(self.write(ColoredString::from("\n")));
            try!(self.indent());
        }
        Ok(())
    }

    fn end_set_item(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_set(&mut self, value: BTreeSet<EdnValue>) -> io::Result<()> {
        try!(self.begin_set());
        for (idx, item) in value.into_iter().enumerate() {
            try!(self.begin_set_item(idx == 0));
            try!(self.write_form(item));
            try!(self.end_set_item());
        }
        try!(self.end_set());
        Ok(())
    }

    fn write_tagged(&mut self, x: String, y: Box<EdnValue>) -> io::Result<()> {
        try!(self.write("#".color(DEFAULT_THEME.tag)));
        try!(self.write(x.color(DEFAULT_THEME.tag)));
        try!(self.write(" ".color(DEFAULT_THEME.tag)));
        try!(self.write_form(*y));

        Ok(())
    }
}

crate fn format_output(forms: Vec<EdnValue>, opts: &OutputOptions) -> io::Result<()> {
    let writer = match &opts.destination {
        OutputDestination::Stdout => io::stdout(),
        OutputDestination::File(_path) => io::stdout(),
    };

    match (&opts.format, &opts.style) {
        (OutputFormat::EDN, OutputStyle::Compact) => {
            try!(CompactEdnFormatter::new(writer).write_forms(forms))
        }
        (OutputFormat::EDN, OutputStyle::Pretty) => {
            try!(PrettyEdnFormatter::new(writer).write_forms(forms))
        }
        (OutputFormat::JSON, OutputStyle::Compact) => {
            try!(CompactEdnFormatter::new(writer).write_forms(forms))
        }
        (OutputFormat::JSON, OutputStyle::Pretty) => {
            try!(PrettyEdnFormatter::new(writer).write_forms(forms))
        }
    };

    Ok(())
}

/* #[cfg(test)]
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
} */
