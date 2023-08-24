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
*
*
* todo 
* - find a better way to do table hashing, most games will have more components than 64, so bitset
* comparisons will quickly become inefficient. A hashset of table components will probably be
* suitable
* - add concurrency where possible. Easy win - parallelise archetype access in queries.
* Concurrency within an archetype will be more tricky 
* - Generational entity ids. Instead of incrementing every new entity (more allocations and a
* finite set), entity ids should be recycled. Generation concept could also be useful for lazy
* removal/adding of components/entities. Entity id can be used to store whether or not its 'alive'
* - Add more features to queries. E.g. exclusive, inclusive, etc. Greater control over what is
* retrieved
* - Add 'systems', that executes over queries at regular intervals. This will be further along when there
* is some kind game loop. Ordering will be necessary
* - Table graph with most visited nodes for each archetype. Speed up transitions between
* archetypes. Could also try to reduce allocations with direct unsafe copies of components without
* the intermediate Vec<Box<dyn Component>.
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
    pub type_id_index: Vec<TypeId>,
    entity_id_to_table_id: HashMap<EntityId, TableId>,
    pub table_ids_with_signature: HashMap<BitSet, TableId>,
    pub tables: HashMap<TableId, EntityTable>,
}

// todo, support adding arbitrary types
fn gen_typeid_map() -> Vec<TypeId> {
    vec![
        TypeId::of::<i8>(),
        TypeId::of::<i16>(),
        TypeId::of::<i32>(),
        TypeId::of::<i64>(),
        TypeId::of::<i128>(),
        TypeId::of::<isize>(),
        TypeId::of::<u8>(),
        TypeId::of::<u16>(),
        TypeId::of::<u32>(),
        TypeId::of::<u64>(),
        TypeId::of::<u128>(),
        TypeId::of::<usize>(),
        TypeId::of::<f32>(),
        TypeId::of::<f64>(),
        TypeId::of::<bool>(),
        TypeId::of::<char>(),
        TypeId::of::<String>(),
        TypeId::of::<&str>(),
        TypeId::of::<str>(),
    ]
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

    pub fn register_component<T: Component>(&mut self) {
        self.type_id_index.push(TypeId::of::<T>());
    }

    // for now assumes no overlapping components with those added and previously in entity
    // will need to check this efficiently somehow
    pub fn add_components(
        &mut self,
        mut comp_to_add: Vec<Box<dyn Component>>,
        entity: EntityId,
    ) -> Option<EntityId> {
        let table_id = self.entity_id_to_table_id.get(&entity)?;
        let entity_table = self.tables.get_mut(table_id)?;
        let mut new_components = entity_table.remove_entity(entity);
        new_components.append(&mut comp_to_add);

        Some(self.spawn(new_components, Some(entity)))
    }

    pub fn remove_components(
        &mut self,
        comp_to_remove: Vec<TypeInfo>,
        entity: EntityId,
    ) -> Option<EntityId> {
        let table_id = self.entity_id_to_table_id.get(&entity)?;
        let entity_table = self.tables.get_mut(table_id)?;

        let new_components: Vec<Box<dyn Component>> = entity_table
            .remove_entity(entity)
            .into_iter()
            .filter(|component| {
                let t_info = (**component).type_info().id;
                let ids_to_remove: Vec<TypeId> =
                    comp_to_remove.iter().map(move |info| info.id).collect();
                !ids_to_remove.contains(&t_info)
            })
            .collect();

        Some(self.spawn(new_components, Some(entity)))
    }

    pub fn remove(&mut self, entity: EntityId) -> EntityId {
        todo!()
    }

    // New id will be generated only no entity id is passed in
    pub fn spawn(
        &mut self,
        entity: Vec<Box<dyn Component>>,
        entity_id: Option<EntityId>,
    ) -> EntityId {
        let table_key: BitSet = {
            let mut bit_set = BitSet::new();

            // must deref boxed input to get underlying type, otherwise  Box<_> is the Component
            entity.iter().map(|c| (**c).type_info().id).for_each(|id| {
                bit_set.insert(
                    self.type_id_index
                        .iter()
                        .position(|type_id| *type_id == id)
                        .unwrap(),
                );
            });
            bit_set
        };

        let table_exists = self.table_ids_with_signature.contains_key(&table_key);
        let new_entity_id = entity_id.unwrap_or_else(|| self.entity_id_gen.next());
        if table_exists {
            // insert into existing table
            let table_id = self.table_ids_with_signature[&table_key];
            if let Some(table) = self.tables.get_mut(&table_id) {
                table.add_entity(entity, new_entity_id);
                self.entity_id_to_table_id.insert(new_entity_id, table_id);
            }
        } else {
            // create new table and add entities
            let mut table = EntityTable::new(
                entity
                    .iter()
                    .map(|component| (**component).type_info())
                    .collect(),
                table_key.clone(),
            );

            table.add_entity(entity, new_entity_id);
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
    use crate::{
        entity,
        storage::component::{Component, TypeInfo},
        world::EntityId,
    };

    use super::{EntityIdGen, World};

    #[test]
    fn can_spawn_entities() {
        let mut world = World::new();
        let entities: Vec<EntityId> = (0..1000)
            .map(|_| world.spawn(entity!(1, 2, "hello"), None))
            .collect();

        let query = world.query::<&u8>().execute();
        assert_eq!(entities.len(), 1000);
    }

    #[test]
    fn can_remove_components_from_entities() {
        let mut world = World::new();
        let entity = world.spawn(entity!(1_u32, 2_u8, "hello"), None);

        let query = world.query::<&u32>().execute();
        assert!(query.count() == 1);

        let query = world.query::<(&u32, &u8)>().execute();
        assert!(query.count() == 1);

        world.remove_components(vec![TypeInfo::of::<u32>()], entity);
        let query = world.query::<&u32>().execute();
        assert!(query.count() == 0);

        let query = world.query::<(&u32, &u8)>().execute();
        assert!(query.count() == 0);

        let query = world.query::<&u8>().execute();
        assert!(query.count() == 1);
    }

    #[test]
    fn can_add_componentts_to_entities() {
        let mut world = World::new();
        let entity = world.spawn(entity!(1_u32, 2_u8), None);

        let query = world.query::<(&u32, &u8)>().execute();
        assert!(query.count() == 1);

        let query = world.query::<(&u32, &u64)>().execute();
        assert!(query.count() == 0);

        world.add_components(entity!(200_u64), entity);


        let query = world.query::<(&u32, &u64)>().execute();
        assert!(query.count() == 1);
    }
}
