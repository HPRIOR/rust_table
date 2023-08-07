use crate::storage::component::TypeInfo;
use crate::storage::query::{QueryInit, TQueryItem, TTableKeySingle, TTableKey};
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

// TableKey should be bitset
type TableKey = Vec<TypeId>;

pub struct World {
    table_id_gen: TableIdGen,
    entity_id_gen: EntityIdGen,

    entity_id_to_table_id: HashMap<EntityId, TableId>,
    
    table_ids_with_signature: HashMap<TableKey, TableId>,
    pub tables: HashMap<TableId, EntityTable>,
    pub tables_with_component_id: HashMap<TypeId, HashSet<TableId>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            table_id_gen: Default::default(),
            entity_id_gen: Default::default(),

            entity_id_to_table_id: Default::default(),
            table_ids_with_signature: Default::default(),
            tables: Default::default(),
            tables_with_component_id: Default::default(),
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

    fn update_table_component_map(&mut self, table_key: &TableKey, table_id: TableId) {
        for type_id in table_key.iter() {
            if !self.tables_with_component_id.contains_key(type_id) {
                self.tables_with_component_id
                    .insert(*type_id, HashSet::new());
            }
            if let Some(table_id_set) = self.tables_with_component_id.get_mut(type_id) {
                table_id_set.insert(table_id);
            }
        }
    }

    // todo bitsets
    pub fn spawn(&mut self, entity: Vec<Box<dyn Component>>) -> EntityId {
        // generate bitset 
        let table_key: TableKey = {
            // must deref boxed input to get underlying type, otherwise  Box<_> is the Component
            let mut keys: Vec<TypeId> = entity.iter().map(|c| (**c).type_info().id).collect();
            keys.sort();
            keys
        };

        let table_exists = self.table_ids_with_signature.contains_key(&table_key);
        let new_entity_id = self.entity_id_gen.next();
        if table_exists {
            // insert into existing table
            let table_id = self.table_ids_with_signature[&table_key];
            if let Some(table) = self.tables.get_mut(&table_id) {
                table.add(entity);
                self.update_table_component_map(&table_key, table_id);
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
            self.update_table_component_map(&table_key, new_table_id);
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
