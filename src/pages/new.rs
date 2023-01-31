use crate::message::Messages;
use rocket::response::content::RawHtml;
use rocket::State;
use std::net::SocketAddr;

#[get("/new")]
/// Page for creating a new message
pub fn new(_req: SocketAddr, _messages: &State<Messages>) -> RawHtml<String> {
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
            <form action="/submit_message" method="post">
                <label for="msg">Enter message</label>
                <br>
                <input type="text" name="msg" id="msg">
                <input type="submit" value="Submit Message">
            </form>
        </body>
    </html>
    "#
        .parse()
        .unwrap(),
    )
}
