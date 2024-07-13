pub mod type_tree;

use crate::{def_unity_class, error::ReadResult};

def_unity_class!(Guid);

pub trait GuidObject {
    fn get_guid(&self) -> ReadResult<String>;
}
