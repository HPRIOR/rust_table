use std::{collections::HashSet, hash::Hash};

pub trait IntersectAll<A> {
    fn intersect_all(&self) -> HashSet<A>;
}

impl<A: Eq + Hash + Copy + Clone> IntersectAll<A> for Vec<&HashSet<A>> {
    // see https://www.reddit.com/r/rust/comments/5v35l6/intersection_of_more_than_two_sets/ for a
    // more efficient implementation. If they're in sorted order better things can be done
    // also no need to compare with self (??)
    fn intersect_all(&self) -> HashSet<A> {
        self.get(0)
            .map(|set| {
                set.iter()
                    .filter(|item| self.iter().all(|set_other| set_other.contains(item)))
                    .map(|i| i.clone())
                    .collect()
            })
            .unwrap_or(HashSet::new())
    }
}
