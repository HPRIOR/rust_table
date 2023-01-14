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
    type Collection<'a>: TCollection;
    type Item<'a>;

    fn get(table: &EntityTable) -> Self::Collection<'_>;
    fn get_at(collection: Self::Collection<'_>, n: usize) -> Self::Item<'_>;
}

pub trait TCollection {
    type Item;
    fn get(&mut self, n: usize) -> Self::Item; // Item must live as long as the
    fn length(&self) -> usize;
}

pub trait TFilter {
    fn filter(query_filter: &mut QueryFilter);
}


// -> Base Implementations <- //

impl<'a, T: Component> TQueryItem for &'a T {
    type Collection<'b> = &'b [T];
    type Item<'b> = &'b T;


    fn get(table: &EntityTable) -> Self::Collection<'_> {
        table.get::<T>()
    }

    fn get_at(collection: Self::Collection<'_>, n: usize) -> Self::Item<'_> {
        &collection[n]
    }
}


impl<'a, T: Component> TFilter for &'a T {
    fn filter(query_filter: &mut QueryFilter) {
        let ti = TypeInfo::of::<T>().id;
        query_filter.included.insert(ti);
    }
}

impl<'a, T: Component> TCollection for &'a [T] {
    type Item = &'a T;

    fn get<'b>(&mut self, n: usize) -> Self::Item {
        &self[n]
    }

    fn length(&self) -> usize {
        self.len()
    }
}


// -> Recursive tuple definitions <- //
// todo: make macro for all tuples

impl<A: TQueryItem, B: TQueryItem> TQueryItem for (A, B) {
    type Collection<'a> = (
        A::Collection<'a>,
        B::Collection<'a>
    );
    type Item<'a> = (A::Item<'a>, B::Item<'a>);

    fn get(table: &EntityTable) -> Self::Collection<'_> {
        (A::get(&table), B::get(&table))
    }

    fn get_at(collection: Self::Collection<'_>, n: usize) -> Self::Item<'_> {
        (A::get_at(collection.0, n), B::get_at(collection.1, n))
    }
}

impl<'a, A: TCollection, B: TCollection> TCollection for (A, B) {
    type Item = (A::Item, B::Item);

    fn get<'b>(&mut self, n: usize) -> Self::Item {
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
    world: &'a World,
    filters: QueryFilter,
    result: Vec<Q::Collection<'a>>,
    inner_index: usize,
    outer_index: usize,
}

impl<'a, Q: TQueryItem + TFilter> QueryExecutor<'a, Q> {
    pub fn new(world: &'a World) -> Self {
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
        self.result = self.world.entity_tables
            .iter()
            .filter(|t| t.has_signature(&self.filters.signature()))
            .map(|t| Q::get(t))
            .collect();
        self
    }
    pub fn data(&self) -> &Vec<Q::Collection<'a>> {
        &self.result
    }
}

impl<'q, Q: TQueryItem> Iterator for QueryExecutor<'q, Q> {
    type Item = <<Q as TQueryItem>::Collection<'q> as TCollection>::Item;

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
    let world = World::new_vec(tables);

    let mut start: QueryExecutor<(&i32, &u8)> = QueryExecutor::new(&world);

    let data = start.get().execute();

    for (a, b) in data {
        println!("{}, {}", a, b);
    }
}


