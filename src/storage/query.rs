use crate::storage::component::{Type, TypeInfo};
use crate::utils::utils::IntersectAll;
use crate::world::{TableId, World};
use crate::{entity, query, storage};
use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::Iter;

use super::{component::Component, table::EntityTable};

// -> Abstractions <- //
pub trait TQueryItem {
    type Collection: TCollection;
    type Item;

    fn get_data(table: *mut EntityTable) -> Self::Collection;
    fn get_data_at(collection: Self::Collection, n: usize) -> Self::Item;
}

pub trait TCollection {
    type Item;
    fn get(&mut self, n: usize) -> Self::Item;
    fn length(&self) -> usize;
}

pub trait TTableKeySingle {
    fn get_key() -> TypeId;
}

pub trait TTableKey {
    fn get_keys() -> Vec<TypeId>;
}

// -> Base Implementations <- //
impl<'a, T: Component> TQueryItem for &'a T {
    type Collection = &'a [T];
    type Item = &'a T;

    fn get_data(table: *mut EntityTable) -> Self::Collection {
        unsafe { (*table).get::<T>() }
    }

    fn get_data_at(collection: Self::Collection, n: usize) -> Self::Item {
        &collection[n]
    }
}

impl<'a, T: Component> TQueryItem for &'a mut T {
    type Collection = &'a mut [T];
    type Item = &'a mut T;

    fn get_data(table: *mut EntityTable) -> Self::Collection {
        unsafe { (*table).get_mut::<T>() }
    }

    fn get_data_at(collection: Self::Collection, n: usize) -> Self::Item {
        &mut collection[n]
    }
}

impl<'a, T: Component> TCollection for &'a [T] {
    type Item = &'a T;

    fn get(&mut self, n: usize) -> Self::Item {
        &self[n]
    }

    fn length(&self) -> usize {
        self.len()
    }
}

impl<'a, T: Component> TCollection for &'a mut [T] {
    type Item = &'a mut T;

    fn get(&mut self, n: usize) -> Self::Item {
        let ptr = self.as_mut_ptr();
        unsafe {
            let value = ptr.add(n).as_mut().unwrap();
            value
        }
    }

    fn length(&self) -> usize {
        self.len()
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


impl<A: TTableKeySingle, B: TTableKeySingle> TTableKey for (A, B) {

    fn get_keys() -> Vec<TypeId> {
        let mut type_ids: Vec<TypeId> = vec![A::get_key(), B::get_key()];
        type_ids.sort();
        type_ids
    }
}

// -> Recursive tuple definitions <- // todo: macros
impl<A: TQueryItem, B: TQueryItem> TQueryItem for (A, B) {
    type Collection = (A::Collection, B::Collection);
    type Item = (A::Item, B::Item);

    fn get_data(table: *mut EntityTable) -> Self::Collection {
        (A::get_data(table), B::get_data(table))
    }

    fn get_data_at(collection: Self::Collection, n: usize) -> Self::Item {
        (
            A::get_data_at(collection.0, n),
            B::get_data_at(collection.1, n),
        )
    }
}

impl<A: TCollection, B: TCollection> TCollection for (A, B) {
    type Item = (A::Item, B::Item);

    fn get(&mut self, n: usize) -> Self::Item {
        (A::get(&mut self.0, n), B::get(&mut self.1, n))
    }

    fn length(&self) -> usize {
        self.0.length()
    }
}

// -> API <- //
pub struct QueryResult<Q: TQueryItem> {
    result: Vec<Q::Collection>,
    inner_index: usize,
    outer_index: usize,
}

impl<Q: TQueryItem> QueryResult<Q> {
    fn new(filtered_tables: Vec<Q::Collection>) -> Self {
        Self {
            result: filtered_tables,
            inner_index: 0,
            outer_index: 0,
        }
    }
}

// todo: this could be replaced by a flatmap over each collection, so long as each collection is
// itself an iterator
impl<Q: TQueryItem> Iterator for QueryResult<Q> {
    type Item = <<Q as TQueryItem>::Collection as TCollection>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.outer_index >= self.result.len() {
            None
        } else if self.inner_index >= self.result[self.outer_index].length() {
            self.outer_index += 1;
            self.inner_index = 0;
            self.next()
        } else {
            // Might be slow because returning a reference each
            let result = self.result[self.outer_index].get(self.inner_index);
            self.inner_index += 1;
            Some(result)
        }
    }
}

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

    pub fn execute(mut self) -> QueryResult<Q> {
        let table_map = &self.world.tables_with_component_id;
        let components = Q::get_keys();

        let matching_table_ids: Vec<&HashSet<TableId>> = components
            .into_iter()
            .filter_map(|id| table_map.get(&id))
            .collect();

        // tables with every requested component
        let query_results = matching_table_ids.intersect_all();

        let mut tables: Vec<<Q as TQueryItem>::Collection> = vec![];
        for table_id in query_results {
            if let Some(table_ref) = self.world.tables.get_mut(&table_id) {
                let table = Q::get_data(table_ref);
                tables.push(table)
            }
        }

        QueryResult::new(tables)
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

    #[test]
    fn test() {
        let mut world = World::new();
        (0..1000).for_each(|_| {
            world.spawn(entity!(1_u8, 2_i32, "hello"));
        });
        (0..1000).for_each(|_| {
            world.spawn(entity!(1_u8, 2_i32, 3_u64));
        });
        let query = QueryInit::<&u8>::new(&mut world).execute();
        assert_eq!(query.count(), 2000)
    }
}
