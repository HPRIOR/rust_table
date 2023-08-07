use crate::storage::component::{Type, TypeInfo};
use crate::utils::utils::IntersectAll;
use crate::world::{TableId, World};
use crate::{entity, query, storage};
use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::iter::{Flatten, Zip};
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::Iter;

use super::{component::Component, table::EntityTable};

// -> Abstractions <- //
pub trait TQueryItem {
    type Item;
    type Collection: Iterator<Item = Self::Item>;

    fn get_data(table: *mut EntityTable) -> Self::Collection;
}

pub trait TTableKeySingle {
    fn get_key() -> TypeId;
}

pub trait TTableKey {
    fn get_keys() -> Vec<TypeId>;
}

// -> Base Implementations <- //
impl<'a, T: Component> TQueryItem for &'a T {
    type Collection = std::slice::Iter<'a, T>;
    type Item = &'a T;

    fn get_data(table: *mut EntityTable) -> Self::Collection {
        unsafe { (*table).get::<T>() }
    }
}

impl<'a, T: Component> TQueryItem for &'a mut T {
    type Collection = std::slice::IterMut<'a, T>;
    type Item = &'a mut T;

    fn get_data(table: *mut EntityTable) -> Self::Collection {
        unsafe { (*table).get_mut::<T>() }
    }
}

impl<'a, T: Component> TTableKeySingle for &'a T {
    fn get_key() -> TypeId {
        TypeInfo::of::<T>().id
    }
}

impl<'a, T: Component> TTableKey for &'a T {
    fn get_keys() -> Vec<TypeId> {
        vec![TypeInfo::of::<T>().id]
    }
}

impl<'a, T: Component> TTableKeySingle for &'a mut T {
    fn get_key() -> TypeId {
        TypeInfo::of::<T>().id
    }
}

impl<'a, T: Component> TTableKey for &'a mut T {
    fn get_keys() -> Vec<TypeId> {
        vec![TypeInfo::of::<T>().id]
    }
}

impl<A: TTableKeySingle, B: TTableKeySingle> TTableKey for (A, B) {
    fn get_keys() -> Vec<TypeId> {
        let mut type_ids: Vec<TypeId> = vec![A::get_key(), B::get_key()];
        type_ids.sort();
        type_ids
    }
}

// -> Recursive tuple definitions <- // todo: macros
impl<A: TQueryItem, B: TQueryItem> TQueryItem for (A, B) {
    type Item = (A::Item, B::Item);
    type Collection = Zip<A::Collection, B::Collection>;

    fn get_data(table: *mut EntityTable) -> Self::Collection {
        A::get_data(table).zip(B::get_data(table))
    }
}

// -> API <- //
pub struct QueryInit<'world, Q: TQueryItem> {
    world: &'world mut World,
    _marker: PhantomData<Q>,
}

impl<'world, Q: TQueryItem + TTableKey> QueryInit<'world, Q> {
    pub fn new(world: &'world mut World) -> Self {
        Self {
            world,
            _marker: Default::default(),
        }
    }

    pub fn execute(mut self) -> Box<dyn Iterator<Item = Q::Item> + 'world> {
        let table_map = &self.world.tables_with_component_id;
        // this should be a bitset 
        let component_keys = Q::get_keys();

        // todo this should return a vector of bitsets from table_ids_with_signature
        let matching_table_ids: Vec<&HashSet<TableId>> = component_keys
            .into_iter()
            .filter_map(|id| table_map.get(&id))
            .collect();

        // todo when bitsets bitwise comparisons can be made (equality, subset, superset, disjoint)
        // extremely efficiently 
        let result = matching_table_ids
            .intersect_all()
            .into_iter()
            .filter_map(|table_id| {
                self.world
                    .tables
                    .get_mut(&table_id)
                    .map(|table| Q::get_data(table))
            })
            .flatten();

        Box::new(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::component::Component;
    use crate::{
        entity,
        storage::query::TTableKeySingle,
        world::{EntityId, World},
    };
    use std::any::{Any, TypeId};

    use super::QueryInit;

    #[derive(Debug)]
    enum Id1 {
        Id1(u8),
    }
    #[derive(Debug)]
    enum Id2 {
        Id2(u8),
    }

    #[test]
    fn test() {
        let mut world = World::new();
        (0..5).for_each(|_| {
            world.spawn(entity!(Id1::Id1(8), Id2::Id2(4)));
        });
        let query = QueryInit::<(&Id1, &Id2)>::new(&mut world).execute();
        for (item, item2) in query {
            println!("{:?}, {:?}", item, item2);
        }
        // assert_eq!(query.count(), 2000)
    }
}
