use std::any::TypeId;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;
use crate::storage::query::TQueryItem;
use super::{
    column::Column,
    component::{Component, Type, TypeInfo},
};

#[derive(Debug)]
pub struct EntityTable {
    pub columns: Box<[Column]>,
    column_info: Vec<TypeInfo>,
    column_id_set: HashSet<TypeId>,
}

impl EntityTable {
    pub fn new(type_infos: Vec<TypeInfo>) -> Self {
        Self {
            columns: type_infos.iter().map(|ti| Column::new(*ti)).collect(),
            column_id_set: type_infos.iter().map(|ti| ti.id).collect(),
            column_info: type_infos,
        }
    }

    pub fn add(&mut self, mut components: Vec<Box<dyn Component>>) {
        unsafe {
            (0..(components.len()))
                .rev()
                .for_each(|i| self.columns[i].push_component(components.remove(i)))
        }
    }

    pub fn has<T: Component>(&self) -> bool {
        self.column_info
            .iter()
            .any(|ci| ci.id == TypeInfo::of::<T>().id)
    }

    pub fn has_query<'q, Q: TQueryItem<'q>>(&self) -> bool {
        // let query_typeinfo = TypeInfo::of::<Q>();
        todo!()
        // if self.column_info.iter().map(|ci| ci.type_name).
    }

    /// Returns true if table contains all of the input types
    pub fn has_signature(&self, type_ids: &HashSet<TypeId>) -> bool {
        type_ids.intersection(&self.column_id_set).count() == type_ids.len()
    }

    fn get_column_index<T: Component>(&self, type_info: &TypeInfo) -> Option<usize> {
        let t_id = type_info.id;
        self.column_info
            .iter()
            .enumerate()
            .filter(|(i, ti)| ti.id == t_id)
            .map(|(i, _)| i)
            .nth(0)
    }

    /// Caller must check whether column is available in table first - panics
    pub fn get<T: Component>(&self) -> &[T] {
        let t_info = TypeInfo::of::<T>();
        let index = self.get_column_index::<T>(&t_info).unwrap();
        self.columns[index].get_column()
    }

    /// Caller must check whether column is available in table first - panics
    pub fn get_mut<T: Component>(&mut self) -> &mut [T] {
        let t_info = TypeInfo::of::<T>();
        let index = self.get_column_index::<T>(&t_info).unwrap();
        self.columns[index].get_column_mut()
    }
}

#[cfg(test)]
mod tests {}
