use venum::venum::Value;
use venum_tds::traits::{VDataContainer, VDataContainerItem};

use crate::{
    errors::{ContainerOpsErrors, Result, VenumTdsTransRichError},
    traits::{container::TransrichContainerInplace, item::SplitUsing, value::Split},
};

#[derive(Debug, Clone, PartialEq)]
pub struct MutateItemIdx {
    pub from: usize,
    pub to: usize,
}
impl MutateItemIdx {
    pub fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
}
impl<C> TransrichContainerInplace<C> for MutateItemIdx
where
    C: VDataContainer,
{
    fn apply(&self, data_container: &mut C) -> Result<()> {
        let container_entry = data_container.get_by_idx_mut(self.from);
        match container_entry {
            None => Err(VenumTdsTransRichError::ContainerOps(
                ContainerOpsErrors::Generic {
                    msg: String::from("No DataEntry with idx {self.from}. Can't mutate index."),
                },
            )),
            Some(date_entry) => {
                date_entry.set_idx(self.to);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteItemAtIdx(pub usize);
impl<C> TransrichContainerInplace<C> for DeleteItemAtIdx
where
    C: VDataContainer,
{
    fn apply(&self, data_container: &mut C) -> Result<()> {
        match data_container.del_by_idx(self.0) {
            Ok(_) => Ok(()),
            Err(e) => Err(VenumTdsTransRichError::from(e)),
        }
    }
}

pub struct AddItem<T: VDataContainerItem>(pub T);
impl<C, D> TransrichContainerInplace<C> for AddItem<D>
where
    D: VDataContainerItem + Clone + Default,
    C: VDataContainer<ITEM = D>,
{
    fn apply(&self, data_container: &mut C) -> Result<()> {
        data_container.add(self.0.clone());
        Ok(())
    }
}

pub struct SplitItemAtIdx<S: Split> {
    pub idx: usize,
    pub splitter: S,
    pub target_left: (Value, usize, String),
    pub target_right: (Value, usize, String),
    pub delete_source_item: bool,
}

impl<CONT, ENTRY, SPLITIMPL> TransrichContainerInplace<CONT> for SplitItemAtIdx<SPLITIMPL>
where
    SPLITIMPL: Split, // The splitter "implementation" to use, to split an ITEM of type Value. This is the lowest level
    ENTRY: VDataContainerItem + SplitUsing<SPLITIMPL, ITEM = ENTRY> + Default, // Entries (of the container) must be container items that also implement "splitUsing", which relies on a certain split implementation (given above)
    CONT: VDataContainer<ITEM = ENTRY>, // The container where we want to split an item inside, making use of the 'splitUsing' of the entry and in turn the 'split' implementation
{
    fn apply(&self, container: &mut CONT) -> Result<()> {
        let entry = container.get_by_idx_mut(self.idx).ok_or_else(|| {
            VenumTdsTransRichError::ContainerOps(ContainerOpsErrors::SplitItemError {
                idx: self.idx,
                msg: format!("Container does not have an entry at idx: {}", self.idx),
            })
        })?;

        let mut t_left = ENTRY::default();
        t_left.set_type_info(self.target_left.0.clone());
        t_left.set_idx(self.target_left.1);
        t_left.set_name(&self.target_left.2);

        let mut t_right = ENTRY::default();
        t_right.set_type_info(self.target_right.0.clone());
        t_right.set_idx(self.target_right.1);
        t_right.set_name(&self.target_right.2);

        let div_res = entry.split_using(&self.splitter, &mut t_left, &mut t_right);
        if div_res.is_ok() {
            container.add(t_left);
            container.add(t_right);
            if self.delete_source_item {
                container.del_by_idx(self.idx).unwrap();
            }
        }
        div_res
    }
}

#[cfg(test)]
mod tests {
    use venum::venum::Value;
    use venum_tds::{cell::DataCell, row::DataCellRow};

    use crate::value_splitting::{ValueStringRegexPairSplit, ValueStringSeparatorCharSplit};

    use super::*;

    #[test]
    fn test_mutate_idx_of_tds_data_cell() {
        let m = MutateItemIdx::new(0, 1);

        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            Value::bool_default(),
            String::from("col1"),
            0,
        ));

        m.apply(&mut c).unwrap();
        assert_eq!(1, c.0.first().unwrap().idx);
    }

    #[test]
    fn test_delete_from_container() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            Value::bool_default(),
            String::from("col1"),
            0,
        ));
        c.0.push(DataCell::new_without_data(
            Value::bool_default(),
            String::from("col2"),
            1,
        ));

        let container_transricher = DeleteItemAtIdx(0);
        container_transricher.apply(&mut c).unwrap();

        let container_transricher2 = DeleteItemAtIdx(1);
        container_transricher2.apply(&mut c).unwrap();

        assert_eq!(0, c.0.len());
    }

    #[test]
    #[should_panic(expected = "Wrapped(VenumTdsError(DataAccess(IllegalIdxAccess { idx: 0 })))")]
    fn test_delete_from_container_err() {
        let mut c = DataCellRow::new();
        let container_transricher = DeleteItemAtIdx(0);
        container_transricher.apply(&mut c).unwrap();
    }

    #[test]
    fn test_add_to_container() {
        let mut c = DataCellRow::new();
        let container_transricher = AddItem(DataCell::new_without_data(
            Value::bool_default(),
            String::from("col1"),
            0,
        ));
        container_transricher.apply(&mut c).unwrap();

        assert_eq!(1, c.0.len());
    }

    #[test]
    fn test_combined() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            Value::bool_default(),
            String::from("col1"),
            0,
        ));
        c.0.push(DataCell::new_without_data(
            Value::bool_default(),
            String::from("col2"),
            1,
        ));

        let mut transrichers: Vec<Box<dyn TransrichContainerInplace<DataCellRow>>> =
            Vec::with_capacity(3);

        transrichers.push(Box::new(AddItem(DataCell::new_without_data(
            Value::bool_default(),
            String::from("col3"),
            2,
        ))));
        transrichers.push(Box::new(DeleteItemAtIdx(1)));
        transrichers.push(Box::new(DeleteItemAtIdx(2)));
        transrichers.push(Box::new(MutateItemIdx::new(0, 10)));

        transrichers
            .iter_mut()
            .map(|t| t.apply(&mut c))
            .collect::<Result<Vec<()>>>()
            .unwrap();

        assert_eq!(1, c.0.len());
        assert_eq!(10, c.0.first().unwrap().get_idx());
        assert_eq!(String::from("col1"), c.0.first().unwrap().get_name());
    }

    #[test]
    pub fn test_split_container_item_using_value_string_separator_char_divider() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new(
            Value::string_default(),
            String::from("col1"),
            0,
            Some(Value::String(String::from("foo:bar"))),
        ));

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ':',
                split_none: false,
            },
            target_left: (Value::string_default(), 1, String::from("col2")),
            target_right: (Value::string_default(), 2, String::from("Col3")),
            delete_source_item: false,
        };

        div_at.apply(&mut c).unwrap();

        assert_eq!(3, c.0.len());
        assert_eq!(
            &Value::String(String::from("foo")),
            c.get_by_idx(1).unwrap().get_data().unwrap()
        );
        assert_eq!(
            &Value::String(String::from("bar")),
            c.get_by_idx(2).unwrap().get_data().unwrap()
        );
    }

    #[test]
    pub fn test_split_container_item_using_value_string_separator_char_divider_delete_src() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new(
            Value::string_default(),
            String::from("col1"),
            0,
            Some(Value::String(String::from("foo:bar"))),
        ));

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ':',
                split_none: false,
            },
            target_left: (Value::string_default(), 1, String::from("col2")),
            target_right: (Value::string_default(), 2, String::from("Col3")),
            delete_source_item: true,
        };

        div_at.apply(&mut c).unwrap();

        assert_eq!(2, c.0.len());
        assert_eq!(
            &Value::String(String::from("foo")),
            c.get_by_idx(1).unwrap().get_data().unwrap()
        );
        assert_eq!(
            &Value::String(String::from("bar")),
            c.get_by_idx(2).unwrap().get_data().unwrap()
        );
    }

    #[test]
    pub fn test_split_container_item_using_value_string_separator_char_divider_none() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            Value::string_default(),
            String::from("col1"),
            0,
        ));

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ':',
                split_none: true,
            },
            target_left: (Value::string_default(), 1, String::from("col2")),
            target_right: (Value::string_default(), 2, String::from("Col3")),
            delete_source_item: false,
        };

        div_at.apply(&mut c).unwrap();

        assert_eq!(3, c.0.len());
        assert_eq!(None, c.get_by_idx(1).unwrap().get_data());
        assert_eq!(None, c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    pub fn test_split_container_item_using_value_string_separator_char_divider_none_delete_src() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            Value::string_default(),
            String::from("col1"),
            0,
        ));

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ':',
                split_none: true,
            },
            target_left: (Value::string_default(), 1, String::from("col2")),
            target_right: (Value::string_default(), 2, String::from("Col3")),
            delete_source_item: true,
        };

        div_at.apply(&mut c).unwrap();

        assert_eq!(2, c.0.len());
        assert_eq!(None, c.get_by_idx(1).unwrap().get_data());
        assert_eq!(None, c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"Value is None, but split_none is false\", src_val: None, details: None })"
    )]
    pub fn test_split_container_item_using_value_string_separator_char_divider_none_but_split_none_is_false(
    ) {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            Value::string_default(),
            String::from("col1"),
            0,
        ));

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ':',
                split_none: false, // <--- !!!
            },
            target_left: (Value::string_default(), 1, String::from("col2")),
            target_right: (Value::string_default(), 2, String::from("Col3")),
            delete_source_item: false,
        };

        div_at.apply(&mut c).unwrap();
    }

    #[test]
    pub fn test_split_container_item_using_value_string_regex_pair_divider() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new(
            Value::string_default(),
            String::from("col1"),
            0,
            Some(Value::String(String::from("1.12 2.23"))),
        ));

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: ValueStringRegexPairSplit::from(
                "(\\d+\\.\\d+).*(\\d+\\.\\d+)".to_string(),
                true,
            )
            .unwrap(),
            target_left: (Value::float32_default(), 1, String::from("col2")),
            target_right: (Value::float32_default(), 2, String::from("Col3")),
            delete_source_item: false,
        };

        div_at.apply(&mut c).unwrap();

        assert_eq!(3, c.0.len());
        assert_eq!(
            &Value::Float32(1.12_f32),
            c.get_by_idx(1).unwrap().get_data().unwrap()
        );
        assert_eq!(
            &Value::Float32(2.23_f32),
            c.get_by_idx(2).unwrap().get_data().unwrap()
        );
    }

    #[test]
    pub fn test_split_container_item_using_value_string_regex_pair_divider_delete_src() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new(
            Value::string_default(),
            String::from("col1"),
            0,
            Some(Value::String(String::from("1.12 2.23"))),
        ));

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: ValueStringRegexPairSplit::from(
                "(\\d+\\.\\d+).*(\\d+\\.\\d+)".to_string(),
                true,
            )
            .unwrap(),
            target_left: (Value::float32_default(), 1, String::from("col2")),
            target_right: (Value::float32_default(), 2, String::from("Col3")),
            delete_source_item: true,
        };

        div_at.apply(&mut c).unwrap();

        assert_eq!(2, c.0.len());
        assert_eq!(
            &Value::Float32(1.12_f32),
            c.get_by_idx(1).unwrap().get_data().unwrap()
        );
        assert_eq!(
            &Value::Float32(2.23_f32),
            c.get_by_idx(2).unwrap().get_data().unwrap()
        );
    }

    #[test]
    pub fn test_split_container_item_using_value_string_regex_pair_divider_none() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            Value::string_default(),
            String::from("col1"),
            0,
        ));

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: ValueStringRegexPairSplit::from(
                "(\\d+\\.\\d+).*(\\d+\\.\\d+)".to_string(),
                true,
            )
            .unwrap(),
            target_left: (Value::float32_default(), 1, String::from("col2")),
            target_right: (Value::float32_default(), 2, String::from("Col3")),
            delete_source_item: false,
        };

        div_at.apply(&mut c).unwrap();

        assert_eq!(3, c.0.len());
        assert_eq!(None, c.get_by_idx(1).unwrap().get_data());
        assert_eq!(None, c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    pub fn test_split_container_item_using_value_string_regex_pair_divider_none_delete_src() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            Value::string_default(),
            String::from("col1"),
            0,
        ));

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: ValueStringRegexPairSplit::from(
                "(\\d+\\.\\d+).*(\\d+\\.\\d+)".to_string(),
                true,
            )
            .unwrap(),
            target_left: (Value::float32_default(), 1, String::from("col2")),
            target_right: (Value::float32_default(), 2, String::from("Col3")),
            delete_source_item: true,
        };

        div_at.apply(&mut c).unwrap();

        assert_eq!(2, c.0.len());
        assert_eq!(None, c.get_by_idx(1).unwrap().get_data());
        assert_eq!(None, c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"Value is None, but split_none is false\", src_val: None, details: None })"
    )]
    pub fn test_split_container_item_using_value_string_regex_pair_divider_none_err() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            Value::string_default(),
            String::from("col1"),
            0,
        ));

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: ValueStringRegexPairSplit::from(
                "(\\d+\\.\\d+).*(\\d+\\.\\d+)".to_string(),
                false,
            )
            .unwrap(),
            target_left: (Value::float32_default(), 1, String::from("col2")),
            target_right: (Value::float32_default(), 2, String::from("Col3")),
            delete_source_item: false,
        };

        div_at.apply(&mut c).unwrap();
    }
}
