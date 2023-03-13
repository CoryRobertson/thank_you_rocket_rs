use crate::state_management::TYRState;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};

#[derive(Default)]
pub struct GetVerifiedGuard(pub bool);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GetVerifiedGuard {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_ip = req.client_ip().unwrap().to_string();
        let outcome: &State<TYRState> = req.guard::<&State<TYRState>>().await.unwrap();
        // if the user is logged in
        if let Some(login_cookie) = req.cookies().get("login") {
            // if there is a verified list
            if let Some(verified_list) = &outcome.admin_state.read().unwrap().verified_list {
                // if the users login is contained within the verified list.
                if verified_list.contains(&login_cookie.to_string()) {
                    return Outcome::Success(Self(true));
                }
            }
        }
        return match &outcome.admin_state.read().unwrap().verified_list {
            None => Outcome::Success(Self(false)),
            Some(list) => Outcome::Success(Self(list.contains(&user_ip))),
        };
    }
}
