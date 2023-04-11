use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AllowList<T> {
    pub deny_list: bool,
    pub list: Vec<T>,
}

impl<T> AllowList<T>
where
    T: PartialEq,
{
    pub fn check(&self, entry: &T) -> bool {
        self.deny_list != self.list.contains(entry)
    }

    pub fn check_many(&self, entries: &[T]) -> Vec<bool> {
        entries.iter().map(|entry| self.check(entry)).collect()
    }
}

#[cfg(test)]
mod test {
    use super::AllowList;

    #[test]
    fn allowlist_check() {
        let allowlist = AllowList {
            deny_list: false,
            list: vec![69, 420],
        };

        assert!(allowlist.check(&69));
        assert!(!allowlist.check(&13));

        let denylist = AllowList {
            deny_list: true,
            list: vec![69, 420],
        };

        assert!(!denylist.check(&69));
        assert!(denylist.check(&13));
    }
}
