use bit_set::BitSet;

use crate::storage::component::TypeInfo;
use crate::storage::query::{QueryInit, TQueryItem, TTableKey};
use crate::storage::{component::Component, table::EntityTable};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

/*
 * Contains entities stored in tables.
 * Various mechanisms are used to keep track of tables and entities.
 *
 * Each table has an ID, tables are stored in a Map: TableId -> Table.
 * The table each entity resides in is stored as a Map: EntityId -> TableId
 * */

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum TableId {
    Value(u64),
}

#[derive(Default, Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct TableIdGen {
    current: u64,
}

impl TableIdGen {
    fn next(&mut self) -> TableId {
        let result = self.current;
        self.current += 1;
        TableId::Value(result)
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum EntityId {
    Value(u64),
}

#[derive(Default, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct EntityIdGen {
    current: u64,
}

impl EntityIdGen {
    fn next(&mut self) -> EntityId {
        let result = self.current;
        self.current += 1;
        EntityId::Value(result)
    }
}

pub struct World {
    table_id_gen: TableIdGen,
    entity_id_gen: EntityIdGen,

    // used to generate bitmap
    pub type_id_index: HashMap<TypeId, usize>,
    entity_id_to_table_id: HashMap<EntityId, TableId>,
    pub table_ids_with_signature: HashMap<BitSet, TableId>,
    pub tables: HashMap<TableId, EntityTable>,
}

fn gen_typeid_map() -> HashMap<TypeId, usize> {
    let mut map = HashMap::<TypeId, usize>::new();
    map.insert(TypeId::of::<i8>(), 0);
    map.insert(TypeId::of::<i16>(), 1);
    map.insert(TypeId::of::<i32>(), 2);
    map.insert(TypeId::of::<i64>(), 3);
    map.insert(TypeId::of::<i128>(), 4);
    map.insert(TypeId::of::<isize>(), 5);
    map.insert(TypeId::of::<u8>(), 6);
    map.insert(TypeId::of::<u16>(), 7);
    map.insert(TypeId::of::<u32>(), 8);
    map.insert(TypeId::of::<u64>(), 9);
    map.insert(TypeId::of::<u128>(), 10);
    map.insert(TypeId::of::<usize>(), 11);
    map.insert(TypeId::of::<f32>(), 12);
    map.insert(TypeId::of::<f64>(), 13);
    map.insert(TypeId::of::<bool>(), 14);
    map.insert(TypeId::of::<char>(), 15);
    map.insert(TypeId::of::<String>(), 16);
    map.insert(TypeId::of::<&str>(), 17);
    map
}

impl World {
    pub fn new() -> Self {
        Self {
            table_id_gen: Default::default(),
            entity_id_gen: Default::default(),
            type_id_index: gen_typeid_map(),
            entity_id_to_table_id: Default::default(),
            table_ids_with_signature: Default::default(),
            tables: Default::default(),
        }
    }

    pub fn add_components(
        &mut self,
        components: Vec<Box<dyn Component>>,
        entity: EntityId,
    ) -> EntityId {
        todo!()
    }

    pub fn remove_components(
        &mut self,
        components: Vec<Box<dyn Component>>,
        entity: EntityId,
    ) -> EntityId {
        todo!()
    }

    pub fn remove(&mut self, entity: EntityId) -> EntityId {
        todo!()
    }


    // todo bitsets
    pub fn spawn(&mut self, entity: Vec<Box<dyn Component>>) -> EntityId {
        // generate bitset
        let table_key: BitSet = {
            // must deref boxed input to get underlying type, otherwise  Box<_> is the Component
            let mut bit_set = BitSet::new();
            entity.iter().map(|c| (**c).type_info().id).for_each(|id| {
                bit_set.insert(self.type_id_index[&id]);
            });
            bit_set
        };

        let table_exists = self.table_ids_with_signature.contains_key(&table_key);
        let new_entity_id = self.entity_id_gen.next();
        if table_exists {
            // insert into existing table
            let table_id = self.table_ids_with_signature[&table_key];
            if let Some(table) = self.tables.get_mut(&table_id) {
                table.add(entity);
                self.entity_id_to_table_id.insert(new_entity_id, table_id);
            }
        } else {
            // create new table and add entities
            let mut table = EntityTable::new(
                entity
                    .iter()
                    .map(|component| (**component).type_info())
                    .collect(),
            );

            table.add(entity);

            let new_table_id = self.table_id_gen.next();
            self.entity_id_to_table_id
                .insert(new_entity_id, new_table_id);
            self.table_ids_with_signature
                .insert(table_key, new_table_id);
            self.tables.insert(new_table_id, table);
        };
        new_entity_id
    }

    /// Main interface for querying
    fn query<'world, Q: TQueryItem + TTableKey + 'world + 'static>(
        &'world mut self,
    ) -> QueryInit<Q> {
        QueryInit::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{entity, storage::component::Component, world::EntityId};

    use super::{EntityIdGen, World};

    #[test]
    fn can_spawn_entities() {
        let mut world = World::new();
        let entities: Vec<EntityId> = (0..1000)
            .map(|_| world.spawn(entity!(1, 2, "hello")))
            .collect();
        assert_eq!(entities.len(), 1000);
    }
}
