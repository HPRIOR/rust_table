#![allow(unused)]
extern crate core;

mod ecs;
mod storage;

use std::marker::PhantomData;
use crate::storage::component::Component;
use crate::storage::component::TypeInfo;
use crate::storage::query::Query;
use crate::storage::table::EntityTable;
use std::any::TypeId;
use std::collections::HashSet;
use count_macro::count;

#[derive(Debug)]
struct Position {
    x: i32,
    y: String,
}

fn main() {
    let mut entity1 = entity![1_u8, 5_i32];
    let mut entity2 = entity![2_u8, 6_i32];
    let mut entity3 = entity![3_u8, 7_i32];
    let mut entity4 = entity![4_u8, 8_i32];

    let type_infos: Vec<TypeInfo> = entity1.iter().map(|c| (**c).type_info()).collect();

    let mut table = EntityTable::new(type_infos);

    table.add(entity1);
    table.add(entity2);
    table.add(entity3);
    table.add(entity4);

    let tables = vec![table];
    create_query!(A, B);
    let query: QueryTuple<u8, i32> = query!(tables => with (u8, i32));

    for (a, b) in query{
        println!("{}, {}", a, b)
    }




    // let comps: &[i32] = table.columns[0].get_components();
    // let column1 = &table.columns[0];
    // let column2 = &table.columns[1];
    //
    // println!("{:#?}", column1.get_column::<u8>());
    // println!("{:#?}", column2.get_column::<u8>());
    //
    // println!("{:#?}", table);
}
