// module for all of the pages that get used for this project
pub mod admin_view;
pub mod error_catch_pages; // pages relating to error catching
pub mod index; // the base page of the project
pub mod new; // the page for creating new messages through a form
pub mod outcome_pages; // module of pages for getting specific info to the user e.g. "message was too long" or "message contained an error" or otherwise
pub mod submit_message; // the post request handler for handling a new message from the new message page
pub mod view; // a page for viewing all the messages sent by a user
