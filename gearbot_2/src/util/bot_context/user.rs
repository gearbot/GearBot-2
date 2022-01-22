use crate::cache::User;
use crate::BotContext;
use gearbot_2_lib::util::GearResult;
use std::sync::Arc;
use twilight_http::error::ErrorType;
use twilight_model::id::UserId;

impl BotContext {
    pub async fn get_user_info(&self, user_id: &UserId) -> GearResult<Option<Arc<User>>> {
        // check cache
        if let Some(user) = self.cache.get_user(user_id) {
            Ok(Some(user))
        } else {
            // try the api
            match self.client.user(*user_id).exec().await {
                Ok(response) => {
                    let user = response.model().await?;
                    Ok(Some(Arc::new(User::assemble(user, None))))
                }
                Err(e) => {
                    // was this an 404 not found cause the userid is invalid?
                    if matches!(e.kind(), ErrorType::Response { status, .. } if *status == 404) {
                        return Ok(None);
                    }
                    //something else went wrong, raise it
                    Err(e.into())
                }
            }
        }
    }
}
