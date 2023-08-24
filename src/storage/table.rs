use bit_set::BitSet;

use super::{
    column::Column,
    component::{Component, Type, TypeInfo},
    query::TQueryItem,
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
    pub column_info: Vec<TypeInfo>,
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

    pub fn add_entity(&mut self, components: Vec<Box<dyn Component>>, entity: EntityId) {
        self.entities.push(entity);
        components.into_iter().for_each(move |component| {
            let type_info = (*component).type_info();
            let column_index = self.get_column_index(&type_info).unwrap();
            self.columns[column_index].push_component(component)
        });
    }

    pub fn remove_entity(&mut self, input_entity: EntityId) -> Vec<Box<dyn Component>> {
        let entity_index = self
            .entities
            .iter()
            .position(|entity| *entity == input_entity);
        let mut components: Vec<Box<dyn Component>>;
        if let Some(entity_index) = entity_index {
            components = self
                .columns
                .iter_mut()
                .map(|column| column.remove_component(entity_index))
                .collect();

            // Ensures entity index reflects new entity representation in column.
            // Removed entity replaced with top entity to ensure compact array
            self.entities.swap_remove(entity_index);
            input_entity
        } else {
            panic!("Could not find entity in table!")
        };
        components
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
mod tests {
    use crate::entity;

    use super::*;
    #[test]
    fn entity_can_be_added_to_table() {
        let mut table = EntityTable::new(
            vec![TypeInfo::of::<i32>(), TypeInfo::of::<u8>()],
            BitSet::new(),
        );

        table.add_entity(entity!(1_i32, 2_u8), EntityId::Value(0));

        let column1: Vec<&i32> = table.get::<i32>().collect();
        let column2: Vec<&u8> = table.get::<u8>().collect();

        assert_eq!(*column1[0], 1);
        assert_eq!(*column2[0], 2);
    }

    #[test]
    fn removing_an_entity_rearranges_table() {
        let mut table = EntityTable::new(
            vec![TypeInfo::of::<i32>(), TypeInfo::of::<u8>()],
            BitSet::new(),
        );

        table.add_entity(entity!(1_i32, 1_u8), EntityId::Value(0));
        table.add_entity(entity!(2_i32, 2_u8), EntityId::Value(1));
        table.add_entity(entity!(3_i32, 3_u8), EntityId::Value(2));
        table.add_entity(entity!(4_i32, 4_u8), EntityId::Value(3));

        let entity = table.remove_entity(EntityId::Value(1));

        let column1: Vec<&i32> = table.get::<i32>().collect();
        let column2: Vec<&u8> = table.get::<u8>().collect();

        assert_eq!(vec![&1, &4, &3], column1);
        assert_eq!(vec![&1, &4, &3], column2);
        assert_eq!(
            vec![EntityId::Value(0), EntityId::Value(3), EntityId::Value(2)],
            table.entities
        )
    }
}
