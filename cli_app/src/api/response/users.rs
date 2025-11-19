use serde::Serialize;

use crate::model::User;

#[derive(Serialize)]
pub struct SingleUserResponse {
    pub data: User,
}
