use std::{
    alloc::Layout,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

use crate::world::EntityId;

use super::component::{Component, Type, TypeInfo};
use std::alloc;

#[derive(Debug)]
pub struct Column {
    ptr: NonNull<u8>,
    type_info: TypeInfo,
    len: usize,
    cap: usize,
    _marker: PhantomData<u8>,
}

impl Column {
    pub fn new(type_info: TypeInfo) -> Self {
        Self {
            ptr: NonNull::new(type_info.layout.align() as *mut u8).unwrap(),
            len: 0,
            cap: 0,
            _marker: PhantomData,
            type_info,
        }
    }

    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            let layout = Layout::from_size_align(
                self.type_info.layout.size(),
                self.type_info.layout.align(),
            )
            .unwrap();
            (1, layout)
        } else {
            let cap = 2 * self.cap;
            let layout = Layout::from_size_align(
                self.type_info.layout.size() * cap,
                self.type_info.layout.align(),
            )
            .unwrap();
            (cap, layout)
        };
        assert!(new_layout.size() <= isize::MAX as usize);

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::from_size_align(
                self.type_info.layout.size() * self.cap,
                self.type_info.layout.align(),
            )
            .unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        self.ptr = match NonNull::new(new_ptr) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }

    fn push_raw(&mut self, component_ptr: *mut u8) {
        if self.len == self.cap {
            self.grow();
        }
        let size = self.type_info.layout.size();
        let index = self.len;

        unsafe {
            let dest = self.ptr.as_ptr().add(size * index).cast::<u8>();
            ptr::copy_nonoverlapping(component_ptr, dest, size)
        }

        self.len += 1;
    }

    fn push<T: Component>(&mut self, mut component: T) {
        unsafe {
            let ptr = Type::get_ptr(&mut component);
            self.push_raw(ptr);
        }
    }

    pub fn push_component(&mut self, component: Box<dyn Component>) {
        unsafe {
            self.push_raw(Type::get_box_ptr(component).cast());
        }
    }

    pub fn get_slice<T: Component>(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr().cast::<T>(), self.len) }
    }

    pub fn get_mut_slice<T: Component>(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr().cast::<T>(), self.len) }
    }


    pub fn get<T: Component>(&mut self, index: usize) -> Option<&T> {
        if index > self.len {
            None
        } else {
            unsafe { self.ptr.as_ptr().cast::<T>().add(index).as_ref() }
        }
    }

    pub fn get_mut<T: Component>(&mut self, index: usize) -> Option<&mut T> {
        if index > self.len {
            None
        } else {
            unsafe { self.ptr.as_ptr().cast::<T>().add(index).as_mut() }
        }
    }

    /// Warning: last entity replaces removed entity. Any entity tracking vector needs to be
    /// modified by caller to reflect change
    pub fn remove(&mut self, entity_index: usize) {
        let size = self.type_info.layout.size();
        if self.len - 1 == entity_index {
            unsafe {
                ptr::drop_in_place(self.ptr.as_ptr().add(self.len * size));
            }
            self.len -= 1;
        } else {
            unsafe {
                let to_remove = self.ptr.as_ptr().add(entity_index * size);
                let top = self.ptr.as_ptr().add((self.len - 1) * size).read();
                ptr::replace(to_remove, top);
                self.len -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::component::TypeInfo;

    use super::Column;

    #[test]
    fn can_create_with_an_arbitrary_type() {
        let mut column = Column::new(TypeInfo::of::<i32>());
        column.push(1);
        column.push(2);
        column.push(3);
        assert_eq!(*column.get::<i32>(0).unwrap(), 1);
        assert_eq!(*column.get::<i32>(1).unwrap(), 2);
        assert_eq!(*column.get::<i32>(2).unwrap(), 3);
    }

    #[test]
    fn can_modify_with_get_mut() {
        let mut column = Column::new(TypeInfo::of::<i32>());
        column.push(1);
        column.push(2);
        column.push(3);

        let second = column.get_mut::<i32>(1).unwrap();
        *second = 10;
        assert_eq!(*column.get::<i32>(1).unwrap(), 10);
    }

    #[test]
    fn removal_at_index_ensures_compact_data() {
        let mut column = Column::new(TypeInfo::of::<i32>());
        column.push(1);
        column.push(2);
        column.push(3);
        column.remove(1);

        assert_eq!(*column.get::<i32>(0).unwrap(), 1);
        assert_eq!(*column.get::<i32>(1).unwrap(), 3);
    }
}
