use crate::storage::column::Column;
use crate::storage::component::{Component, TypeInfo};
use crate::storage::table::EntityTable;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;


// TODO: it would be nice to create a query interface that could return a tuple of arguments from a
// table in one iteration e.g. Query<'a, T, U> -> for (t, u) in query {}

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
    fn from_iter<I: IntoIterator<Item = &'a [T]>>(iter: I) -> Self {
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
pub struct QueryTuple<'a, T, U> {
    data: Vec<(&'a [T], &'a [U])>,
    outer_index: usize,
    inner_index: usize,
}
impl<'a, T, U> QueryTuple<'a, T, U> {
    pub fn new() -> Self {
        Self {
            data: vec![],
            outer_index: 0,
            inner_index: 0,
        }
    }

    pub fn push(&mut self, element: (&'a [T], &'a [U])) {
        self.data.push(element);
    }
}

impl<'a, T, U> FromIterator<(&'a [T],&'a [U])> for QueryTuple<'a, T, U> {
    fn from_iter<I: IntoIterator<Item = (&'a [T],&'a [U])>>(iter: I) -> Self {
        let mut query = QueryTuple::new();
        for element in iter {
            query.push(element);
        }
        query
    }
}

impl<'a, T, U> Iterator for QueryTuple<'a, T, U> {
    type Item = (&'a T, &'a U);

    fn next(&mut self) -> Option<Self::Item> {
        let data = &self.data;

        if self.outer_index >= data.len() {
            None
        } else if self.inner_index >= data[self.outer_index].0.len() {
            self.inner_index = 0;
            self.outer_index += 1;
            self.next()
        } else {
            let outer = &data[self.outer_index];
            let result = (&outer.0[self.inner_index] as &T, &outer.1[self.inner_index] as &U);
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
