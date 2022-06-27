use venum::venum::Value;

use crate::errors::Result;

pub trait DivideValue {
    fn divide(&self, src: &Option<Value>) -> Result<(Option<Value>, Option<Value>)>;
}

pub trait SplitValue {
    fn split(&self, src: &Option<Value>) -> Result<Vec<Option<Value>>>;
}
