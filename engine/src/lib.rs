#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::cargo)]
#![deny(unused_crate_dependencies)]

pub use allowlist::AllowList;
use guild_common::User;
use guild_requirement::Requirement;
use thiserror::Error;

mod allowlist;

pub struct Role {
    pub id: String,
    pub filter: Option<AllowList<String>>,
    pub logic: String,
    pub requirements: Option<Vec<Requirement>>,
}

#[derive(Error, Debug)]
pub enum RoleError {
    #[error("Missing requirements")]
    InvalidRole,
    #[error(transparent)]
    Requiem(#[from] requiem::ParseError),
}

impl Role {
    pub async fn check(&self, client: &reqwest::Client, user: &User) -> Result<bool, RoleError> {
        self.check_batch(client, &[user.clone()])
            .await
            .map(|accesses| accesses[0])
    }

    pub async fn check_batch(
        &self,
        client: &reqwest::Client,
        users: &[User],
    ) -> Result<Vec<bool>, RoleError> {
        todo!()
    }
}
