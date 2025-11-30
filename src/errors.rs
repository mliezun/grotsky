use crate::value::Value;

#[derive(Debug, Clone)]
pub struct RuntimeErr {
    pub msg: &'static str,

    // Signaling of events that should be handled by the VM
    pub signal: Option<Value>,
}

const _SINGAL_MSG: &'static str = "";

impl RuntimeErr {
    pub const fn new(msg: &'static str) -> RuntimeErr {
        RuntimeErr {
            msg: msg,
            signal: None,
        }
    }

    pub fn new_signal(v: Value) -> RuntimeErr {
        RuntimeErr {
            msg: _SINGAL_MSG,
            signal: Some(v),
        }
    }
}

pub const ERR_UNDEFINED_VAR: RuntimeErr = RuntimeErr::new("Undefined variable");
pub const ERR_GLOBAL_ALREADY_DEFINED: RuntimeErr = RuntimeErr::new("Global already defined");
pub const ERR_ONLY_NUMBERS: RuntimeErr =
    RuntimeErr::new("The operation is only defined for numbers");
pub const ERR_UNDEFINED_OP: RuntimeErr = RuntimeErr::new("Undefined operation");
pub const ERR_EXPECTED_STEP: RuntimeErr = RuntimeErr::new("Expected step of the slice");
pub const ERR_EXPECTED_KEY: RuntimeErr = RuntimeErr::new("Expected key for accessing dictionary");
pub const ERR_INVALID_ACCESS: RuntimeErr = RuntimeErr::new("The object is not subscriptable");
pub const ERR_ONLY_FUNCTION: RuntimeErr = RuntimeErr::new("Can only call functions");
pub const ERR_INVALID_NUMBER_ARGUMENTS: RuntimeErr = RuntimeErr::new("Invalid number of arguments");
pub const ERR_EXPECTED_COLLECTION: RuntimeErr = RuntimeErr::new("Collection expected");
pub const ERR_EXPECTED_OBJECT: RuntimeErr = RuntimeErr::new("Object expected");
pub const ERR_EXPECTED_IDENTIFIERS_DICT: RuntimeErr =
    RuntimeErr::new("Expected 1 or 2 identifiers for dict");
pub const ERR_CANNOT_UNPACK: RuntimeErr = RuntimeErr::new("Cannot unpack value");
pub const ERR_WRONG_NUMBER_OF_VALUES: RuntimeErr =
    RuntimeErr::new("Wrong number of values to unpack");
pub const ERR_METHOD_NOT_FOUND: RuntimeErr = RuntimeErr::new("Method not found");
pub const ERR_UNDEFINED_PROP: RuntimeErr = RuntimeErr::new("Undefined property");
pub const ERR_READ_ONLY: RuntimeErr =
    RuntimeErr::new("Trying to set a property on a Read-Only object");
pub const ERR_UNDEFINED_OPERATOR: RuntimeErr =
    RuntimeErr::new("Undefined operator for this object");
pub const ERR_EXPECTED_NUMBER: RuntimeErr =
    RuntimeErr::new("A number was expected at this position");
pub const ERR_EXPECTED_CLASS: RuntimeErr = RuntimeErr::new("A class was expected at this position");
pub const ERR_EXPECTED_STRING: RuntimeErr =
    RuntimeErr::new("A string was expected at this position");
pub const ERR_EXPECTED_INDEX: RuntimeErr =
    RuntimeErr::new("An index was expected at this position");
pub const ERR_EXPECTED_FUNCTION: RuntimeErr =
    RuntimeErr::new("A function was expected at this position");
pub const ERR_EXPECTED_SUPERCLASS: RuntimeErr =
    RuntimeErr::new("Keyword 'super' is only valid inside an object");
pub const ERR_EXPECTED_DICT: RuntimeErr =
    RuntimeErr::new("A dictionary was expected at this position");
pub const ERR_EXPECTED_LIST: RuntimeErr = RuntimeErr::new("A list was expected at this position");
pub const ERR_MAX_RECURSION: RuntimeErr = RuntimeErr::new("Max recursion depth exceeded");
pub const ERR_LIST_EMPTY: RuntimeErr = RuntimeErr::new("List is empty");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Value, NumberValue};

    #[test]
    fn test_new_error() {
        let err = RuntimeErr::new("test error");
        assert_eq!(err.msg, "test error");
        assert!(err.signal.is_none());
    }

    #[test]
    fn test_new_signal() {
        let val = Value::Number(NumberValue { n: 1.0 });
        let err = RuntimeErr::new_signal(val);
        assert_eq!(err.msg, "");
        assert!(err.signal.is_some());
    }
}
