use super::{
    column::Column,
    component::{Component, Type, TypeInfo},
};
use crate::world::EntityIdGen;
use std::any::TypeId;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct EntityTable {
    // add unique ID
    pub columns: Box<[Column]>,
    column_info: Vec<TypeInfo>,
}


impl EntityTable {
    pub fn new(mut type_infos: Vec<TypeInfo>) -> Self {
        type_infos.sort_by(|a, b| a.id.cmp(&b.id));
        Self {
            columns: type_infos.iter().map(|ti| Column::new(*ti)).collect(),
            column_info: type_infos,
        }
    }

    /// assumes sorted by type_id
    pub fn add(&mut self, mut components: Vec<Box<dyn Component>>) {
        // probably very slow - needed to ensure columns are allocated the correect type -- also
        // duplicated due to the entity! macro
        components.sort_by(|a, b| (**a).type_info().id.cmp(&(**b).type_info().id));
        unsafe {
            (0..(components.len()))
                .rev()
                .for_each(|i| self.columns[i].push_component(components.remove(i)))
        }
    }

    // this should be lifted out to a higher module
    // e.g. ecs system will keep a hashmap of component type to entity tables which contain that type
    pub fn has<T: Component>(&self) -> bool {
        self.column_info
            .iter()
            .any(|ci| ci.id == TypeInfo::of::<T>().id)
    }

    // todo: this should be cached Hash<TypeInfo, usize>
    fn get_column_index<T: Component>(&self, type_info: &TypeInfo) -> Option<usize> {
        let t_id = type_info.id;
        self.column_info
            .iter()
            .enumerate()
            .filter(|(i, ti)| ti.id == t_id)
            .map(|(i, _)| i)
            .next()
    }

    /// Caller must check whether column is available in table first - panics
    pub fn get<T: Component>(&self) -> std::slice::Iter<T> {
        let t_info = TypeInfo::of::<T>();
        let index = self.get_column_index::<T>(&t_info).unwrap();
        self.columns[index].get_column().iter()
    }

    /// Caller must check whether column is available in table first - panics
    pub fn get_mut<T: Component>(&mut self) -> std::slice::IterMut<T> {
        let t_info = TypeInfo::of::<T>();
        let index = self.get_column_index::<T>(&t_info).unwrap();
        self.columns[index].get_column_mut().iter_mut()
    }
}

#[cfg(test)]
mod tests {}
