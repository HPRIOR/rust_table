use bit_set::BitSet;

use super::{
    column::Column,
    component::{Component, Type, TypeInfo},
};
use crate::world::{EntityId, EntityIdGen};
use std::any::TypeId;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct EntityTable {
    // index corresponds with row in table
    entities: Vec<EntityId>,
    // add unique ID
    pub id: BitSet,
    pub columns: Vec<Column>,
    column_info: Vec<TypeInfo>,
}

impl EntityTable {
    pub fn new(type_infos: Vec<TypeInfo>, id: BitSet) -> Self {
        Self {
            entities: Default::default(),
            columns: type_infos.iter().map(|ti| Column::new(*ti)).collect(),
            column_info: type_infos,
            id,
        }
    }

    fn get_column_index(&self, type_info: &TypeInfo) -> Option<usize> {
        let t_id = type_info.id;
        self.column_info.iter().position(|ti| ti.id == t_id)
    }

    pub fn add(&mut self, components: Vec<Box<dyn Component>>, entity: EntityId) {
        self.entities.push(entity);
        components.into_iter().for_each(move |component| {
            let type_info = (*component).type_info();
            let column_index = self.get_column_index(&type_info).unwrap();
            self.columns[column_index].push_component(component)
        });
    }

    pub fn remove(&mut self, input_entity: EntityId) -> EntityId {
        let entity_index = self
            .entities
            .iter()
            .position(|entity| *entity == input_entity);

        if let Some(entity_index) = entity_index {
            self.columns
                .iter_mut()
                .for_each(|column| column.remove(entity_index));

            // Ensure entity index reflects new entity representation in column.
            // Removed entity replaced with top entity to ensure compact array
            self.entities.swap_remove(entity_index);
            input_entity
        } else {
            panic!("Could not find entity in table!")
        }
    }

    /// Caller must check whether column is available in table first - panics
    pub fn get<T: Component>(&self) -> std::slice::Iter<T> {
        let t_info = TypeInfo::of::<T>();
        let index = self.get_column_index(&t_info).unwrap();
        self.columns[index].get_slice().iter()
    }

    /// Caller must check whether column is available in table first - panics
    pub fn get_mut<T: Component>(&mut self) -> std::slice::IterMut<T> {
        let t_info = TypeInfo::of::<T>();
        let index = self.get_column_index(&t_info).unwrap();
        self.columns[index].get_mut_slice().iter_mut()
    }
}

#[cfg(test)]
mod tests {}
