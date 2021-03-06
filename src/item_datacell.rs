use venum::venum::Value;
use venum_tds::cell::DataCell;

use crate::{
    errors::{Result, SplitError, VenumTdsTransRichError},
    traits::{item::SplitUsing, value::Split},
};

impl<D: Split> SplitUsing<D> for DataCell {
    type ITEM = Self;

    fn split_using(
        &self,
        splitter_impl: &D,
        dst_left: &mut DataCell,
        dst_right: &mut DataCell,
    ) -> Result<()> {
        let (split_res_left, split_res_right) = splitter_impl.split(&self.data)?;

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
                // TODO We can do better, but we don't support arbitrary convertions for now...
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

// TODO: merge

#[cfg(test)]
mod tests {
    use venum::venum::Value;
    use venum_tds::{cell::DataCell, traits::VDataContainerItem};

    use crate::{
        traits::item::SplitUsing,
        value_splitting::{ValueStringRegexPairSplit, ValueStringSeparatorCharSplit},
    };

    #[test]
    fn test_split_datacell_by_char_seperator_divider() {
        let dc1 = DataCell::new(
            Value::string_default(),
            String::from("col1"),
            0,
            Some(Value::from(String::from("true;1.12"))),
        );

        let sp = ValueStringSeparatorCharSplit {
            sep_char: ';',
            split_none: true,
        };

        let mut dc_left =
            DataCell::new_without_data(Value::bool_default(), String::from("is_true"), 1);
        let mut dc_right =
            DataCell::new_without_data(Value::float32_default(), String::from("f32_val"), 2);

        let res = dc1.split_using(&sp, &mut dc_left, &mut dc_right);
        assert!(res.is_ok());

        assert!(dc_left.get_data().is_some());
        assert!(dc_right.get_data().is_some());

        assert_eq!(true, bool::try_from(dc_left.get_data().unwrap()).unwrap());
        assert_eq!(
            1.12f32,
            f32::try_from(dc_right.get_data().unwrap()).unwrap()
        );
    }

    #[test]
    fn test_split_datacell_by_regex_divider() {
        let dc1 = DataCell::new(
            Value::string_default(),
            String::from("col1"),
            0,
            Some(Value::from(String::from("1.12 2.23"))),
        );

        let sp =
            ValueStringRegexPairSplit::from(String::from("(\\d+\\.\\d+).*(\\d+\\.\\d+)"), true)
                .unwrap();

        let mut dc_left =
            DataCell::new_without_data(Value::float32_default(), String::from("f32_val_left"), 1);
        let mut dc_right =
            DataCell::new_without_data(Value::float32_default(), String::from("f32_val_right"), 2);

        let res = dc1.split_using(&sp, &mut dc_left, &mut dc_right);
        assert!(res.is_ok());

        assert!(dc_left.get_data().is_some());
        assert!(dc_right.get_data().is_some());

        assert_eq!(1.12f32, dc_left.get_data().unwrap().try_into().unwrap());
        assert_eq!(2.23f32, dc_right.get_data().unwrap().try_into().unwrap());
    }
}
