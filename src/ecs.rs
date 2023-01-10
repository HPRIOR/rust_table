use crate::storage::{component::Component, table::EntityTable};
use crate::storage::component::TypeInfo;
use crate::storage::query::{Filter, Query, QueryExecutor};

pub struct World {
    pub entity_tables: Vec<EntityTable>,
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


    fn query<'a, Q: Query + Filter + 'a + 'static>(&'a self) -> QueryExecutor<Q> {
        let type_info = TypeInfo::of::<Q>();
        // println!("type of query {}", type_info.type_name);
        QueryExecutor::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::World;
    use crate::entity;
    use crate::storage::component::TypeInfo;
    use crate::storage::table::EntityTable;
    use crate::storage::component::Component;
    use crate::storage::query::Query;

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

        let ref_info = TypeInfo::of::<&i32>();
        let literal_info = TypeInfo::of::<i32>();

        println!("{:#?}", ref_info);
        println!("{:#?}", literal_info);

        let mut query = ecs.query::<(&i32, &f32)>();
        let result = query.get().execute().data();

        // for element in result{
        //     println!("{:#?}", element);
        // }
    }
}

