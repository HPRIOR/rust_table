#![allow(unused)]

use std::{
    alloc::{self, Layout},
    any::TypeId,
    marker::PhantomData,
    mem::size_of,
    ops::Deref,
    ptr::{self, NonNull},
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

#[derive(Debug)]
struct Table {
    columns: Box<[Column]>,
    column_info: Vec<TypeInfo>,
}
impl Table {
    fn new(type_infos: Vec<TypeInfo>) -> Self {
        Self {
            columns: type_infos.iter().map(|ti| Column::new(*ti)).collect(),
            column_info: type_infos,
        }
    }

    fn add(&mut self, mut components: Vec<Box<dyn Component>>) {
        unsafe {
            (0..(components.len()))
                .rev()
                .for_each(|i| self.columns[i].push(Type::get_box_ptr(components.remove(i))))
        }
    }
}

#[derive(Debug)]
struct Column {
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

    fn push(&mut self, component_ptr: *mut u8) {
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

    fn push_component<T: Component>(&mut self, component: &mut T) {
        unsafe {
            let ptr = Type::get_ptr(component);
            self.push(ptr);
        }
    }
    fn push_component_trait(&mut self, component: Box<dyn Component>) {
        unsafe {
            self.push(Type::get_box_ptr(component).cast());
        }
    }

    fn get_components<T: Component>(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr().cast::<T>(), self.len) }
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

struct Type {}

impl Type {
    fn info<T: Component>() -> TypeInfo {
        TypeInfo::of::<T>()
    }

    fn info_from<T: Component>(component: &T) -> TypeInfo {
        TypeInfo::of::<T>()
    }

    fn as_boxed<T: Component>(component: T) -> Box<dyn Component> {
        Box::new(component)
    }

    unsafe fn get_ptr<T: Component>(component: &mut T) -> *mut u8 {
        (component as *mut T).cast()
        // core::mem::forget(component);
    }

    unsafe fn get_box_ptr(component: Box<dyn Component>) -> *mut u8 {
        Box::into_raw(component).cast()
    }
}

fn main() {

    let mut entity = vec![12.as_component(), "as component".as_component()];
    entity.sort_by(|a, b| a.type_info().partial_cmp(&b.type_info()).unwrap());

    let type_infos: Vec<TypeInfo> = entity.iter().map(|c| (**c).type_info()).collect();

    let mut table = Table::new(type_infos);

    table.add(entity);

    let comps: &[i32] = table.columns[0].get_components();
    let comps2: &[&str] = table.columns[1].get_components();
    println!("{:#?}", comps);
    println!("{:#?}", comps2);

    println!("{:#?}", table);
}
