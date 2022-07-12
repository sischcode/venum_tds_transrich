use venum_tds::traits::VDataContainer;

use crate::errors::Result;
use crate::traits::container::TransrichContainerInplace;

pub struct TransrichPass<C: VDataContainer> {
    pub transformer: Vec<Box<dyn TransrichContainerInplace<C>>>,
    pub order: Option<Vec<Box<dyn TransrichContainerInplace<C>>>>,
}

impl<C: VDataContainer> TransrichPass<C> {
    pub fn apply(&mut self, container: &mut C) -> Result<()> {
        self.transformer
            .iter()
            .try_for_each(|tri| tri.apply(container))?;

        if let Some(orderings) = &self.order {
            orderings.iter().try_for_each(|o| o.apply(container))?;
        }
        Ok(())
    }
}

pub struct TransrichPassesConfig<C: VDataContainer> {
    pub passes: Vec<TransrichPass<C>>,
}

impl<C: VDataContainer> TransrichPassesConfig<C> {
    pub fn apply(&mut self, container: &mut C) -> Result<()> {
        self.passes
            .iter_mut()
            .try_for_each(|pass| pass.apply(container))
    }
}

#[cfg(test)]
mod tests {
    use crate::container::{DeleteItemAtIdx, MutateItemIdx, SplitItemAtIdx};
    use crate::container_transrich_pass::TransrichPassesConfig;
    use crate::value_splitting::ValueStringSeparatorCharSplit;

    use super::TransrichPass;
    use venum::venum::Value;
    use venum_tds::cell::DataCell;
    use venum_tds::row::DataCellRow;
    use venum_tds::traits::VDataContainer;

    // TODO: more tests!

    #[test]
    fn test_transrich_pass_del_after_split() {
        let mut trp: TransrichPass<DataCellRow> = TransrichPass {
            transformer: vec![Box::new(SplitItemAtIdx {
                delete_source_item: true,
                idx: 0,
                target_left: (Value::float32_default(), 1, String::from("amount")),
                target_right: (Value::string_default(), 2, String::from("currency")),
                splitter: ValueStringSeparatorCharSplit {
                    sep_char: ' ',
                    split_none: true,
                },
            })],
            order: Some(vec![
                Box::new(MutateItemIdx { from: 1, to: 0 }), // CAUTION!!!
                Box::new(MutateItemIdx { from: 2, to: 1 }), // You need to order from low to high!
            ]),
        };

        let mut data = DataCellRow::new();
        data.add(DataCell::new(
            Value::string_default(),
            String::from("amount+currency"),
            0,
            Some(Value::String(String::from("10.10 CHF"))),
        ));

        trp.apply(&mut data).unwrap();

        assert_eq!(2, data.0.len());
        assert_eq!(
            Some(Value::Float32(10.10)),
            data.get_by_idx(0).unwrap().data
        );
        assert_eq!(
            Some(Value::String(String::from("CHF"))),
            data.get_by_idx(1).unwrap().data
        );
    }

    #[test]
    fn test_transrich_pass_remain_after_split_then_delete() {
        let mut trp: TransrichPass<DataCellRow> = TransrichPass {
            transformer: vec![
                Box::new(SplitItemAtIdx {
                    delete_source_item: false, // <--- false this time!
                    idx: 0,
                    target_left: (Value::float32_default(), 1, String::from("amount")),
                    target_right: (Value::string_default(), 2, String::from("currency")),
                    splitter: ValueStringSeparatorCharSplit {
                        sep_char: ' ',
                        split_none: true,
                    },
                }),
                Box::new(DeleteItemAtIdx { 0: 0 }),
            ],
            order: Some(vec![
                Box::new(MutateItemIdx { from: 1, to: 0 }),
                Box::new(MutateItemIdx { from: 2, to: 1 }),
            ]),
        };

        let mut data = DataCellRow::new();
        data.add(DataCell::new(
            Value::string_default(),
            String::from("amount+currency"),
            0,
            Some(Value::String(String::from("10.10 CHF"))),
        ));

        trp.apply(&mut data).unwrap();

        assert_eq!(2, data.0.len());
        assert_eq!(
            Some(Value::Float32(10.10)),
            data.get_by_idx(0).unwrap().data
        );
        assert_eq!(
            Some(Value::String(String::from("CHF"))),
            data.get_by_idx(1).unwrap().data
        );
    }

    #[test]
    fn test_transrich_passes() {
        let trp1: TransrichPass<DataCellRow> = TransrichPass {
            transformer: vec![Box::new(SplitItemAtIdx {
                delete_source_item: false, // <--- false this time!
                idx: 0,
                target_left: (Value::float32_default(), 1, String::from("amount")),
                target_right: (Value::string_default(), 2, String::from("currency")),
                splitter: ValueStringSeparatorCharSplit {
                    sep_char: ' ',
                    split_none: true,
                },
            })],
            order: Some(vec![
                Box::new(MutateItemIdx { from: 0, to: 3 }), // move the old "column" out of the way
                Box::new(MutateItemIdx { from: 1, to: 0 }),
                Box::new(MutateItemIdx { from: 2, to: 1 }),
            ]),
        };

        let trp2: TransrichPass<DataCellRow> = TransrichPass {
            transformer: vec![Box::new(DeleteItemAtIdx { 0: 3 })],
            order: None,
        };

        let mut passes_config = TransrichPassesConfig {
            passes: vec![trp1, trp2],
        };

        let mut data = DataCellRow::new();
        data.add(DataCell::new(
            Value::string_default(),
            String::from("amount+currency"),
            0,
            Some(Value::String(String::from("10.10 CHF"))),
        ));

        passes_config.apply(&mut data).unwrap();

        assert_eq!(2, data.0.len());
        assert_eq!(
            Some(Value::Float32(10.10)),
            data.get_by_idx(0).unwrap().data
        );
        assert_eq!(
            Some(Value::String(String::from("CHF"))),
            data.get_by_idx(1).unwrap().data
        );
    }
}
