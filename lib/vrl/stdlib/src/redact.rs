use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use vrl::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Redact;

impl Function for Redact {
    fn identifier(&self) -> &'static str {
        "redact"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[
            Parameter {
                keyword: "value",
                kind: kind::BYTES | kind::OBJECT | kind::ARRAY,
                required: true,
            },
            Parameter {
                keyword: "filters",
                kind: kind::ARRAY,
                required: false,
            },
        ]
    }

    fn examples(&self) -> &'static [Example] {
        // TODO
        &[]
    }

    fn compile(&self, mut arguments: ArgumentList) -> Compiled {
        let value = arguments.required("value");
        let filters = arguments.required("filters");
        //.optional_enum_list("filters", &Filter::all_str())?
        //.unwrap_or_default()
        //.into_iter()
        //.map(|s| Filter::from_str(&s).expect("validated enum"))
        //.collect::<Vec<_>>();

        let redactor = Redactor::Full;
        //.optional_enum("redactor", &Redactor::all_str())?
        //.map(|s| Redactor::from_str(&s).expect("validated enum"))
        //.unwrap_or_default();

        Ok(Box::new(RedactFn {
            value,
            filters,
            redactor,
        }))
    }
}

//-----------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct RedactFn {
    value: Box<dyn Expression>,
    filters: Box<dyn Expression>,
    redactor: Redactor,
}

fn redact(value: Value, filters: &Vec<Filter>, redactor: &Redactor) -> Value {
    match value {
        Value::Bytes(bytes) => {
            let input = String::from_utf8_lossy(&bytes);
            let output = filters
                .iter()
                .fold(input, |input, filter| filter.redact(input, redactor));
            Value::Bytes(output.into_owned().into())
        }
        Value::Array(values) => {
            let values = values
                .into_iter()
                .map(|value| redact(value, filters, redactor))
                .collect();
            Value::Array(values)
        }
        Value::Object(map) => {
            let map = map
                .into_iter()
                .map(|(key, value)| (key, redact(value, filters, redactor)))
                .collect();
            Value::Object(map)
        }
        _ => value,
    }
}

impl Expression for RedactFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let value = self.value.resolve(ctx)?;

        let filters = self
            .filters
            .resolve(ctx)?
            .try_array()?
            .into_iter()
            .map(|value| value.try_into().map_err(Into::into))
            .collect::<Result<Vec<Filter>>>()?;

        //let mut input = value.try_bytes_utf8_lossy()?.into_owned();

        Ok(redact(value, &filters, &self.redactor))
        //for filter in &filters {
        //match filter {
        //Filter::Pattern => self
        //.patterns
        //.as_deref()
        //.unwrap_or_default()
        //.iter()
        //.try_for_each::<_, Result<()>>(|expr| match expr.resolve(ctx)? {
        //Value::Bytes(bytes) => {
        //let pattern = String::from_utf8_lossy(&bytes);

        //input = input.replace(pattern.as_ref(), self.redactor.pattern());
        //Ok(())
        //}
        //Value::Regex(regex) => {
        //input = regex
        //.replace_all(&input, self.redactor.pattern())
        //.into_owned();
        //Ok(())
        //}
        //v => Err(value::Error::Expected(
        //value::Kind::Bytes | value::Kind::Regex,
        //v.kind(),
        //)
        //.into()),
        //})?,
        //}
        //}

        //Ok(input.into())
    }

    fn type_def(&self, state: &state::Compiler) -> TypeDef {
        self.value.type_def(state).fallible()

        //let mut typedef = self
        //.value
        //.type_def(state)
        //.fallible_unless(Kind::Bytes)
        //.with_constraint(Kind::Bytes);

        //match &self.patterns {
        //Some(patterns) => {
        //for p in patterns {
        //typedef = typedef.merge(
        //p.type_def(state)
        //.fallible_unless(Kind::Regex)
        //.with_constraint(Kind::Bytes),
        //)
        //}
        //}
        //None => (),
        //}

        //typedef
    }
}

//-----------------------------------------------------------------------------

/// The redaction filter to apply to the given value.
#[derive(Debug, Clone)]
enum Filter {
    Pattern(Vec<regex::Regex>),
    CreditCard,
}
impl TryFrom<value::Value> for Filter {
    type Error = &'static str;

