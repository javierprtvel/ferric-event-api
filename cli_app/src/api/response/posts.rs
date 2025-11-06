use serde::Serialize;

use crate::model::Post;

#[derive(Serialize)]
pub struct SinglePostResponse {
    pub data: Post,
}

#[derive(Serialize)]
pub struct ListPostsResponse {
    pub data: Vec<Post>,
}
