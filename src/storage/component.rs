use std::{
    alloc::Layout,
    any::TypeId,
    collections::{HashMap, HashSet},
    ptr::{self, NonNull},
};

// We need some efficient way to identify groups of components that make up a table
// This can be done using a bitset: each component type has an index into a bitset,
// an array of components can then be transformed into a bitset for fast comparisons.
// Primitive types can be assigned an index into a bitset,
// custom types will need to be registered, hopefully with attribute macros:
// #[component]
// struct Vector3 { ... }
//

#[derive(Default)]
struct TypeBitMap {
    map: HashSet<TypeId, u8>,
}

impl TypeBitMap {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TypeInfo {
    pub id: TypeId,
    pub layout: Layout,
    pub type_name: &'static str,
    pub drop: unsafe fn(*mut u8),
    pub to_component: unsafe fn(*mut u8) -> Box<dyn Component>,
    pub replace: unsafe fn(*mut u8, *mut u8) -> Box<dyn Component>,
}

impl TypeInfo {
    pub fn of<T: Component>() -> Self {
        // Overloads for 'untyped' pointer functions. This allows typed functions to to be used
        // in untyped contexts where type information is available. Need to check how this is
        // monomorphised, could make typeinfo to large, and it's copy
        unsafe fn drop_ptr<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place()
        }

        unsafe fn to_component<T: Component>(x: *mut u8) -> Box<dyn Component> {
            let component = ptr::read(x.cast::<T>());
            Box::new(component)
        }
        unsafe fn replace<T: Component>(dest: *mut u8, src: *mut u8) -> Box<dyn Component> {
            let src = ptr::read(src.cast::<T>());
            let dst = dest.cast::<T>();
            let removed = ptr::replace(dst, src);
            Box::new(removed)
        }

        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            drop: drop_ptr::<T>,
            to_component: to_component::<T>,
            replace: replace::<T>,
            type_name: core::any::type_name::<T>(),
        }
    }
}

impl Ord for TypeInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for TypeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.type_name.partial_cmp(other.type_name)
    }
}

pub trait Component: Send + Sync + 'static {
    fn to_component_ref(self) -> Box<dyn Component>;
    fn type_info(&self) -> TypeInfo;
}

impl PartialEq for Box<dyn Component> {
    fn eq(&self, other: &Self) -> bool {
        (**self).type_info().id == (**other).type_info().id
    }
}

impl<T: Send + Sync + 'static> Component for T {
    fn to_component_ref(self) -> Box<dyn Component> {
        Box::new(self)
    }
    fn type_info(&self) -> TypeInfo {
        TypeInfo::of::<T>()
    }
}

pub struct Type {}

impl Type {
    pub unsafe fn get_ptr<T: Component>(component: &mut T) -> *mut u8 {
        (component as *mut T).cast()
        // core::mem::forget(component);
    }

    pub unsafe fn get_box_ptr(component: Box<dyn Component>) -> *mut u8 {
        Box::into_raw(component).cast()
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::component::Component;

    #[test]
    fn component_type_info_will_retrieve_correct_name() {
        let str = "hello";
        let type_info = str.type_info();
        assert_eq!(type_info.type_name, "&str")
    }
}
