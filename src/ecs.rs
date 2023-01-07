use crate::storage::{component::Component, table::EntityTable};
use crate::storage::query::{Query, QueryExecutor};

pub struct World {
    entity_tables: Vec<EntityTable>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_tables: vec![]
        }
    }
    pub fn new_vec(tables: Vec<EntityTable>) -> Self {
        Self { entity_tables: tables }
    }

    pub fn spawn_entity(&mut self, entities: Vec<Box<dyn Component>>) {
        // check if table exists for entities 
        // if not then create table and add it to tables/
        // return some entity ID that can reference an entity 
    }

    fn query<'a, Q: Query + 'a>(&'a self) -> QueryExecutor<Q> {
        QueryExecutor::new(&self.entity_tables)
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::World;
    use crate::entity;
    use crate::storage::component::TypeInfo;
    use crate::storage::table::EntityTable;
    use crate::storage::component::Component;

    #[test]
    fn test() {
        let init_entity = entity![1 + 1 as i32, (1 / 2) as f32];
        let type_infos: Vec<TypeInfo> = init_entity.iter().map(|c| (**c).type_info()).collect();
        let mut table = EntityTable::new(type_infos);
        (0..10).for_each(|n| {
            let mut entity = entity![n + 1 as i32, (n / 2) as f32];
            table.add(entity);
        });

        let tables = vec![table];
        let ecs = World::new_vec(tables);

        let query  = ecs.query::<&i32>();
        let result = query.execute();

        for element in result{
            println!("{:#?}", element);
        }
    }
}

