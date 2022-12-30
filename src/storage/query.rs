use crate::storage::column::Column;
use crate::storage::component::{Component, TypeInfo};
use crate::storage::table::EntityTable;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

// structure which contains a reference to the world (all tables) and a list of all matching tables
// The structure is builder type, which subsequent queries add table id to 'matched', 'excluded' sets
// Final Query intersects, minuses, etc the matched and excluded sets to produce a final result
// tables are returned by using this list of ids
// somehow the tables columns are returned with their types fully formed

// seems like a macro is the way to go, something like:
// query![
//      world with (included types) without (excluded)
// ]
// This would generate a query based on the types with the query builder
// the macro would expand the types at the end to produce a tuple of columns which can be iterated
// over with the correct types by calling

struct QueryBuilder<'a> {
    table_map: HashMap<u32, &'a EntityTable>,
    include: HashSet<u32>,
    exclude: HashSet<u32>,
    funcs: fn(Box<dyn Component>) -> dyn Component,
}

impl<'a> QueryBuilder<'a> {
    fn with<T: Component>(self) -> QueryBuilder<'a> {
        todo!()
    }

    fn excluding<T: Component>(self) -> QueryBuilder<'a> {
        todo!()
    }

    fn query(self) -> Vec<Column> {
        todo!()
    }
}

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

    #[test]
    fn query_can_accept_tuples() {
        let query: Query<(i32, u8)> = vec![
            &[(0_i32, 1_u8), (1_i32, 2_u8), (2_i32, 3_u8), (3_i32, 4_u8)] as &[(i32, u8)],
            &[(5_i32, 6_u8), (6_i32, 7_u8), (7_i32, 8_u8), (8_i32, 9_u8)] as &[(i32, u8)],
        ]
        .into_iter()
        .collect();
    }
    // #[test]
    // fn test(){
    //     TestStruct::new();
    // }

}
