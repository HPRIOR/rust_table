use std::{
    alloc::Layout,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

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
                self.type_info.layout.size() * 1,
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

    pub fn push(&mut self, component_ptr: *mut u8) {
        if self.len == self.cap {
            self.grow();
        }
        let type_id = self.type_info.id;
        let size = self.type_info.layout.size();
        let index = self.len;

        unsafe {
            let dest = self.ptr.as_ptr().add(size * index).cast::<u8>();
            ptr::copy_nonoverlapping(component_ptr, dest, size)
        }

        self.len += 1;
    }

    pub fn push_component(&mut self, component: Box<dyn Component>) {
        unsafe {
            self.push(Type::get_box_ptr(component).cast());
        }
    }

    pub fn get_column<T: Component>(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr().cast::<T>(), self.len) }
    }
    pub fn get_column_mut<T: Component>(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr().cast::<T>(), self.len) }
    }
}
