use venum_tds::traits::DataContainer;

use crate::errors::Result;

pub trait TransrichContainerInplace<C: DataContainer> {
    fn apply(&self, container: &mut C) -> Result<()>;
}
