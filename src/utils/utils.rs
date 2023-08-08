use std::{collections::HashSet, hash::Hash};

pub trait IntersectAll<A> {
    fn intersect_all(&self) -> HashSet<A>;
}

impl<A: Eq + Hash + Copy + Clone> IntersectAll<A> for Vec<&HashSet<A>> {
    fn intersect_all(&self) -> HashSet<A> {
        self.get(0)
            .map(|set| {
                set.iter()
                    .copied()
                    .filter(|item| self.iter().all(|set_other| set_other.contains(item)))
                    .collect()
            })
            .unwrap_or(HashSet::new())
    }
}
