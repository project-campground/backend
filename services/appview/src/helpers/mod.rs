pub fn deduplicate_list<T>(list: Vec<T>) -> Vec<T>
    where T: PartialEq + Clone
{
    let mut deduplicated = Vec::new();
    for item in list {
        if !deduplicated.contains(&item) {
            deduplicated.push(item.clone());
        }
    }
    deduplicated
}

pub fn lower_list<T>(list: Vec<T>) -> Vec<String>
    where T: ToString
{
    let mut lowered = Vec::new();
    for item in list {
        lowered.push(item.to_string().to_lowercase());
    }
    lowered
}

pub mod util;
pub mod views;
pub mod campsites;
pub mod tents;
pub mod api;
pub mod posts;