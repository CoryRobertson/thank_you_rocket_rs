use crate::pages::outcome_pages::paste_404;
use crate::paste::{Paste, PasteContents};
use crate::verified_guard::{GetVerifiedGuard, RequireVerifiedGuard};
use crate::TYRState;
use maud::{html, PreEscaped};
use rocket::data::ToByteUnit;
use rocket::form::Form;
use rocket::http::{ContentType, CookieJar, Status};
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::tokio::io::AsyncReadExt;
use rocket::{Data, State};
use rocket_download_response::DownloadResponse;
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Read, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(FromForm, Debug, Clone)]
/// Form struct for a message
pub struct NewPaste {
    pub text: String,
    pub custom_url: Option<String>,
}

#[post("/paste/upload/<filename>", data = "<paste>")]
/// Route for uploading a file to the paste section
/// echo "this is a test" | curl --data-binary @- http://localhost:8080/paste/upload/<filename>
pub async fn upload(
    paste: Data<'_>,
    state: &State<TYRState>,
    filename: String,
    req: SocketAddr,
    jar: &CookieJar<'_>,
    _require_verified: RequireVerifiedGuard
) -> Redirect {
    let mut file_content = String::new();
    let _file_size = paste
        .open(128.kibibytes())
        .read_to_string(&mut file_content)
        .await
        .unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    file_content.hash(&mut hasher);

    let path = PathBuf::from(format!("./output/file_uploads/{}", filename));
    // println!("path: {:?}",path);
    if !Path::new(&path).exists() {
        let mut file = match File::create(&path) {
            Ok(f) => {
                // println!("ok file create");
                f
            }
            Err(_) => return Redirect::to(uri!("/error_message")),
        };

        match file.write_all(file_content.as_bytes()) {
            Ok(_) => {
                // println!("ok file write all");
            }
            Err(_) => return Redirect::to(uri!("/error_message")),
        }

        let _ = file.sync_all();

        let mut lock = state.pastes.write().unwrap();

        lock.insert(
            hasher.finish().to_string(),
            Paste::new_file_paste(path, &req, jar),
        );

        Redirect::to(uri!("/"))
    } else {
        Redirect::to(uri!("/error_message"))
    }
}

#[post("/paste/upload", data = "<data>")]
/// Route for uploading a file to the paste section
/// echo "this is a test" | curl --data-binary @- http://localhost:8080/paste/upload/<filename>
pub async fn upload_multipart(
    content_type: &ContentType,
    data: Data<'_>,
    state: &State<TYRState>,
    req: SocketAddr,
    jar: &CookieJar<'_>,
    _require_verified: RequireVerifiedGuard
) -> Redirect {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("data"), // this one allows for random txt files
        MultipartFormDataField::bytes("data"),
    ]);

    match MultipartFormData::parse(content_type, data, options).await {
        Ok(multipart_form_data) => {
            if let Some(file) = multipart_form_data.texts.get("data") {
                // println!("{:?}", file);
                if let Some(text_field) = file.get(0) {
                    let path = PathBuf::from(format!(
                        "./output/file_uploads/{}",
                        text_field.file_name.clone().unwrap_or_default()
                    ));
                    if !path.exists() {
                        let mut file = match File::create(path.clone()) {
                            Ok(f) => f,
                            Err(_) => {
                                return Redirect::to(uri!("/error_message"));
                            }
                        };

                        match file.write_all(text_field.text.as_bytes()) {
                            Ok(_) => {
                                // println!("ok file write all");
                            }
                            Err(_) => return Redirect::to(uri!("/error_message")),
                        }

                        let _ = file.sync_all();

                        let mut hasher = DefaultHasher::new();
                        text_field.text.hash(&mut hasher);

                        let mut lock = state.pastes.write().unwrap();

                        lock.insert(
                            hasher.finish().to_string(),
                            Paste::new_file_paste(path, &req, jar),
                        );

                        return Redirect::to(uri!("/"));
                    }
                }
            } else if let Some(raw_bytes_vec) = multipart_form_data.raw.get("data") {
                if let Some(raw_bytes_data) = raw_bytes_vec.get(0) {
                    let vec_bytes = &raw_bytes_data.raw;

                    let path = PathBuf::from(format!(
                        "./output/file_uploads/{}",
                        raw_bytes_data.file_name.clone().unwrap_or_default()
                    ));
                    if !path.exists() {
                        let mut file = match File::create(path.clone()) {
                            Ok(f) => f,
                            Err(_) => {
                                return Redirect::to(uri!("/error_message"));
                            }
                        };

                        match file.write_all(vec_bytes) {
                            Ok(_) => {
                                // println!("ok file write all");
                            }
                            Err(_) => return Redirect::to(uri!("/error_message")),
                        }

                        let _ = file.sync_all();

                        let mut hasher = DefaultHasher::new();
                        vec_bytes.hash(&mut hasher);

                        let mut lock = state.pastes.write().unwrap();
                        let file_hash = hasher.finish().to_string();

                        lock.insert(
                            file_hash.clone(),
                            Paste::new_file_paste(path, &req, jar),
                        );

                        return Redirect::to(uri!(view_paste(file_hash)));
                    }
                }
            }

            // println!("{:?}", multipart_form_data);
        }
        Err(_err) => {
            // println!("{}", err);
            // println!("{}", content_type);
        }
    }

    return Redirect::to(uri!("/error_message"));
}

