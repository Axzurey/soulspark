use std::any::{type_name_of_val, Any};

pub fn get_typed<T: 'static>(value: &mut dyn Any) -> Option<&mut T> {
    
    value.downcast_mut::<T>()
}