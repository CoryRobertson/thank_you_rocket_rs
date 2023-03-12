use crate::state_management::TYRState;
use rocket::request::{FromRequest, Outcome};
use rocket::{request, Request, State};

#[derive(Default)]
pub struct GetVerifiedGuard(pub bool);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GetVerifiedGuard {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_ip = req.client_ip().unwrap().to_string();
        let outcome: &State<TYRState> = req.guard::<&State<TYRState>>().await.unwrap();
        return match &outcome.admin_state.read().unwrap().verified_ip_addressed {
            None => Outcome::Success(Self(false)),
            Some(list) => Outcome::Success(Self(list.contains(&user_ip))),
        };
    }
}
