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
    pub columns: Vec<Column>,
    column_info: Vec<TypeInfo>,
}

impl EntityTable {
    pub fn new(type_infos: Vec<TypeInfo>) -> Self {
        Self {
            columns: type_infos.iter().map(|ti| Column::new(*ti)).collect(),
            column_info: type_infos,
        }
    }

    fn get_column_index(&self, type_info: &TypeInfo) -> Option<usize> {
        let t_id = type_info.id;
        self.column_info.iter().position(|ti| ti.id == t_id)
    }

    pub fn add(&mut self, components: Vec<Box<dyn Component>>) {
        components.into_iter().for_each(move |component| {
            let type_info = (*component).type_info();
            let column_index = self.get_column_index(&type_info).unwrap();
            self.columns[column_index].push_component(component)
        });
    }

    /// Caller must check whether column is available in table first - panics
    pub fn get<T: Component>(&self) -> std::slice::Iter<T> {
        let t_info = TypeInfo::of::<T>();
        let index = self.get_column_index(&t_info).unwrap();
        self.columns[index].get_column().iter()
    }

    /// Caller must check whether column is available in table first - panics
    pub fn get_mut<T: Component>(&mut self) -> std::slice::IterMut<T> {
        let t_info = TypeInfo::of::<T>();
        let index = self.get_column_index(&t_info).unwrap();
        self.columns[index].get_column_mut().iter_mut()
    }
}

#[cfg(test)]
mod tests {}
