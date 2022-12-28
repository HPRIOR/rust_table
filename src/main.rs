#![allow(unused)]
mod storage;
mod ecs;

use storage::table::EntityTable;
use storage::component::TypeInfo;
use crate::storage::component::Component;

#[derive(Debug)]
struct Position {
    x: i32,
    y: String,
}

fn main() {
    // let mut entity = vec![12.as_component(), "as component".as_component()];
    let mut entity1 = vec![1_u8.as_component(), 5_u8.as_component()];
    let mut entity2 = vec![2_u8.as_component(), 6_u8.as_component()];
    let mut entity3 = vec![3_u8.as_component(), 7_u8.as_component()];
    let mut entity4 = vec![4_u8.as_component(), 8_u8.as_component()];
    entity1.sort_by(|a, b| a.type_info().partial_cmp(&b.type_info()).unwrap());
    entity2.sort_by(|a, b| a.type_info().partial_cmp(&b.type_info()).unwrap());
    entity3.sort_by(|a, b| a.type_info().partial_cmp(&b.type_info()).unwrap());
    entity3.sort_by(|a, b| a.type_info().partial_cmp(&b.type_info()).unwrap());

    let type_infos: Vec<TypeInfo> = entity1.iter().map(|c| (**c).type_info()).collect();

    let mut table = EntityTable::new(type_infos);

    table.add(entity1);
    table.add(entity2);
    table.add(entity3);
    table.add(entity4);

    // let comps: &[i32] = table.columns[0].get_components();
    let column1 = &table.columns[0];
    let column2 = &table.columns[1];
    println!("{:#?}", column1.get_column::<u8>());
    println!("{:#?}", column2.get_column::<u8>());

    println!("{:#?}", table);
}