#[post("/paste/new", data = "<paste>")]
/// Post request handler for creating new pastes.
pub fn new_paste_post(
    paste: Form<NewPaste>,
    req: SocketAddr,
    state: &State<TYRState>,
    jar: &CookieJar,
    is_verified: GetVerifiedGuard,
) -> Redirect {
    let mut hasher = DefaultHasher::new();
    paste.text.hash(&mut hasher);
    let text_hash = hasher.finish();
    let mut lock = state.pastes.write().unwrap();
    let paste_struct = Paste::new(paste.text.clone(), &req, jar);
    // custom url is either the forms given custom url, or the text hash if no custom url is given.
    let custom_url = paste.custom_url.clone().unwrap_or(text_hash.to_string());
    let url_already_exists = { lock.iter().map(|(id, _)| id).any(|id| id == &custom_url) }; // variable for if the given custom url already exists

    if is_verified.0 && !url_already_exists {
        // if the user is both verified, and this given custom url does not exist.
        lock.insert(custom_url.clone(), paste_struct);
        let uri = uri!(view_paste(custom_url));
        Redirect::to(uri)
    } else {
        lock.insert(text_hash.to_string(), paste_struct);
        let uri = uri!(view_paste(text_hash.to_string()));
        Redirect::to(uri)
    }
}

#[get("/paste/view/<paste_id>/file")]
/// Page for viewing created pastes that are files, attempts to have the user download the paste.
pub async fn download_file_paste(
    paste_id: String,
    _req: SocketAddr,
    state: &State<TYRState>,
) -> Result<DownloadResponse, Status> {
    let binding = state.pastes.read().unwrap().clone();
    let paste_opt = binding.get(&paste_id.clone());
    match paste_opt {
        None => Err(Default::default()),
        Some(paste) => match &paste.content {
            PasteContents::File(path) => {
                let file_name = path.file_name().unwrap().to_str();
                DownloadResponse::from_file(path.clone().into_boxed_path(), file_name, None)
                    .await
                    .map_err(|err| {
                        if err.kind() == ErrorKind::NotFound {
                            Status::NotFound
                        } else {
                            Status::InternalServerError
                        }
                    })
            }
            PasteContents::PlainText(text) => {
                Ok(DownloadResponse::from_vec(text.clone().into_bytes(),Some(paste_id),None))
            },
        },
    }
}

#[get("/paste/view/<paste_id>")]
/// Page for viewing created pastes, viewing only, download optional.
pub fn view_paste(paste_id: String, _req: SocketAddr, state: &State<TYRState>) -> RawHtml<String> {
    let binding = state.pastes.read().unwrap();
    let paste_opt = binding.get(&paste_id);
    let back_button = "<button onclick=\"window.location.href=\'/\';\">Go back</button>";
    let file_button = format!("<button onclick=\"window.location.href=\'/paste/view/{}/file\';\">Download file</button>", paste_id);

    let escaped = match paste_opt {
        None => paste_404(),
        Some(text_paste) => {
            match &text_paste.content {
                PasteContents::File(path) => {
                    match File::open(path).ok() {
                        None => "File un-readable. Error occurred.".to_string(),
                        Some(mut file) => {
                            let mut file_contents = String::new();
                            file.read_to_string(&mut file_contents).unwrap_or_default();
                            let escaped = html_escape::encode_safe(&file_contents); // escape so no xss can happen!
                            escaped.to_string()
                        }
                    }
                }
                PasteContents::PlainText(text) => {
                    let escaped = html_escape::encode_safe(&text); // escape so no xss can happen!
                    escaped.replace("\r\n", "<br>").replace('\n', "<br>")
                }
            }
        }
    };

    RawHtml(
        html! {
            (PreEscaped(back_button))
            (PreEscaped(file_button))
            p {(PreEscaped(escaped))}
        }
        .into_string(),
    )
}

#[get("/paste/new")]
/// Page for creating a new paste
pub fn new_paste(
    _req: SocketAddr,
    _state: &State<TYRState>,
    is_verified: GetVerifiedGuard,
) -> RawHtml<String> {
    let back_button = "<button onclick=\"window.location.href=\'/\';\">Go back</button>";
    if is_verified.0 {
        RawHtml(
            html! {
            (PreEscaped(r#"
            <form action="/paste/new" method="post">
                <label for="ip">Enter paste</label>
                <br>
                    <textarea rows = "5" cols = "60" name = "text"></textarea>
                    <br>
                    <p>Custom url: </p>
                    <input type="text" name="custom_url" id="custom_url">
                <br>
                <br>
                <input type="submit" value="Submit paste">
            </form>
            <form id="form" enctype="multipart/form-data" method="post" action="/paste/upload">
                <div class="input-group">
                    <label for="files">Select files</label>
                    <input id="file" name="data" type="file" multiple />
                </div>
            <button class="submit-btn" type="submit">Upload</button>
        </form>
    "#))
                br;
                (PreEscaped(back_button))
            }
            .into_string(),
        )
    } else {
        // user is not verified
        RawHtml(
            html! {
            (PreEscaped(r#"
            <form action="/paste/new" method="post">
                <label for="ip">Enter paste</label>
                <br>
                    <textarea rows = "5" cols = "60" name = "text"></textarea>
                <br>
                <input type="submit" value="Submit paste">
            </form>
    "#))
                br;
                (PreEscaped(back_button))
            }
            .into_string(),
        )
    }
}
