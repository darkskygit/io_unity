pub mod type_tree;

use crate::{def_unity_class, error::ReadResult, type_tree::TypeTreeObjectRef};

def_unity_class!(KeyValuePair);

pub trait KeyValuePairObject {
  fn get_pair(&self) -> ReadResult<(TypeTreeObjectRef, TypeTreeObjectRef)>;
}
