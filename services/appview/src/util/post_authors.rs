use std::collections::HashSet;

use crate::database::{
    models::appview::Actor,
    models::appview::Profile,
    models::appview::ProfilePost,
};

pub fn get_authors_from_posts(posts: &Vec<ProfilePost>, collect_parent_authors: bool) -> HashSet<String> {
    posts
        .iter()
        // at://did:.../
        .flat_map(|x| {
            // Gets authors from parent URIs as well
            (vec![
                Some(x.uri.split('/').collect::<Vec<&str>>()[2].to_string()),

                if collect_parent_authors
                    { x.parent_uri.as_ref().map(|y| y.split('/').collect::<Vec<&str>>()[2].to_string()) }
                else
                    { None }
            ])
                .iter()
                .filter_map(|y| y.clone())
                .collect::<Vec<String>>()
        })
        .collect::<HashSet<String>>()
}

pub fn populate_profile_posts_with_authors(posts: Vec<ProfilePost>, author_profiles: &Vec<(Actor, Profile)>) -> Vec<(Actor, Profile, ProfilePost)> {
    populate_profile_posts_with_authors_from_iter(posts.into_iter().clone(), author_profiles)
}

pub fn populate_profile_posts_with_authors_from_iter<I>(posts: I, author_profiles: &Vec<(Actor, Profile)>) -> Vec<(Actor, Profile, ProfilePost)>
    where I: Iterator<Item = ProfilePost>
{
    return posts
        .map(|x| {
            let actor_tuple = author_profiles.iter().find(|y| y.0.did == x.author);
            (actor_tuple, x)
        })
        .filter(|x| x.0.is_some())
        .map(|x| {
            let a: &(Actor, Profile) = x.0.unwrap();
            (a.0.clone(), a.1.clone(), x.1.clone())
        })
        .collect::<Vec<(Actor, Profile, ProfilePost)>>();
}
