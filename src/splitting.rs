use regex::Regex;

use venum::venum::Value;
use venum_tds::cell::DataCell;

use crate::errors::{Result, SplitError, VenumTdsTransRichError};

pub trait DivideValue {
    fn divide(&self, src: &Option<Value>) -> Result<(Option<Value>, Option<Value>)>;
}

pub trait SplitValue {
    fn split(&self, src: &Option<Value>) -> Result<Vec<Option<Value>>>;
}

#[derive(Debug)]
pub struct ValueStringSeparatorCharDivider {
    pub sep_char: char,
    pub split_none: bool,
}

impl DivideValue for ValueStringSeparatorCharDivider {
    fn divide(&self, src: &Option<Value>) -> Result<(Option<Value>, Option<Value>)> {
        if let Some(val) = src {
            match val {
                Value::String(s) => {
                    let splitted: Vec<&str> = s.split(self.sep_char).collect();
                    if splitted.len() != 2 {
                        return Err(VenumTdsTransRichError::Split(SplitError::from(
                            format!(
                                "expected 2 tokens as result of split, but got: {}",
                                splitted.len()
                            ),
                            src.clone(),
                            None,
                        )));
                    }
                    return Ok((
                        Some(Value::from(String::from(splitted[0]))),
                        Some(Value::from(String::from(splitted[1]))),
                    ));
                }
                _ => Err(VenumTdsTransRichError::Split(SplitError::minim(
                    String::from("Not a Value::String. Can't split."),
                ))),
            }
        } else if src.is_none() && self.split_none {
            Ok((None, None))
        } else {
            Err(VenumTdsTransRichError::Split(SplitError::minim(
                String::from("Value is None, but split_none is false"),
            )))
        }
    }
}

#[derive(Debug)]
pub struct ValueStringSeparatorCharSplitter {
    pub sep_char: char,
    pub split_none: bool,
    pub split_none_into_num_clones: Option<usize>,
}

impl SplitValue for ValueStringSeparatorCharSplitter {
    fn split(&self, src: &Option<Value>) -> Result<Vec<Option<Value>>> {
        match src {
            None => match (&self.split_none, &self.split_none_into_num_clones) {
                (true, None) =>  Err(VenumTdsTransRichError::Split(SplitError::minim(String::from(
                    "Value is None, split_none is true, but split_none_into_num_clones is not set. Can't split into undefined number of targets!",
                )))),
                (false, _) => Err(VenumTdsTransRichError::Split(SplitError::minim(String::from(
                    "Value is None but split_none is false. Not allowed to split!",
                )))),
                (true, Some(num_targets)) => {
                    let mut v: Vec<Option<Value>> = Vec::with_capacity(*num_targets);
                    for _ in 1..=*num_targets {
                        v.push(None);
                    }
                    Ok(v)
                }
            },
            Some(val) => {
                match val {
                    Value::String(s) => {
                        match s.is_empty() {
                            true => Err(VenumTdsTransRichError::Split(SplitError::minim(String::from(
                                "Source Value is empty string. Can't split.", // TODO: None!?
                            )))),
                            false => {
                                let splitted: Vec<&str> = s.split(self.sep_char).collect(); // this will never return a length of 0, as it's implemented by rust!
                                match splitted.len() {
                                    1 => Err(VenumTdsTransRichError::Split(SplitError::minim(String::from(
                                        "expected 2 (or more) tokens as result of split, but got: 1",
                                    )))),
                                    _ => Ok(splitted
                                        .into_iter()
                                        .map(|v| Some(Value::from(String::from(v))))
                                        .collect()),
                                }
                            }
                        }                        
                    },
                    _ => Err(VenumTdsTransRichError::Split(SplitError::minim(String::from(
                        "Not a Value::String. Can't split.",
                    )))),
                }
            }
        }
    }
}


#[derive(Debug)]
pub struct ValueStringRegexPairSplitter {
    pub re: Regex,
    pub split_none: bool,
}

impl ValueStringRegexPairSplitter {
    pub fn from(regex_pattern: String, split_none: bool) -> Result<Self> {
        let re = Regex::new(regex_pattern.as_str()).map_err(|e| {
            let mut err_msg = format!("{}", e);
            err_msg.push_str(" (RegexPairSplitter, ERROR_ON_REGEX_COMPILE)");
            VenumTdsTransRichError::Split(SplitError::minim(err_msg))
        })?;
        Ok(ValueStringRegexPairSplitter { re, split_none })
    }
}

