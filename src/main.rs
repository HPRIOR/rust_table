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

#[derive(Debug)]
struct Position {
    x: i32,
    y: String,
}

fn main() {
    // let mut entity = vec![12.as_component(), "as component".as_component()];
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

    // let a: (u32, u8) = test!((u32, u8), 1);
    // println!("{:?}", a);
    //
    let (ones, twos) = query!(tables => with (u8, i32));



    for one in ones{
        println!("{}", one)
    }

    for two in twos{

        println!("{}", two)
    }

    create_query!(A, B);

    let test_query: TestQuery<u32, u32>;


    // let comps: &[i32] = table.columns[0].get_components();
    // let column1 = &table.columns[0];
    // let column2 = &table.columns[1];
    //
    // println!("{:#?}", column1.get_column::<u8>());
    // println!("{:#?}", column2.get_column::<u8>());
    //
    // println!("{:#?}", table);
}
