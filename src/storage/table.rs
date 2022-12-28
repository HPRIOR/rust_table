use super::{
    column::Column,
    component::{Component, Type, TypeInfo},
};

#[derive(Debug)]
pub struct EntityTable {
    pub columns: Box<[Column]>,
    column_info: Vec<TypeInfo>,
}

impl EntityTable {
    pub fn new(type_infos: Vec<TypeInfo>) -> Self {
        Self {
            columns: type_infos.iter().map(|ti| Column::new(*ti)).collect(),
            column_info: type_infos,
        }
    }

    pub fn add(&mut self, mut components: Vec<Box<dyn Component>>) {
        unsafe {
            (0..(components.len()))
                .rev()
                .for_each(|i| self.columns[i].push_component(components.remove(i)))
        }
    }
}

#[cfg(test)]
mod tests {}
