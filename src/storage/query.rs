use crate::storage::column::Column;
use crate::storage::component::{Component, TypeInfo};
use crate::storage::table::EntityTable;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

// PROBLEM: It's not possible to alias structs with different generic type parameters. This current
// approach therefore is not possible

// TODO: it would be nice to create a query interface that could return a tuple of arguments from a
// table in one iteration e.g. Query<'a, T, U> -> for (t, u) in query {}
//

trait TQuery {
    type Item<'a>;

    fn next_item<'a>(&mut self) -> Self::Item<'a>;
}

trait TFetcher<'a, T>{
    fn new(query: &'a mut T) -> Box<dyn Self>;
}

struct Fetcher<'a, T: TQuery> {
    query: &'a mut T,
}

impl<'a, T: TQuery> TFetcher<'a, T> for Fetcher<'a, T> {
    fn new(query: &'a mut T) -> Self {
        Self {
            query
        }
    }
}


impl<'a, T: TQuery> Iterator for Fetcher<'a, T> {
    type Item = <T as TQuery>::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let query = &mut self.query;
        Some(query.next_item())
    }
}

impl<'a, T> TQuery for Query<'a, T> {
    type Item<'q> = &'a T;

    fn next_item<'q>(&mut self) -> Self::Item<'q> {
        self.next().unwrap()
    }
}

struct QueryTuple<'a, T, U>((Query<'a, T>, Query<'a, U>));


impl<'a, T, U> TQuery for QueryTuple<'a, T, U> {
    type Item<'q> = (&'a T, &'a U);

    fn next_item<'q>(&mut self) -> Self::Item<'q> {
        (self.0.0.next().unwrap(), self.0.1.next().unwrap())
    }
}

fn use_query_interface<'a, T: TQuery>(tables: Vec<EntityTable>) -> Box<dyn TFetcher<'a, T>> {
    let mut type_ids: HashSet<TypeId> = HashSet::new();
    let type_id1 = TypeInfo::of::<u32>().id;
    type_ids.insert(type_id1);

    let matching_tables: Vec<&EntityTable> =
        tables.iter().filter(|t| t.has_signature(&type_ids)).collect();

    // generate queries
    let mut query_one: Query<i32> =
        (&matching_tables).iter().map(|t| (t.get::<i32>())).collect();


    let fetcher = Fetcher::new(&mut query_one);
    Box::new(fetcher)
}

pub struct Query<'a, T> {
    data: Vec<&'a [T]>,
    outer_index: usize,
    inner_index: usize,
}

impl<'a, T> Query<'a, T> {
    pub fn new() -> Self {
        Self {
            data: vec![],
            outer_index: 0,
            inner_index: 0,
        }
    }

    pub fn push(&mut self, element: &'a [T]) {
        self.data.push(element);
    }
}

impl<'a, T> FromIterator<&'a [T]> for Query<'a, T> {
    fn from_iter<I: IntoIterator<Item=&'a [T]>>(iter: I) -> Self {
        let mut query = Query::new();
        for element in iter {
            query.push(element);
        }
        query
    }
}

impl<'a, T> Iterator for Query<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let data = &self.data;

        if self.outer_index >= data.len() {
            None
        } else if self.inner_index >= self.data[self.outer_index].len() {
            self.inner_index = 0;
            self.outer_index += 1;
            self.next()
        } else {
            let result = &data[self.outer_index][self.inner_index];
            self.inner_index += 1;
            Some(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::query::Query;

    #[test]
    fn query_can_be_created_from_nested_iterable() {
        fn get_int_array<'a>() -> &'a [i32] {
            &[1, 2, 3, 4]
        }
        let query: Query<i32> = (0..3).into_iter().map(|a| get_int_array()).collect();
    }

    #[test]
    fn query_can_be_iterated_over() {
        let query: Query<i32> = vec![&[0, 1, 2, 3] as &[i32], &[4, 5, 6, 7] as &[i32]]
            .into_iter()
            .collect();

        for (i, elem) in query.enumerate() {
            assert_eq!(i, *elem as usize);
        }
    }
}
