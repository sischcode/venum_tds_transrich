use venum_tds::traits::{DataAccess, DataContainer, DataIdent};

use crate::{
    errors::{ContainerOpsErrors, Result, VenumTdsTransRichError},
    traits::container::TransrichContainerInplace,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MutateItemIndex {
    pub from: usize,
    pub to: usize,
}
impl MutateItemIndex {
    pub fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
}
impl<C> TransrichContainerInplace<C> for MutateItemIndex
where
    C: DataContainer,
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
pub struct DeleteItemByIndex(pub usize);
impl<C> TransrichContainerInplace<C> for DeleteItemByIndex
where
    C: DataContainer,
{
    fn apply(&self, data_container: &mut C) -> Result<()> {
        match data_container.del_by_idx(self.0) {
            Ok(_) => Ok(()),
            Err(e) => Err(VenumTdsTransRichError::from(e)),
        }
    }
}

pub struct AddItem<T: DataIdent + DataAccess>(pub T);
impl<C, D> TransrichContainerInplace<C> for AddItem<D>
where
    D: DataIdent + DataAccess + Clone + Default,
    C: DataContainer<ITEM = D>,
{
    fn apply(&self, data_container: &mut C) -> Result<()> {
        data_container.add(self.0.clone());
        Ok(())
    }
}

// TODO: splitting

#[cfg(test)]
mod tests {
    use venum::venum::Value;
    use venum_tds::{cell::DataCell, row::DataCellRow};

    use super::*;

    #[test]
    fn test_mutate_index_of_tds_data_cell() {
        let m = MutateItemIndex::new(0, 1);

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

        let container_transricher = DeleteItemByIndex(0);
        container_transricher.apply(&mut c).unwrap();

        let container_transricher2 = DeleteItemByIndex(1);
        container_transricher2.apply(&mut c).unwrap();

        assert_eq!(0, c.0.len());
    }

    #[test]
    #[should_panic(expected = "Wrapped(VenumTdsError(DataAccess(IllegalIdxAccess { idx: 0 })))")]
    fn test_delete_from_container_err() {
        let mut c = DataCellRow::new();
        let container_transricher = DeleteItemByIndex(0);
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
        transrichers.push(Box::new(DeleteItemByIndex(1)));
        transrichers.push(Box::new(DeleteItemByIndex(2)));
        transrichers.push(Box::new(MutateItemIndex::new(0, 10)));

        transrichers
            .iter_mut()
            .map(|t| t.apply(&mut c))
            .collect::<Result<Vec<()>>>()
            .unwrap();

        assert_eq!(1, c.0.len());
        assert_eq!(10, c.0.first().unwrap().get_idx());
        assert_eq!(String::from("col1"), c.0.first().unwrap().get_name());
    }
}
