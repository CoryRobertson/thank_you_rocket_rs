use crate::state_management::TYRState;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};

#[derive(Default)]
/// Request guard that returns if the user is verified.
pub struct GetVerifiedGuard(pub bool);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GetVerifiedGuard {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_ip = req.client_ip().unwrap().to_string();
        let outcome: &State<TYRState> = req.guard::<&State<TYRState>>().await.unwrap();

        return match &outcome.admin_state.read().unwrap().verified_list {
            None => Outcome::Success(Self(false)), // if no verified list exists, then clearly this user is not verified.

            Some(ver_list) => {
                // if the user is logged in
                if let Some(login_cookie) = req.cookies().get("login") {
                    // if the users login is contained within the verified ver_list.
                    if ver_list.contains(&login_cookie.to_string()) {
                        return Outcome::Success(Self(true));
                    }
                }
                Outcome::Success(Self(ver_list.contains(&user_ip)))
            }
        };
    }
}

#[derive(Default)]
/// Request guard that requires the user to be verified in order for the given route to be valid.
pub struct RequireVerifiedGuard(pub bool);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequireVerifiedGuard {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_ip = req.client_ip().unwrap().to_string();
        let outcome: &State<TYRState> = req.guard::<&State<TYRState>>().await.unwrap();

        return match &outcome.admin_state.read().unwrap().verified_list {
            None => Outcome::Forward(()), // if no verified list exists, then clearly this user is not verified.

            Some(ver_list) => {
                // if the user is logged in
                if let Some(login_cookie) = req.cookies().get("login") {
                    // if the users login is contained within the verified ver_list.
                    if ver_list.contains(&login_cookie.to_string()) {
                        return Outcome::Success(Self(true));
                    }
                }
                if ver_list.contains(&user_ip) {
                    return Outcome::Success(Self(true));
                }
                Outcome::Forward(())
            }
        };
    }
}