#![allow(unused)]
mod ecs;
mod storage;

use crate::storage::component::Component;
use storage::component::TypeInfo;
use storage::table::EntityTable;

#[derive(Debug)]
struct Position {
    x: i32,
    y: String,
}


fn main() {
    // let mut entity = vec![12.as_component(), "as component".as_component()];
    let mut entity1 = entity![1_u8, 5_u8];
    let mut entity2 = entity![2_u8, 6_u8];
    let mut entity3 = entity![3_u8, 7_u8];
    let mut entity4 = entity![4_u8, 8_u8];

    let type_infos: Vec<TypeInfo> = entity1.iter().map(|c| (**c).type_info()).collect();

    let mut table = EntityTable::new(type_infos);

    table.add(entity1);
    table.add(entity2);
    table.add(entity3);
    table.add(entity4);

    let tables  = vec![table];

    query!(tables => with (i32, &str));

    // let comps: &[i32] = table.columns[0].get_components();
    // let column1 = &table.columns[0];
    // let column2 = &table.columns[1];
    //
    // println!("{:#?}", column1.get_column::<u8>());
    // println!("{:#?}", column2.get_column::<u8>());
    //
    // println!("{:#?}", table);
}
