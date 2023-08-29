use bit_set::BitSet;

use crate::storage::component::{Type, TypeInfo};
use crate::utils::utils::IntersectAll;
use crate::world::{TableId, World};
use crate::{entity, query, storage};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
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

pub trait TTableKey {
    // todo bitmap
    fn get_key(type_map: &[TypeId]) -> BitSet;
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

impl<'a, T: Component> TTableKey for &'a T {
    fn get_key(type_map: &[TypeId]) -> BitSet {
        let type_info = TypeInfo::of::<T>().id;
        let i = type_map
            .iter()
            .position(|type_id| *type_id == type_info)
            .unwrap();
        let mut bit_set = BitSet::new();
        bit_set.insert(i);
        bit_set
    }
}

impl<'a, T: Component> TTableKey for &'a mut T {
    fn get_key(type_map: &[TypeId]) -> BitSet {
        let type_info = TypeInfo::of::<T>().id;
        let i = type_map
            .iter()
            .position(|type_id| *type_id == type_info)
            .unwrap();
        let mut bit_set = BitSet::new();
        bit_set.insert(i);
        bit_set
    }
}

impl<A: TTableKey, B: TTableKey> TTableKey for (A, B) {
    fn get_key(type_map: &[TypeId]) -> BitSet {
        let mut bit_set_a = A::get_key(type_map);
        let bit_set_b = B::get_key(type_map);
        bit_set_a.union_with(&bit_set_b);
        bit_set_a
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

    pub fn execute(mut self) -> impl Iterator<Item = Q::Item> + 'world {
        let table_sigs = &self.world.table_ids_with_signature;
        let type_id_index = &self.world.type_id_index;
        let component_keys: BitSet = Q::get_key(type_id_index);

        let table_columns = table_sigs
            .keys()
            .filter(move |table_sig| component_keys.is_subset(table_sig))
            .filter_map(|key| table_sigs.get(key))
            .filter_map(|table_id| {
                self.world
                    .tables
                    .get_mut(table_id)
                    .map(|table| Q::get_data(table))
            })
            .flatten();

        table_columns
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::component::Component;
    use crate::{
        entity,
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
        let amount = 1000000;
        (0..amount).for_each(|_| {
            world.spawn(entity!(8_u8, 20_i32), None);
        });
        (0..amount).for_each(|_| {
            world.spawn(entity!(9_u8, 10_i32), None);
        });
        println!("{} entities created", amount * 2);
        let query = QueryInit::<(&u8, &mut i32)>::new(&mut world).execute();
        let mut t = 0;
        for (x, y) in query {
            *y += *x as i32;
            t += *y;
            // println!("{:?}", y);
        }


        println!("query complete: {}", t);
        // assert_eq!(query.count(), 2000)
    }
}
