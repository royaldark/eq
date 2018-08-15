use edn::Value;

crate struct TransformOptions {
    crate expression: String,
}

fn value_type_name(value: &Value) -> String {
    match value {
        Value::Nil => "nil".to_owned(),
        Value::Boolean(_b) => "a boolean".to_owned(),
        Value::String(_s) => "a string".to_owned(),
        Value::Char(_c) => "a character".to_owned(),
        Value::Symbol(_s) => "a symbol".to_owned(),
        Value::Keyword(_k) => "a keyword".to_owned(),
        Value::Integer(_i) => "an integer".to_owned(),
        Value::Float(_f) => "a float".to_owned(),
        Value::List(_l) => "a list".to_owned(),
        Value::Vector(_v) => "a vector".to_owned(),
        Value::Map(_m) => "a map".to_owned(),
        Value::Set(_s) => "a set".to_owned(),
        Value::Tagged(_x, _y) => "a tagged".to_owned(),
    }
}

#[derive(Debug)]
crate struct OperationError(String);

type OperationResult = Result<Value, OperationError>;

crate trait Operation {
    fn execute(&self, input: Value) -> OperationResult;
}

#[derive(Debug, PartialEq)]
crate struct IdentityOperation {}

impl Operation for IdentityOperation {
    fn execute(&self, input: Value) -> OperationResult {
        Ok(input)
    }
}

crate struct KeysOperation {}

impl Operation for KeysOperation {
    fn execute(&self, input: Value) -> OperationResult {
        match input {
            Value::Map(m) => Ok(Value::Vector(m.keys().cloned().collect())),
            _ => Err(OperationError(format!(
                "Can not apply 'keys' operation to {}",
                value_type_name(&input)
            ))),
        }
    }
}

crate struct ValuesOperation {}

impl Operation for ValuesOperation {
    fn execute(&self, input: Value) -> OperationResult {
        match input {
            Value::Map(m) => Ok(Value::Vector(m.values().cloned().collect())),
            _ => Err(OperationError(format!(
                "Can not apply 'values' operation to {}",
                value_type_name(&input)
            ))),
        }
    }
}

#[derive(Debug, PartialEq)]
crate struct GetOperation {
    crate key: Value,
}

impl Operation for GetOperation {
    fn execute(&self, input: Value) -> OperationResult {
        match input {
            Value::Map(m) => Ok(m.get(&self.key).unwrap_or(&Value::Nil).clone()),
            _ => Err(OperationError(format!(
                "Can not apply 'get' operation to {}",
                value_type_name(&input)
            ))),
        }
    }
}

crate struct MapOperation {
    op: Box<dyn Operation>,
}

impl MapOperation {
    fn do_map(&self, input: Vec<Value>) -> OperationResult {
        input
            .into_iter()
            .try_fold(vec![], |mut state, x| {
                self.op.execute(x).map(|value| {
                    state.push(value);
                    state
                })
            })
            .map(|values| Value::Vector(values))
    }
}

impl Operation for MapOperation {
    fn execute(&self, input: Value) -> OperationResult {
        match input {
            Value::List(l) => self.do_map(l),
            Value::Vector(v) => self.do_map(v),
            Value::Set(s) => self.do_map(s.into_iter().collect()),
            _ => Err(OperationError(format!(
                "Can not apply 'get' operation to {}",
                value_type_name(&input)
            ))),
        }
    }
}

fn parse_transform(_transform: &'a str) -> Option<Vec<Box<dyn Operation>>> {
    /*let x = super::parse::identity(_transform.as_bytes());

    match x {
        Ok(y) => println!("{:?}", y),
        Err(e) => println!("error: {:?}", e)
    }*/

    Some(vec![Box::new(IdentityOperation {})])

    /*Some(vec![Box::new(MapOperation {
        op: Box::new(GetOperation {
            key: Value::Keyword("abc".to_owned()),
        }), //Box::new(GetOperation { key: Value::Keyword("abc".to_owned())
    })])*/
}

fn transform_form(form: Value, operations: &Vec<Box<dyn Operation>>) -> OperationResult {
    operations.iter().try_fold(form, |acc, op| op.execute(acc))
}

crate fn transform_edn(
    forms: Vec<Value>,
    transform: &TransformOptions,
) -> Result<Vec<Value>, OperationError> {
    let operations = parse_transform(&transform.expression);

    match operations {
        None => Ok(forms),
        Some(ops) => forms.into_iter().try_fold(vec![], |mut acc, form| {
            transform_form(form, &ops).map(|x| {
                acc.push(x);
                acc
            })
        }),
    }
}
