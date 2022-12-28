use crate::storage::{component::Component, table::EntityTable};

pub struct ECS {
    entity_tables: Box<[EntityTable]>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            entity_tables: Box::new([]),
        }
    }

    pub fn spawn_entity(&mut self, entities: Vec<Box<dyn Component>>) {
        // check if table exists for entities 
        // if not then create table and add it to tables/
        // return some entity ID that can reference an entity 
    }

    // TODO
    // - entity concept, some identifier for an entity that can be used to find it in a table 
    // - removing/adding components to an entity (creating/updating tables with new entity)
    //      There are some efficiencies that can be made here with graph from one table to another 
    // - entity signature/hash - unique id for table that can quickly get table for a given entity 
    //      based on what types it contains
    // - flesh out column data structure (removing entities, Drop, Deref, DerefMut, ZSTs )



}
