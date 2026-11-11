pub fn routes() -> Vec<rocket::Route> {
    routes![
        v1::subscribe
    ]
}

mod v1;
mod permissions;
mod reactive;