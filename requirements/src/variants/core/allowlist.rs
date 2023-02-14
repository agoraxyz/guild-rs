use crate::Requirement;
use serde::{Deserialize, Serialize};
use std::{
    cmp::PartialEq,
    marker::{Send, Sync},
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AllowList<T> {
    pub verification_data: Vec<T>,
    pub deny_list: bool,
}

impl<T> Requirement for AllowList<T>
where
    T: Sync + Send + PartialEq,
{
    type Error = ();
    type VerificationData = T;

    fn verify(&self, vd: &Self::VerificationData) -> bool {
        self.deny_list != self.verification_data.contains(vd)
    }

    fn verify_batch(&self, vd: &[Self::VerificationData]) -> Vec<bool> {
        vd.iter().map(|v| self.verify(v)).collect()
    }
}

#[cfg(test)]
mod test {
    use super::{AllowList, Requirement};

    #[test]
    fn allowlist_requirement_check() {
        let allowlist = AllowList {
            verification_data: vec![69, 420],
            deny_list: false,
        };

        assert!(allowlist.verify(&69));
        assert!(!allowlist.verify(&13));

        let denylist = AllowList {
            verification_data: vec![69, 420],
            deny_list: true,
        };

        assert!(!denylist.verify(&69));
        assert!(denylist.verify(&13));
    }
}
