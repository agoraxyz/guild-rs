use crate::Requirement;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Free;

impl Requirement for Free {
    type Error = ();
    type VerificationData = ();

    fn verify(&self, _v: &Self::VerificationData) -> bool {
        true
    }

    fn verify_batch(&self, v: &[Self::VerificationData]) -> Vec<bool> {
        vec![true; v.len()]
    }
}

#[cfg(test)]
mod test {
    use super::{Free, Requirement};

    #[test]
    fn free_requirement_check() {
        let req = Free;

        assert!(req.verify(&()));
    }
}