    fn try_from(value: value::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let r#type = map
                    .get("type")
                    .ok_or("filters specified as objects must have type paramater")?
                    .try_bytes()?;

                match r#type {
                    b"pattern" => {
                        let patterns = map
                            .get("patterns")
                            .ok_or("pattern filter must have `patterns` specified")?
                            .try_array()?
                            .iter()
                            .map(|pattern| pattern.try_regex())
                            .collect::<Result<_>>()?;

                        Ok(Filter::Pattern(patterns))
                    }
                    b"credit_card" => Ok(Filter::CreditCard),
                    _ => Err("unknown filter name"),
                }
            }
            Value::Bytes(bytes) => match bytes.as_ref() {
                // TODO move into from_str?
                b"pattern" => Err("pattern cannot be used without arguments"),
                b"credit_card" => Ok(Filter::CreditCard),
                _ => Err("unknown filter name"),
            },
            Value::Regex(regex) => Ok(Filter::Pattern(vec![(*regex).clone()])),
            _ => Err("unknown type for filter, must be a regex, filter name, or object"),
        }
    }
}

impl Filter {
    fn redact<'t>(&self, input: Cow<'t, str>, redactor: &Redactor) -> Cow<'t, str> {
        use Filter::*;

        match &self {
            Pattern(patterns) => patterns.iter().fold(input, |input, pattern| {
                // TODO see if we can avoid cloning here via into_owned()
                pattern
                    .replace_all(&input, redactor.pattern())
                    .into_owned()
                    .into()
            }),
            CreditCard => input,
        }
    }
}

//impl Filter {
//fn all_str() -> Vec<&'static str> {
//use Filter::*;

//vec![Pattern]
//.into_iter()
//.map(|p| p.as_str())
//.collect::<Vec<_>>()
//}

//const fn as_str(self) -> &'static str {
//use Filter::*;

//match self {
//Pattern => "pattern",
//}
//}
//}

//impl FromStr for Filter {
//type Err = &'static str;

//fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
//use Filter::*;

//match s {
//"pattern" => Ok(Pattern),
//_ => Err("unknown filter"),
//}
//}
//}

//-----------------------------------------------------------------------------

/// The recipe for redacting the matched filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Redactor {
    Full,
}

impl Redactor {
    fn pattern(&self) -> &str {
        use Redactor::*;

        match self {
            Full => "****",
        }
    }
}

impl Default for Redactor {
    fn default() -> Self {
        Redactor::Full
    }
}

impl FromStr for Redactor {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use Redactor::*;

        match s {
            "full" => Ok(Full),
            _ => Err("unknown redactor"),
        }
    }
}

//#[cfg(test)]
//mod test {
//use super::*;
//use regex::Regex;

//test_type_def![
//string_infallible {
//expr: |_| RedactFn {
//value: lit!("foo").boxed(),
//filters: vec![Filter::Pattern],
//patterns: None,
//redactor: Redactor::Full,
//},
//def: TypeDef {
//kind: value::Kind::Bytes,
//..Default::default()
//},
//}

//non_string_fallible {
//expr: |_| RedactFn {
//value: lit!(27).boxed(),
//filters: vec![Filter::Pattern],
//patterns: None,
//redactor: Redactor::Full,
//},
//def: TypeDef {
//fallible: true,
//kind: value::Kind::Bytes,
//..Default::default()
//},
//}

//valid_pattern_infallible {
//expr: |_| RedactFn {
//value: lit!("1111222233334444").boxed(),
//filters: vec![Filter::Pattern],
//patterns: Some(vec![Literal::from(Regex::new(r"/[0-9]{16}/").unwrap()).into()]),
//redactor: Redactor::Full,
//},
//def: TypeDef {
//kind: value::Kind::Bytes,
//..Default::default()
//},
//}

//invalid_pattern_fallible {
//expr: |_| RedactFn {
//value: lit!("1111222233334444").boxed(),
//filters: vec![Filter::Pattern],
//patterns: Some(vec![lit!("i am a teapot").into()]),
//redactor: Redactor::Full,
//},
//def: TypeDef {
//fallible: true,
//kind: value::Kind::Bytes,
//..Default::default()
//},
//}
//];
//}
