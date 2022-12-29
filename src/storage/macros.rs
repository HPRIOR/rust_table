#[macro_export]
macro_rules! entity {
    ($($x:expr),*) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.as_component());
            )*

            temp_vec.sort_by(|a, b| a.type_info().partial_cmp(&b.type_info()).unwrap());
            temp_vec
        }
    };
}

#[macro_export]
macro_rules! query {
    ($tables: expr => with ($($type:ty),*)) => {
        let mut included_types: Vec<TypeInfo> = vec![];
        $(
            let typeinfo = TypeInfo::of::<$type>();
            included_types.push(typeinfo);
        )*
        // let columns = 

    };
}
