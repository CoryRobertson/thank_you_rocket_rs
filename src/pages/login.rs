use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use lazy_static::lazy_static;
use rocket::form::Form;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;


lazy_static! {
    // probably not best practice, but a lazy static salt at run time seems like an ok idea for now.
    // later down the road, this will invalidate all passwords every time the website resets, which will probably not be good.
    // maybe use a const rng system to generate some random strings as a salt?
    static ref SALT: SaltString = SaltString::generate(&mut OsRng);
}

#[get("/login")]
pub fn login() -> RawHtml<String> {
    // TODO: hide password when typed
    RawHtml(
        r#"
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Submit</title>
    </head>
        <body>
            <form action="/login" method="post">
                <label for="password">Enter password</label>
                <br>
                <input type="text" name="password" id="password">
                <input type="submit" value="Submit password">
            </form>
        </body>
    </html>
    "#
            .parse()
            .unwrap(),
    )
}

#[derive(FromForm, Debug, Clone)]
pub struct Login {
    pub password: String
}

#[post("/login", data = "<password>")]
pub fn login_post(password: Form<Login>) -> Redirect {
    let a2 = Argon2::default();
    // at the moment, salt is insecure, fix later FIXME
    let hash_password = a2.hash_password(password.password.as_bytes(),"ABFDABFDABFDABFD").unwrap();

    println!("Password: {}", password.password);
    println!("Hash: {}", hash_password.hash.unwrap());
    Redirect::to(uri!("/"))
}
