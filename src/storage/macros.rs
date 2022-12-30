use crate::storage::component::TypeInfo;
use crate::storage::table::EntityTable;
use crate::storage::query::Query;
use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::Add;
#[macro_export]
macro_rules! entity {
    ($($x:expr),*) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.as_component());
            )*

            temp_vec.sort_by(|a, b| a.type_info().partial_cmp(&b.type_info()).unwrap());
            temp_vec
        }
    };
}

#[macro_export]
macro_rules! test {
    () => {
        pub struct TestStruct;
        impl TestStruct {
            pub fn new() -> Self{
                Self
            }
        }
    };
}


#[macro_export]
macro_rules! create_query {
    ($($t:ident),*)=> {

        pub struct TestQuery<$($t,)*> {
            vec: Vec<($($t,)*)>,
            outer_index: usize,
            inner_index: usize,
        }
    };
}

/// POC macro, currently just collects data from exactly matching tables and returns list of lists
/// TODO:
/// Instead of returning nested data structure, return an Iterable which abstracts the nesting
/// Iterable<&[type]> -> .collect -> Query<type> -> for i in Query do something
#[macro_export]
macro_rules! query {
    ($tables: expr => with ($($query_type:ty),*)) => {
        {
            let mut type_ids: HashSet<TypeId> = HashSet::new();
            $(
                let type_id = TypeInfo::of::<$query_type>().id;
                type_ids.insert(type_id);
            )*

            let matching_tables: Vec<&EntityTable> =
                $tables.iter().filter(|t| t.has_signature(&type_ids)).collect();


            let result: ($(Query<$query_type>,)*)  =
                ($( (&matching_tables).iter().map(|t| t.get::<$query_type>()).collect(), )*);

            result
        }
    }
}
