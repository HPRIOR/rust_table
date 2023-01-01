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

trait TQuery<'a> {
    type Item;

    fn next_item(&mut self) -> Self::Item;
    fn to_trait(self) -> Box<dyn TQuery<'a, Item=Self::Item> + 'a>;
}


impl<'a, T> TQuery<'a> for Query<'a, T> {
    type Item = &'a T;

    fn next_item(&mut self) -> Self::Item {
        self.next().unwrap()
    }

    fn to_trait(self) -> Box<dyn TQuery<'a, Item=Self::Item> + 'a> {
        Box::new(self)
    }
}

struct QueryTuple<'a, T, U>(Query<'a, T>, Query<'a, U>);

impl<'a, T, U> TQuery<'a> for QueryTuple<'a, T, U> {
    type Item = (&'a T, &'a U);

    fn next_item(&mut self) -> Self::Item {
        (self.0.next().unwrap(), self.1.next().unwrap())
    }

    fn to_trait(self) -> Box<dyn TQuery<'a, Item=Self::Item> + 'a> {
        Box::new(self)
    }
}

impl<'a, T, U> TQuery<'a> for (Query<'a, T>, Query<'a, U>) {
    type Item = (&'a T, &'a U);

    fn next_item(&mut self) -> Self::Item {
        (self.0.next().unwrap(), self.1.next().unwrap())
    }

    fn to_trait(self) -> Box<dyn TQuery<'a, Item=Self::Item> + 'a> {
        Box::new(self)
    }
}


fn use_query_interface<'a, Q, C, A>(tables: &'a Vec<EntityTable>) -> Q
    where Q: TQuery<'a, Item=(C, A)> + FromIterator<(&'a [C], &'a [A])>,
          C: Component,
          A: Component

{
    let mut type_ids: HashSet<TypeId> = HashSet::new();
    let type_id1 = TypeInfo::of::<C>().id;
    type_ids.insert(type_id1);

    let matching_tables: Vec<&EntityTable> =
        tables.iter().filter(|t| t.has_signature(&type_ids)).collect();

    // generate queries

    let mut query: Q =
        (&matching_tables)
            .iter()
            .map(|t| (t.get::<C>())).
            zip(
                (&matching_tables)
                    .iter()
                    .map(|t| (t.get::<A>())))
            .collect();

    query
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

impl<'a, T: Component> FromIterator<&'a [T]> for Query<'a, T> {
    fn from_iter<I: IntoIterator<Item=&'a [T]>>(iter: I) -> Self {
        let mut query = Query::new();
        for element in iter {
            query.push(element);
        }
        query
    }
}

impl<'a, T: Component> FromIterator<&'a mut [T]> for Query<'a, T> {
    fn from_iter<I: IntoIterator<Item=&'a mut [T]>>(iter: I) -> Self {
        let mut query = Query::new();
        for element in iter {
            query.push(element);
        }
        query
    }
}

impl<'a, T: Component, U: Component> FromIterator<(&'a [T], &'a [U])> for QueryTuple<'a, T, U> {
    fn from_iter<I: IntoIterator<Item=(&'a [T], &'a [U])>>(iter: I) -> Self {
        let mut query_one = Query::new();
        let mut query_two = Query::new();
        for element in iter {
            query_one.push(element.0);
            query_two.push(element.1);
        }

        QueryTuple(query_one, query_two)
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