impl DivideValue for ValueStringRegexPairSplitter {
    fn divide(&self, src: &Option<Value>) -> Result<(Option<Value>, Option<Value>)> {
        if let Some(val) = src {
            match val {
                Value::String(s) => {
                    let caps = self.re.captures(&s).ok_or(VenumTdsTransRichError::Split(
                        SplitError::from(
                            String::from("No captures, but we need exactly two."),
                            src.clone(),
                            Some(format!("regex: {}", self.re.as_str())),
                        ),
                    ))?;
                    if caps.len() == 3 {
                        let token_match_1 = caps.get(1).unwrap().as_str();
                        let token_match_2 = caps.get(2).unwrap().as_str();
                        Ok((
                            Some(Value::String(String::from(token_match_1))),
                            Some(Value::String(String::from(token_match_2))),
                        ))
                    } else {
                        Err(VenumTdsTransRichError::Split(SplitError::from(
                            format!("{} capture group(s), but we need exactly two.", caps.len()-1),
                            src.clone(),
                            Some(String::from(self.re.as_str())),
                        )))
                    }
                }
                _ => Err(VenumTdsTransRichError::Split(SplitError::minim(
                    String::from("Not a Value::String. Can't split."),
                ))),
            }
        } else if src.is_none() && self.split_none {
            Ok((None, None))
        } else {
            Err(VenumTdsTransRichError::Split(SplitError::minim(
                String::from("Value is None, but split_none is false"),
            )))
        }
    }
}

trait DivideTBy<S, T>
where
    S: DivideValue,
{
    fn divide_by(&self, splitter_impl: &S, dst_left: &mut T, dst_right: &mut T) -> Result<()>;
}

impl<S: DivideValue> DivideTBy<S, DataCell> for DataCell {
    fn divide_by(
        &self,
        splitter_impl: &S,
        dst_left: &mut DataCell,
        dst_right: &mut DataCell,
    ) -> Result<()> {
        let (split_res_left, split_res_right) = splitter_impl.divide(&self.data)?;

        fn converse_to(val: &Value, type_info: &Value) -> Result<Option<Value>> {
            match val {
            // we have the same enum variant in src and dst, we can use/clone it as is
            _ if std::mem::discriminant(val) == std::mem::discriminant(type_info) => {
                Ok(Some(val.clone()))
            }
            // we have a String variant as src type try converting it to the target type
            Value::String(s) => {
                let transf_val = Value::from_string_with_templ(s, type_info)?;
                Ok(transf_val)
            }
            // We can do better, but we don't support arbitrary convertions for now...
            _ => Err(VenumTdsTransRichError::Split(SplitError::from(
                format!("type mismatch. {val:?} cannot be parsed/converted/put into destination of type {type_info:?}"),
                Some(val.clone()),
                None,
            ))),
        }
        }

        match (split_res_left, split_res_right) {
            (Some(ref data_left), Some(ref data_right)) => {
                dst_left.data = converse_to(data_left, &dst_left.type_info)?;
                dst_right.data = converse_to(data_right, &dst_right.type_info)?;
            }
            (Some(ref data_left), None) => {
                dst_left.data = converse_to(data_left, &dst_left.type_info)?
            }
            (None, Some(ref data_right)) => {
                dst_right.data = converse_to(data_right, &dst_right.type_info)?
            }
            (None, None) => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divide_seperator_char() {
        let sep = ValueStringSeparatorCharDivider {
            sep_char: ' ',
            split_none: true,
        };
        let data = Some(Value::from("foo bar".to_string()));
        let split_res = sep.divide(&data);
        assert!(split_res.is_ok());
        let split_vals = split_res.unwrap();
        assert_eq!(Some(Value::from("foo".to_string())), split_vals.0);
        assert_eq!(Some(Value::from("bar".to_string())), split_vals.1);
    }

    #[test]
    fn test_divide_seperator_char_none() {
        let sep = ValueStringSeparatorCharDivider {
            sep_char: ' ',
            split_none: true,
        };
        let data = None;
        let split_res = sep.divide(&data);
        assert!(split_res.is_ok());
        let split_vals = split_res.unwrap();
        assert_eq!(None, split_vals.0);
        assert_eq!(None, split_vals.1);
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"expected 2 tokens as result of split, but got:"
    )]
    fn test_divide_seperator_char_err() {
        let sep = ValueStringSeparatorCharDivider {
            sep_char: ' ',
            split_none: true,
        };
        let data = Some(Value::from("foo bar baz".to_string()));
        sep.divide(&data).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"expected 2 tokens as result of split, but got: 1\", src_val: Some(String(\"foo\")), detail: None })"
    )]
    fn test_divide_seperator_char_err2() {
        let sep = ValueStringSeparatorCharDivider {
            sep_char: ' ',
            split_none: true,
        };
        let data = Some(Value::from("foo".to_string()));
        sep.divide(&data).unwrap();
    }

