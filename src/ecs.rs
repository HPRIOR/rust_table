use crate::storage::{component::Component, table::EntityTable};
use crate::storage::component::TypeInfo;
use crate::storage::query::{TInclude, TQueryItem, QueryInit};

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

    pub fn spawn_entity(&mut self, entities: Vec<Box<dyn Component>>) { // should return some
                                                                        // entity ID
        // check if table exists for entities 
        // if not then create table and add it to tables/
        // return some entity ID that can reference an entity 
    }


    fn query<'a, Q: TQueryItem + TInclude + 'a + 'static>(&'a mut self) -> QueryInit<Q> {
        QueryInit::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::World;
    use crate::entity;
    use crate::storage::component::TypeInfo;
    use crate::storage::table::EntityTable;
    use crate::storage::component::Component;
    use crate::storage::query::TQueryItem;

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
        let mut world = World::new_vec(tables);

        let ref_info = TypeInfo::of::<&i32>();
        let literal_info = TypeInfo::of::<i32>();

        println!("{:#?}", ref_info);
        println!("{:#?}", literal_info);

        let mut query = world.query::<(&i32, &f32)>().execute();

        for element in query{
            println!("{:#?}", element);
        }
    }
}

