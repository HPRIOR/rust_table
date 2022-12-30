use std::any::TypeId;
use std::collections::HashSet;
use std::ops::Add;
use crate::storage::component::TypeInfo;
use crate::storage::table::EntityTable;
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

fn test(tables: &Vec<EntityTable>, type_infos: HashSet<TypeId>) {
    let mut type_infos: HashSet<TypeId> = HashSet::new();
    let matching_tables: Vec<&EntityTable> =
        tables.iter().filter(|t| t.has_signature(&type_infos)).collect();

    let columnsBox = (&matching_tables).iter().map(|t| t.get::<i32>());
}

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

            let result  = ($( (&matching_tables).iter().map(|t| t.get::<$query_type>()).collect(), )*);

            result
        }
    }
}

fn ret<T>(a: T) -> T {
    // let ti = TypeInfo::of::<T>().type_name.to_string();
    a
}

#[macro_export]
macro_rules! test {
    (($($t: ty),*), $a: expr) => {
        {
            let mut strings: Vec<String> = vec![];
            $(
                strings.push(TypeInfo::of::<$t>().type_name.to_string());
            )*
            strings.iter().for_each(|s| println!("{}", s));
            ($($a as $t,)*)
        }
    }
}
