use std::{
    alloc::Layout,
    any::TypeId,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TypeInfo {
    pub id: TypeId,
    pub layout: Layout,
    pub drop: unsafe fn(*mut u8),
    pub type_name: &'static str,
}

impl Ord for TypeInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for TypeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.type_name.partial_cmp(&other.type_name)
    }
}

impl TypeInfo {
    pub fn of<T: 'static>() -> Self {
        unsafe fn drop_ptr<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place()
        }
        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            drop: drop_ptr::<T>,
            type_name: core::any::type_name::<T>(),
        }
    }
}

pub trait Component: Send + Sync + 'static {
    fn as_component(self) -> Box<dyn Component>;
    fn type_info(&self) -> TypeInfo;
}

impl<T: Send + Sync + 'static> Component for T {
    fn as_component(self) -> Box<dyn Component> {
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
