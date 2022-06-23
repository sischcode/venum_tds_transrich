use venum_tds::traits::Indexed;

use crate::errors::Result;

pub trait TransformEnrichInplace<T> {
    fn apply(&self, imf: &mut T) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutateIndexOfTds {
    pub from: usize,
    pub to: usize,
}
impl MutateIndexOfTds {
    pub fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
}
impl<T> TransformEnrichInplace<T> for MutateIndexOfTds
where
    T: Indexed,
{
    fn apply(&self, d: &mut T) -> Result<()> {
        d.set_idx(self.to);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use venum::venum::Value;
    use venum_tds::cell::DataCell;

    use super::*;

    #[test]
    fn test_mutate_index_of_tds_data_cell() {
        let m = MutateIndexOfTds::new(0, 1);
        let mut d = DataCell::new_without_data(Value::bool_default(), String::from("col1"), 0);

        m.apply(&mut d).unwrap();
        assert_eq!(1, d.get_idx());
    }
}
