use std::marker::PhantomData;
use crate::entity;
use crate::storage::component::TypeInfo;

use super::{component::Component, table::EntityTable};

pub trait Query
{
    // same as self
    type Item<'a>;

    type Fetch: Fetch;

    fn get<'a>(fetch: &Self::Fetch, table: &'a EntityTable) -> Self::Item<'a>;
}

pub trait Fetch {
    type Item<'a>;

    fn execute<'a>(table: &'a EntityTable) -> Self::Item<'a>;
    fn new() -> Self;
}

pub struct FetchRead<T> (PhantomData<T>);

impl<T: Component> Fetch for FetchRead<T> {
    type Item<'a> = &'a [T];

    fn execute<'a>(table: &'a EntityTable) -> Self::Item<'a> {
        if table.has::<T>() {
            println!("table found with type {}", TypeInfo::of::<T>().type_name);
            table.get::<T>()
        } else {
            panic!("table not found with type {}", TypeInfo::of::<T>().type_name);
        }
    }

    fn new() -> Self { Self { 0: Default::default() } }
}

impl<A: Fetch, B: Fetch> Fetch for (A, B) {
    type Item<'a> = (A::Item<'a>, B::Item<'a>);

    fn execute<'a>(table: &'a EntityTable) -> Self::Item<'a> {
        (A::execute(&table), B::execute(&table))
    }

    fn new() -> Self {
        (A::new(), B::new())
    }
}

impl<A: Query, B: Query> Query for (A, B) {
    type Item<'a> = (<<A as Query>::Fetch as Fetch>::Item<'a>, <<B as Query>::Fetch as Fetch>::Item<'a>);
    type Fetch = (A::Fetch, B::Fetch);

    fn get<'a>(fetch: &Self::Fetch, table: &'a EntityTable) -> Self::Item<'a> {
        Self::Fetch::execute(table)
    }
}


impl<'a, T: Component> Query for &'a T {
    type Item<'b> = &'b [T];

    type Fetch = FetchRead<T>;

    fn get<'b>(fetch: &Self::Fetch, table: &'b EntityTable) -> Self::Item<'b> {
        Self::Fetch::execute(table)
    }
}

pub struct Start<Q: Query> {
    tables: Vec<EntityTable>,
    _marker: PhantomData<Q>,
}

impl<Q: Query> Start<Q> {
    fn new(tables: Vec<EntityTable>) -> Self {
        Self {
            tables,
            _marker: PhantomData::default(),
        }
    }

    fn execute(&self) -> <Q as Query>::Item<'_> {
        let fetcher = Q::Fetch::new();
        let result = Q::get(&fetcher, &self.tables[0]);
        result
    }
}

pub fn test() {
    let init_entity = entity![1 + 1 as i32, (1 / 2) as f32];
    let type_infos: Vec<TypeInfo> = init_entity.iter().map(|c| (**c).type_info()).collect();
    let mut table = EntityTable::new(type_infos);
    (0..10000).for_each(|n| {
        let mut entity = entity![n + 1 as i32, (n / 2) as f32];
        table.add(entity);
    });

    let tables = vec![table];

    let start: Start<(&i32, &f32)> = Start::new(tables);
    let data = start.execute();

    let v: Vec<(&i32, &f32)> = data.0.iter().zip(data.1.iter()).collect();

    for (a, b) in v {
        println!("{},{}", a, b)
    }
}
