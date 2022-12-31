#![allow(unused)]
extern crate core;

mod ecs;
mod storage;

use std::marker::PhantomData;
use crate::storage::component::Component;
use crate::storage::component::TypeInfo;
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
    let init_entity = entity![1 + 1 as i32, (1 / 2) as f32];
    let type_infos: Vec<TypeInfo> = init_entity.iter().map(|c| (**c).type_info()).collect();
    let mut table = EntityTable::new(type_infos);
    (0..10000).for_each(|n| {
        let mut entity = entity![n + 1 as i32, (n / 2) as f32];
        table.add(entity);
    });

    let tables = vec![table];
    // create_query!(A, B);
    // let query: Query<i32, f32> = query!(tables => with (i32, f32));
    //
    // for (a, b) in query {
    //     println!("{}, {}", a, b)
    // }


}
