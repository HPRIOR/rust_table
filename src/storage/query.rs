use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::path::Iter;
use crate::ecs::World;
use crate::entity;
use crate::storage::component::TypeInfo;

use super::{component::Component, table::EntityTable};

// random notes:
// TODO: take in 'world' from query. Come up with some solution that can filter the world for the
// relevant tables for a given query and pass these onto the fetch and query methods. The query result
// should also accept a nested array to account for the query across multiple tables

// Design decision: should query be responsible for filtering out which tables in the world should
// or should the tables be passed down to the query already filtered?
// - The top level function in World takes a generic Query as a param. There can only be one implementation
// for this for &T and &mut T, so you'd lose the ability to define different types of queries based on
// the simple type definition. Favouring making table filtering the responsibility of the Query


// Possible implementation
// QueryExecutor contains a mutable data structure which can be manipulated by each Query implementation
// for example an array of indexes which will be used to filter out the entity table list eventually
// passed to the query.

// This could be inefficient as each query param would have their member called, and possibly some data be stored
// for later queries to observe and modify.
// E.g. possible algo for Query<(&i32, &u8, Without<bool>)>.
// there is list of usize in QueryExecutor.
// For &i32 and &u8 each table is checked for having this data, and indexes are added to list
// For without indexes are added to another list
// The difference is what is used to generate the final table. Wouldn't be so bad if they were
// hashmaps of each table

// Query<(&i32, &u8, Without<bool>)> This interface is more complicated than it's worth. If Without
// implements Query then it must return some value which will be added to the returned tuple. Instead,
// stick with the current implementation and use a builder pattern to modify query before it's executed:
// let query =
//      QueryExecutor<(A, B)>::new()
//          .get::<(A, B)>()        // registers call back for execute
//          .except::<C>()
//          .some_filter::<T>()
//          .some_other_filter::<U>()
//          .execute();

// would still be worth splitting the Filter trait out from query so subsequent filter only methods
// can be used without attachment to query trait (interface seg principle?)



// After this is implemented, there needs to be a way of iterating over the nested sequence as if
// it were a homogenous sequence
// This is difficult as the Iterator cannot be implemented on tuples - ideal I would just write
// an iterator for QueryResult, and then another iterator on successive generic tuple combinations
// where T implements Query result. But this is not possible unless you wrap the tuple inside of a struct.
// A new struct would need to be created for every combination of tuple, and an iterator implemented for that.

// one route might be to implement iterator directly on the QueryExecutor,
// or generate another Iterator implementation of the QueryExecutor results

// After this mutable queries need to be made. Hopefully this will just be another impl if the traits
// which are currently working for borrows.

// Then different types of queries should be allowed.
// E.g.
// Get type with type but not y
//  Get type with othertype
// get type with othertype but not type


// concurrency and efficiencies
// concurrent iteration for borrows
// fast table lookups for quires (hashing and signatures)

// Entities
// lookup and removal of entities
// removing components from entities, reorganising tables as a result, dynamic graph implementation
// to find relevant tables to move entities in and out of


trait QueryResult {
    type Item;
    fn next_item(&mut self) -> Option<Self::Item>;
}

impl<'a, T: Component> From<Vec<&'a [T]>> for ReadQueryResult<'a, T> {
    fn from(value: Vec<&'a [T]>) -> Self {
        ReadQueryResult::new(value)
    }
}

struct ReadQueryResult<'a, T: Component> {
    results: Vec<&'a [T]>,
    index_inner: usize,
    index_outer: usize,
}


impl<'a, T: Component> QueryResult for ReadQueryResult<'a, T> {
    type Item = &'a T;

    fn next_item(&mut self) -> Option<Self::Item> {
        if self.index_outer >= self.results.len() {
            None
        } else if self.index_inner >= self.results[self.index_outer].len() {
            self.index_inner = 0;
            self.index_outer += 1;
            self.next_item()
        } else {
            let result = Some(&self.results[self.index_outer][self.index_inner]);
            self.index_inner += 1;
            result
        }
    }
}

impl<'a, T: Component> ReadQueryResult<'a, T> {
    fn new(results: Vec<&'a [T]>) -> Self {
        Self { results, index_inner: Default::default(), index_outer: Default::default() }
    }
}

impl<'a, T: Component> Iterator for ReadQueryResult<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_item()
    }
}

trait QueryIterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}


impl<A: QueryResult, B: QueryResult> QueryIterator for (A, B) {
    type Item = (A::Item, B::Item);

    fn next(&mut self) -> Option<Self::Item> {
        Some((self.0.next_item()?, self.1.next_item()?))
    }
}


// This is problematic. In order to define more tuples, more structs would need to
// be defined on each tuple combination.
struct QueryResultTuple<A: QueryResult, B: QueryResult>((A, B));

impl<A: QueryResult, B: QueryResult> From<(A, B)> for QueryResultTuple<A, B> {
    fn from(value: (A, B)) -> Self {
        QueryResultTuple(value)
    }
}


impl<A: QueryResult, B: QueryResult> Iterator for QueryResultTuple<A, B> {
    type Item = (A::Item, B::Item);

    fn next(&mut self) -> Option<Self::Item> {
        Some((self.0.0.next_item()?, self.0.1.next_item()?))
    }
}


pub trait Query
{
    // same as self
    type Item<'a>;

    type Fetch: Fetch;

