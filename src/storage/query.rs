use std::marker::PhantomData;
use std::path::Iter;
use crate::entity;
use crate::storage::component::TypeInfo;

use super::{component::Component, table::EntityTable};

// TODO: current query will return a single array slice from a single table.
// In a real case several tables would be queries for matching types, and a 2d array would be returned
// create an abstraction that would allow for the iterating over multiple arrays of a particular type
// and for tuples of various types
// create macros too

trait QueryResult {
    type Item;
    fn next_item(&mut self) -> Option<Self::Item>;
}

struct ReadQueryResult<'a, T: Component> {
    results: &'a [&'a [T]],
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
    fn new(results: &'a [&'a [T]]) -> Self {
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


// This is problematic. In order to define more tuples, more structs would need to be defined on each tuple combination.
struct QueryResultTuple<A: QueryResult, B: QueryResult>((A, B));

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
}

pub trait Fetch {
    type Item<'a>;

    fn fetch(table: &EntityTable) -> Self::Item<'_>;
    fn new() -> Self;
}

pub struct FetchRead<T> (PhantomData<T>);

impl<T: Component> Fetch for FetchRead<T> {
    type Item<'a> = &'a [T];

    // this will need to change to take a reference to World
    // and query across multiple tables
    fn fetch(table: &EntityTable) -> Self::Item<'_> {
        if table.has::<T>() {
            println!("table found with type {}", TypeInfo::of::<T>().type_name);
            table.get::<T>()
        } else {
            panic!("table not found with type {}", TypeInfo::of::<T>().type_name);
        }
    }

    fn new() -> Self { Self { 0: Default::default() } }
}

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
}


impl<'a, T: Component> Query for &'a T {
    type Item<'b> = &'b [T];

    type Fetch = FetchRead<T>;

    fn get<'b>(fetch: &Self::Fetch, table: &'b EntityTable) -> Self::Item<'b> {
        Self::Fetch::fetch(table)
    }
}

pub struct QueryExecutor<'a, Q: Query> {
    tables: &'a Vec<EntityTable>,
    data: Option<&'a [Q::Item<'a>]>,
    _marker: PhantomData<Q>,
}

impl<'a, Q: Query> QueryExecutor<'a, Q> {
    pub fn new(tables: &'a Vec<EntityTable>) -> Self {
        Self {
            tables,
            data: None,
            _marker: PhantomData::default(),
        }
    }

    pub fn execute(&self) -> <Q as Query>::Item<'_> {
        let fetcher = Q::Fetch::new();
        // no logic to fetch the correct data from a range of queries yet
        let result = Q::get(&fetcher, &self.tables[0]);
        result
    }
    pub fn execute_on_self(&mut self) {
        let fetcher = Q::Fetch::new();
        // no logic to fetch the correct data from a range of queries yet
        let result = Q::get(&fetcher, &self.tables[0]);
        todo!()
        // self.data = Some(result);
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
    let mut table = EntityTable::new(type_infos);
    (0..10000).for_each(|n| {
        let mut entity = entity![n + 1 as i32, (n / 2) as f32];
        table.add(entity);
    });

    let tables = vec![table];

    let start: QueryExecutor<(&i32, &f32)> = QueryExecutor::new(&tables);
    let data = start.execute();

    let v: Vec<(&i32, &f32)> = data.0.iter().zip(data.1.iter()).collect();

    for (a, b) in v {
        println!("{},{}", a, b)
    }
}


#[cfg(test)]
mod tests {
    use crate::storage::query::{QueryResult, QueryResultTuple, ReadQueryResult};

    #[test]
    fn query_result_can_iter_over_nested() {
        let result: &[&[i32]] = &[&[1, 2, 3, 4], &[5, 6, 7, 8], &[9, 10, 11, 12]];
        let expected = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let mut query_result = ReadQueryResult::new(result);
        query_result.into_iter().zip(expected).for_each(|(a, b)| assert_eq!(a, b));
    }

    #[test]
    fn query_result_can_iter_over_nested_tuple() {
        let result: QueryResultTuple<ReadQueryResult<i32>, ReadQueryResult<i32>> =
            QueryResultTuple(
                (ReadQueryResult::new(&[&[1, 2, 3, 4], &[5, 6, 7, 8], &[9, 10, 11, 12]]),
                 ReadQueryResult::new(&[&[1, 2, 3, 4], &[5, 6, 7, 8], &[9, 10, 11, 12]]))
            );

        for (a, b) in result {
            println!("{}, {}", a, b)
        }
    }
}
