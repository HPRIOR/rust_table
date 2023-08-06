use crate::storage::component::{Component, TypeInfo};
use count_macro::count;
use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::Add;

#[macro_export]
macro_rules! entity {
    ($($x:expr),*) => {
        {
            // let mut temp_vec: Vec<Box<dyn Component>> = Vec::new();
            // $(
            //     temp_vec.push($x.to_component_ref());
            // )*
            let mut temp_vec: Vec<Box<dyn Component>> = vec![
            $(
                $x.to_component_ref(),
            )* ];

            temp_vec.sort_by(|a, b| a.type_info().id.partial_cmp(&b.type_info().id).unwrap());

            temp_vec
        }
    };
}

// #[macro_export]
// macro_rules! create_query {
//     ($($t:ident),*) => {
//         pub struct Query<'a, $($t,)*> {
//             data: Vec<($(&'a [$t]),*)>,
//             outer_index: usize,
//             inner_index: usize,
//         }
//
//         impl<'a, $($t,)*> Query<'a, $($t,)*> {
//             pub fn new() -> Self {
//                 Self {
//                     data: vec![],
//                     outer_index: 0,
//                     inner_index: 0,
//                 }
//             }
//
//             pub fn push(&mut self, element: ($(&'a [$t]),*)) {
//                 self.data.push(element);
//             }
//         }
//         impl<'a, $($t,)*> FromIterator<($(&'a [$t]),*)> for Query<'a, $($t,)*> {
//             fn from_iter<I: IntoIterator<Item = ($(&'a [$t]),*)>>(iter: I) -> Self {
//                 let mut query = Query::new();
//                 for element in iter {
//                     query.push(element);
//                 }
//                 query
//             }
//         }
//
//         impl<'a, $($t,)*> Iterator for Query<'a, $($t,)*> {
//             type Item = ($(&'a $t),*);
//
//             fn next(&mut self) -> Option<Self::Item> {
//                 let data = &self.data;
//
//                 if self.outer_index >= data.len() {
//                     None
//                 } else if self.inner_index >= data[self.outer_index].0.len() {
//                     self.inner_index = 0;
//                     self.outer_index += 1;
//                     self.next()
//                 } else {
//                     let outer = &data[self.outer_index];
//                     let result = count!{
//                         ($(&outer._int_[self.inner_index] as &$t,)*)
//                     };
//                     self.inner_index += 1;
//                     Some(result)
//                 }
//             }
//         }
//     };
// }

// create_query!(A, B);

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


            // let result: ($(Query<$query_type>,)*)  =
            //     ($( (&matching_tables).iter().map(|t| t.get::<$query_type>()).collect(), )*);
                let result: Query<$(($query_type),)*>  =
                    (&matching_tables).iter().map(|t|{ ($((t.get::<$query_type>()),)*)}).collect();

            result
        }
    }
}