    #[test]
    fn test_split_seperator_char() {
        let sep = ValueStringSeparatorCharSplitter {
            sep_char: ' ',
            split_none: false,
            split_none_into_num_clones: None
        };
        let data = Some(Value::from("foo bar baz".to_string()));
        let split_res = sep.split(&data);
        assert!(split_res.is_ok());
        let split_vals = split_res.unwrap();
        assert_eq!(&Some(Value::from("foo".to_string())), split_vals.get(0).unwrap());
        assert_eq!(&Some(Value::from("bar".to_string())), split_vals.get(1).unwrap());
        assert_eq!(&Some(Value::from("baz".to_string())), split_vals.get(2).unwrap());
    }

    #[test]
    fn test_split_seperator_char_none() {
        let sep = ValueStringSeparatorCharSplitter {
            sep_char: ' ',
            split_none: true,
            split_none_into_num_clones: Some(3),
        };
        let data = None;
        let split_res = sep.split(&data);
        assert!(split_res.is_ok());
        let split_vals = split_res.unwrap();
        assert_eq!(&None, split_vals.get(0).unwrap());
        assert_eq!(&None, split_vals.get(1).unwrap());
        assert_eq!(&None, split_vals.get(2).unwrap());
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"Value is None, split_none is true, but split_none_into_num_clones is not set. Can't split into undefined number of targets!\", src_val: None, detail: None })"
    )]
    fn test_split_seperator_char_none_err_config() {
        let sep = ValueStringSeparatorCharSplitter {
            sep_char: ' ',
            split_none: true,
            split_none_into_num_clones: None,
        };
        let data = None;
        sep.split(&data).unwrap();
    }

    #[test]
    fn test_split_regex_pair() {
        let sep_res =
            ValueStringRegexPairSplitter::from("(\\d+\\.\\d+).*(\\d+\\.\\d+)".to_string(), true);
        assert!(sep_res.is_ok());
        let sep = sep_res.unwrap();

        let data = Some(Value::from("1.12 2.23".to_string()));
        let split_res = sep.divide(&data);
        assert!(split_res.is_ok());
        let split_vals = split_res.unwrap();
        assert_eq!(Some(Value::from("1.12".to_string())), split_vals.0);
        assert_eq!(Some(Value::from("2.23".to_string())), split_vals.1);
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"No captures, but we need exactly two.\""
    )]
    fn test_split_regex_err_no_captures() {
        let sep_res =
            ValueStringRegexPairSplitter::from("(\\d+\\.\\d+).*(\\d+\\.\\d+).*(\\d+\\.\\d+)".to_string(), true);
        assert!(sep_res.is_ok());
        let sep = sep_res.unwrap();

        let data = Some(Value::from("1.12 2.23".to_string()));
        sep.divide(&data).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"1 capture group(s), but we need exactly two.\""
    )]
    fn test_split_regex_err_too_few_capture_groups() {
        let sep_res =
            ValueStringRegexPairSplitter::from("(\\d+\\.\\d+)".to_string(), true);
        assert!(sep_res.is_ok());
        let sep = sep_res.unwrap();

        let data = Some(Value::from("1.12 2.23".to_string()));
        sep.divide(&data).unwrap();
    }

    #[test]
    fn test_split_regex_pair_illegal_regex() {
        let sep_res = ValueStringRegexPairSplitter::from("FWPUJWDJW/)!(!()?))".to_string(), true);
        assert!(sep_res.is_err());
    }
}