    fn get<'a>(fetch: &Self::Fetch, table: &'a EntityTable) -> Self::Item<'a>;
    fn filter(query_filter: &mut QueryFilter);
}

pub trait Fetch {
    type Item<'a>;

    fn fetch(table: &EntityTable) -> Self::Item<'_>;
    fn new() -> Self;
}

pub struct FetchRead<T> (PhantomData<T>);

impl<T: Component> Fetch for FetchRead<T> {
    type Item<'a> = &'a [T];

    /// Assumes tables have been checked for the existence of T before executing
    fn fetch(table: &EntityTable) -> Self::Item<'_> {
        table.get::<T>()
    }

    fn new() -> Self { Self { 0: Default::default() } }
}


// Tuple implementation for Fetch - todo create macro
// Recursive definition for tuples. Fetch is implemented on tuples of Fetch, calling the base on each
// one to derive a tuple result.
impl<A: Fetch, B: Fetch> Fetch for (A, B) {
    type Item<'a> = (A::Item<'a>, B::Item<'a>);

    fn fetch(table: &EntityTable) -> Self::Item<'_> {
        (A::fetch(&table), B::fetch(&table))
    }

    fn new() -> Self {
        (A::new(), B::new())
    }
}

impl<A: Query, B: Query> Query for (A, B) {
    type Item<'a> = (
        <<A as Query>::Fetch as Fetch>::Item<'a>,
        <<B as Query>::Fetch as Fetch>::Item<'a>
    );

    type Fetch = (A::Fetch, B::Fetch);

    fn get<'a>(fetch: &Self::Fetch, table: &'a EntityTable) -> Self::Item<'a> {
        Self::Fetch::fetch(table)
    }

    fn filter(query_filter: &mut QueryFilter) {
        A::filter(query_filter);
        B::filter(query_filter);
    }
}


impl<'a, T: Component> Query for &'a T {
    type Item<'b> = &'b [T];

    type Fetch = FetchRead<T>;

    fn get<'b>(fetch: &Self::Fetch, table: &'b EntityTable) -> Self::Item<'b> {
        Self::Fetch::fetch(table)
    }

    fn filter(query_filter: &mut QueryFilter) {
        let ti = TypeInfo::of::<T>().id;
        query_filter.included.insert(ti);
    }
}

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

pub struct QueryExecutor<'a, Q: Query> {
    world: &'a World,
    filters: QueryFilter,
    _marker: PhantomData<Q>,
}

impl<'a, Q: Query> QueryExecutor<'a, Q> {
    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            filters: Default::default(),
            _marker: PhantomData::default(),
        }
    }

    /// Provides Fetch and Query abstractions with required world data. The implementation
    /// of Query and Fetch depends on the type that's implemented Query (e.g. &T -> Borrow; &mut T -> mutable borrow)
    /// Hence, query can be made using the types passed to this generic function
    pub fn execute(&mut self) -> Vec<<Q as Query>::Item<'_>> {
        let fetcher = Q::Fetch::new();

        // modify query filters
        Q::filter(&mut self.filters);
        self.world.entity_tables
            .iter()
            .filter(|t| t.has_signature(&self.filters.signature()))
            .map(|t| Q::get(&fetcher, t))
            .collect()
    }
}

impl<'q, Q: Query> Iterator for QueryExecutor<'q, Q> {
    type Item = Q::Item<'q>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}


pub fn test() {
    let init_entity = entity![1 + 1 as i32, (1 / 2) as f32];
    let type_infos: Vec<TypeInfo> = init_entity.iter().map(|c| (**c).type_info()).collect();
    let mut table_one = EntityTable::new(type_infos);
    (0..50).for_each(|n| {
        let mut entity = entity![n + 1 as i32, (n / 2) as f32];
        table_one.add(entity);
    });

    let init_entity = entity![1 + 1 as i32, 1  as u8];
    let type_infos: Vec<TypeInfo> = init_entity.iter().map(|c| (**c).type_info()).collect();
    let mut table_two = EntityTable::new(type_infos);
    (0..50).for_each(|n| {
        let mut entity = entity![n * 20 as i32, (1) as u8];
        table_two.add(entity);
    });


    let tables = vec![table_one, table_two];
    let world = World::new_vec(tables);

    let mut start: QueryExecutor<(&i32, &f32)> = QueryExecutor::new(&world);

    let data = start.execute();

    for (a,b) in data {
        for i in a {
            println!("{}", i)
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::storage::query::{QueryResult, QueryResultTuple, ReadQueryResult};

    #[test]
    fn query_result_can_iter_over_nested() {
        let result: Vec<&[i32]> = vec![&[1, 2, 3, 4], &[5, 6, 7, 8], &[9, 10, 11, 12]];
        let expected = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let mut query_result = ReadQueryResult::new(result);
        query_result.into_iter().zip(expected).for_each(|(a, b)| assert_eq!(a, b));
    }

    #[test]
    fn query_result_can_iter_over_nested_tuple() {
        let result: QueryResultTuple<ReadQueryResult<i32>, ReadQueryResult<i32>> =
            QueryResultTuple(
                (ReadQueryResult::new(vec![&[1, 2, 3, 4], &[5, 6, 7, 8], &[9, 10, 11, 12]]),
                 ReadQueryResult::new(vec![&[1, 2, 3, 4], &[5, 6, 7, 8], &[9, 10, 11, 12]]))
            );

        for (a, b) in result {
            println!("{}, {}", a, b)
        }
    }
}
