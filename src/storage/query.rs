use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::Iter;
use crate::ecs::World;
use crate::entity;
use crate::storage::component::TypeInfo;

use super::{component::Component, table::EntityTable};

// Current problem:
// Need to TQueryItem for &mut T.
// In order to get items from the table with T, the get() requires a reference to the EntityTable
// To call get_mut on this table requires a mutable reference to EntityTable.
// In order to implement this for Tuples, multiple mutable references to table are required
// which is prohibited by safe rust. We know that tuples must be of different types, so any calls
// to table.get_mut will not be accessing the same data;


// -> Abstractions <- //

pub trait TQueryItem
{
    type Collection: TCollection;
    type Item;

    fn get(table: *mut EntityTable) -> Self::Collection;
    fn get_at(collection: Self::Collection, n: usize) -> Self::Item;
}

pub trait TCollection {
    type Item;
    fn get(&mut self, n: usize) -> Self::Item;
    // Item must live as long as the
    fn length(&self) -> usize;
}

pub trait TFilter {
    fn filter(query_filter: &mut QueryFilter);
}


// -> Base Implementations <- //

impl<'a, T: Component> TQueryItem for &'a T {
    type Collection = &'a [T];
    type Item = &'a T;


    fn get(table: *mut EntityTable) -> Self::Collection {
        unsafe {
            (*table).get::<T>()
        }
    }

    fn get_at(collection: Self::Collection, n: usize) -> Self::Item {
        &collection[n]
    }
}

impl<'a, T: Component> TQueryItem for &'a mut T {
    type Collection = &'a mut [T];
    type Item = &'a mut T;


    fn get(table: *mut EntityTable) -> Self::Collection {
        unsafe {
            (*table).get_mut::<T>()
        }
    }

    fn get_at(collection: Self::Collection, n: usize) -> Self::Item {
        &mut collection[n]
    }
}


impl<'a, T: Component> TFilter for &'a T {
    fn filter(query_filter: &mut QueryFilter) {
        let ti = TypeInfo::of::<T>().id;
        query_filter.included.insert(ti);
    }
}

impl<'a, T: Component> TFilter for &'a mut T {
    fn filter(query_filter: &mut QueryFilter) {
        let ti = TypeInfo::of::<T>().id;
        query_filter.included.insert(ti);
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
            let value =
                ptr.add(n).as_mut().unwrap();
            value
        }
    }

    fn length(&self) -> usize {
        self.len()
    }
}


// -> Recursive tuple definitions <- //
// todo: make macro for all tuples
//
impl<'a, A: TQueryItem, B: TQueryItem> TQueryItem for (A, B) {
    type Collection = (
        A::Collection,
        B::Collection
    );
    type Item = (A::Item, B::Item);

    fn get(table: *mut EntityTable) -> Self::Collection {
        (A::get(table), B::get(table))
    }

    fn get_at(collection: Self::Collection, n: usize) -> Self::Item {
        (A::get_at(collection.0, n), B::get_at(collection.1, n))
    }
}

impl<'a, A: TCollection, B: TCollection> TCollection for (A, B) {
    type Item = (A::Item, B::Item);

    fn get(&mut self, n: usize) -> Self::Item {
        (A::get(&mut self.0, n), B::get(&mut self.1, n))
    }

    fn length(&self) -> usize {
        self.0.length()
    }
}


impl<A: TFilter, B: TFilter> TFilter for (A, B) {
    fn filter(query_filter: &mut QueryFilter) {
        A::filter(query_filter);
        B::filter(query_filter);
    }
}


struct Without<T: Component>(PhantomData<T>);

impl<T: Component> TFilter for Without<T> {
    fn filter(query_filter: &mut QueryFilter) {
        todo!()
    }
}

// -> API <- //

pub struct QueryFilter {
    included: HashSet<TypeId>,
    excluded: HashSet<TypeId>,
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            included: Default::default(),
            excluded: Default::default(),
        }
    }
}

impl QueryFilter {
    fn new() -> Self {
        Self {
            included: Default::default(),
            excluded: Default::default(),
        }
    }

    fn signature(&self) -> HashSet<TypeId> {
        self.included
            .difference(&self.excluded)
            .map(|t| *t)
            .collect()
    }
}

pub struct QueryExecutor<'a, Q: TQueryItem> {
    world: &'a mut World,
    filters: QueryFilter,
    result: Vec<Q::Collection>,
    inner_index: usize,
    outer_index: usize,
}

impl<'a, Q: TQueryItem + TFilter> QueryExecutor<'a, Q> {
    pub fn new(world: &'a mut World) -> Self {
        Self {
            world,
            filters: Default::default(),
            result: Default::default(),
            inner_index: 0,
            outer_index: 0,
        }
    }

    pub fn get(&mut self) -> &mut Self {
        Q::filter(&mut self.filters);
        self
    }

    pub fn with_filter<F: TFilter>(&mut self) {
        F::filter(&mut self.filters)
    }

    pub fn execute(&mut self) -> &mut Self {
        unsafe {
            self.result = self.world.entity_tables
                .iter_mut()
                .filter(|t| t.has_signature(&self.filters.signature()))
                .map(|t| Q::get(t))
                .collect();
        }
        self
    }
    pub fn data(&self) -> &Vec<Q::Collection> {
        &self.result
    }
}

impl<'q, Q: TQueryItem> Iterator for QueryExecutor<'q, Q> {
    type Item = <<Q as TQueryItem>::Collection as TCollection>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.outer_index >= self.result.len() {
            None
        } else if self.inner_index >= self.result[self.outer_index].length() {
            self.outer_index += 1;
            self.inner_index = 0;
            self.next()
        } else {
            let result = self.result[self.outer_index].get(self.inner_index);
            self.inner_index += 1;
            Some(result)
        }
    }
}


pub fn test() {
    let init_entity = entity![1  as i32, (1.0 / 2.0) as f32];
    let type_infos: Vec<TypeInfo> = init_entity.iter().map(|c| (**c).type_info()).collect();
    let mut table_one = EntityTable::new(type_infos);
    (0..50).for_each(|n| {
        let mut entity = entity![1 as i32, (n as f32 / 2.0) as f32];
        table_one.add(entity);
    });

    let init_entity = entity![2 as i32, 1  as u8];
    let type_infos: Vec<TypeInfo> = init_entity.iter().map(|c| (**c).type_info()).collect();
    let mut table_two = EntityTable::new(type_infos);
    (0..50).for_each(|n| {
        let mut entity = entity![2 as i32, (1) as u8];
        table_two.add(entity);
    });


    let tables = vec![table_one, table_two];
    let mut world = World::new_vec(tables);

    let mut start: QueryExecutor<(&mut i32, &u8)> = QueryExecutor::new(&mut world);

    let data = start.get().execute();

    for (a, b) in data {
        *a = (*b as i32);
        println!("{}, {}", a, b);
    }
}


