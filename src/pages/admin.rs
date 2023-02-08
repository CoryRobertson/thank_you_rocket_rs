

// TODO: implement a random uuid as a password, generated at runtime, navigating to this page displays all messages sent and stored in the state.
//  password would use the uuid crate, and have a page that is of low route priority, and takes in any string, validates the priority, then displays the content, if not, displays the 404 error
//  this will use a request guard!
//  this will also use a specific page that stores the key as a cookie?

// TODO: on first run of program (potentially use a file existing or not as a marker? maybe with password hash stored inside?) ask the first person to connect to "/admin" to type in a password, since
//  they are the first person there, take what ever they type, store its hash, and use that for login.

// TODO: store the fact that a user is an admin by using a cookie? if this is the correct way to do this.

// TODO: create a request guard that required admin rights to go to any page with this request guard.