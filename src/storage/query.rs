use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use crate::storage::column::Column;
use crate::storage::component::{Component, TypeInfo};
use crate::storage::table::EntityTable;

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

struct GenFunBuilder;

impl GenFunBuilder {
    fn build<T: Component>() -> fn(T) -> T {
        todo!()
    }
}

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



