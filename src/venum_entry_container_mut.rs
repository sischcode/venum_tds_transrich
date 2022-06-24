use venum_tds::errors::VenumTdsError;
use venum_tds::traits::{DataContainer, DataEntry};

use crate::errors::{ContainerMutErrors, Result, VenumTdsTransRichError};

pub trait TransrichContainerInplace<D: DataEntry, C: DataContainer<D>> {
    fn apply(&mut self, container: &mut C) -> Result<()>; // TODO: not really happy about that. the mut is just needed for the add-transricher, so that we can mem::take
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutateEntriesIndices {
    pub from: usize,
    pub to: usize,
}
impl MutateEntriesIndices {
    pub fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
}

impl<D, C> TransrichContainerInplace<D, C> for MutateEntriesIndices
where
    D: DataEntry,
    C: DataContainer<D>,
{
    fn apply(&mut self, data_container: &mut C) -> Result<()> {
        let container_entry = data_container.get_by_idx_mut(self.from);
        match container_entry {
            None => Err(VenumTdsTransRichError::ContainerMut(
                ContainerMutErrors::Generic {
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
pub struct DeleteEntriesByIndices {
    indices: Vec<usize>,
}
impl DeleteEntriesByIndices {
    pub fn new(indices: Vec<usize>) -> Self {
        Self { indices }
    }
}

impl<D, C> TransrichContainerInplace<D, C> for DeleteEntriesByIndices
where
    D: DataEntry,
    C: DataContainer<D>,
{
    fn apply(&mut self, data_container: &mut C) -> Result<()> {
        let deleted = self
            .indices
            .iter()
            .map(|&idx| data_container.del_by_idx(idx))
            .collect::<std::result::Result<Vec<D>, VenumTdsError>>();

        if let Err(e) = deleted {
            return Err(VenumTdsTransRichError::from(e));
        }
        Ok(())
    }
}

pub struct ContainerAppendEntry<T: DataEntry>(pub T);

impl<D, C> TransrichContainerInplace<D, C> for ContainerAppendEntry<D>
where
    D: DataEntry + Default,
    C: DataContainer<D>,
{
    fn apply(&mut self, data_container: &mut C) -> Result<()> {
        data_container.add(std::mem::take(&mut self.0));
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
        let mut m = MutateEntriesIndices::new(0, 1);

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

        let mut container_transricher = DeleteEntriesByIndices::new(vec![0, 1]);
        container_transricher.apply(&mut c).unwrap();

        assert_eq!(0, c.0.len());
    }

    #[test]
    #[should_panic(expected = "Wrapped(VenumTdsError(DataAccess(IllegalIdxAccess { idx: 0 })))")]
    fn test_delete_from_container_err() {
        let mut c = DataCellRow::new();
        let mut container_transricher = DeleteEntriesByIndices::new(vec![0]);
        container_transricher.apply(&mut c).unwrap();
    }

    #[test]
    fn test_add_to_container() {
        let mut c = DataCellRow::new();
        let mut container_transricher = ContainerAppendEntry(DataCell::new_without_data(
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

        let mut transrichers: Vec<Box<dyn TransrichContainerInplace<DataCell, DataCellRow>>> =
            Vec::with_capacity(3);

        transrichers.push(Box::new(ContainerAppendEntry(DataCell::new_without_data(
            Value::bool_default(),
            String::from("col3"),
            2,
        ))));
        transrichers.push(Box::new(DeleteEntriesByIndices::new(vec![1, 2])));
        transrichers.push(Box::new(MutateEntriesIndices::new(0, 10)));

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
