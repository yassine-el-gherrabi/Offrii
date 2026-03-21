use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::friends::{
    FriendRequestResponse, FriendResponse, SentFriendRequestResponse, UserSearchResult,
};
use crate::errors::AppError;
use crate::models::{FriendRequest, FriendRequestStatus, FriendWithSince, Friendship};

#[async_trait]
pub trait FriendRepo: Send + Sync {
    async fn create_friend_request(
        &self,
        from_user_id: Uuid,
        to_user_id: Uuid,
    ) -> Result<FriendRequest>;

    async fn find_pending_requests(&self, to_user_id: Uuid) -> Result<Vec<FriendRequest>>;

    async fn find_pending_requests_paginated(
        &self,
        to_user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<FriendRequest>>;

    async fn count_pending_requests(&self, to_user_id: Uuid) -> Result<i64>;

    async fn find_request_by_id(&self, id: Uuid) -> Result<Option<FriendRequest>>;

    async fn update_request_status(&self, id: Uuid, status: FriendRequestStatus) -> Result<bool>;

    async fn create_friendship(&self, user_a_id: Uuid, user_b_id: Uuid) -> Result<Friendship>;

    async fn delete_friendship(&self, user_a_id: Uuid, user_b_id: Uuid) -> Result<bool>;

    async fn list_friends(&self, user_id: Uuid) -> Result<Vec<Uuid>>;

    async fn list_friends_with_since(&self, user_id: Uuid) -> Result<Vec<FriendWithSince>>;

    async fn list_friends_with_since_paginated(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<FriendWithSince>>;

    async fn count_friends(&self, user_id: Uuid) -> Result<i64>;

    async fn are_friends(&self, user_a_id: Uuid, user_b_id: Uuid) -> Result<bool>;

    async fn find_sent_requests(&self, from_user_id: Uuid) -> Result<Vec<FriendRequest>>;

    async fn find_sent_requests_paginated(
        &self,
        from_user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<FriendRequest>>;

    async fn count_sent_requests(&self, from_user_id: Uuid) -> Result<i64>;

    async fn find_pending_between(
        &self,
        from_user_id: Uuid,
        to_user_id: Uuid,
    ) -> Result<Option<FriendRequest>>;

    async fn count_shared_items_per_user(
        &self,
        friend_ids: &[Uuid],
        viewer_id: Uuid,
    ) -> Result<HashMap<Uuid, i64>>;
}

#[async_trait]
pub trait FriendService: Send + Sync {
    async fn search_users(
        &self,
        query: &str,
        requester_id: Uuid,
    ) -> Result<Vec<UserSearchResult>, AppError>;

    async fn send_request(
        &self,
        from_user_id: Uuid,
        to_username: &str,
    ) -> Result<FriendRequestResponse, AppError>;

    async fn list_pending_requests(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FriendRequestResponse>, i64), AppError>;

    async fn accept_request(
        &self,
        request_id: Uuid,
        user_id: Uuid,
    ) -> Result<FriendResponse, AppError>;

    async fn decline_request(&self, request_id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn list_friends(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FriendResponse>, i64), AppError>;

    async fn list_sent_requests(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<SentFriendRequestResponse>, i64), AppError>;

    async fn cancel_request(&self, request_id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn remove_friend(&self, user_id: Uuid, friend_id: Uuid) -> Result<(), AppError>;
}
