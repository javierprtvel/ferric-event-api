use serde::Serialize;

use crate::model::User;

#[derive(Serialize)]
pub struct SingleUserResponse {
    pub data: User,
}

#[derive(Serialize)]
pub struct ListUsersResponse {
    pub data: Vec<User>,
}
