//! Just a temporary extension for utoipa, to merge/nest 2 separate apis, until the `nest` macro is published.
//! 
//! Rather inefficient and of subpar quality, but it's ran only once and it's temporary.

use tracing::info;
use utoipa::openapi::{Components, ComponentsBuilder, OpenApi, OpenApiBuilder, Paths, PathsBuilder};
use super::general::{kind_option_lift, merge_btreemaps, merge_hashmaps, merge_vecs};



const UNKNOWN_TAG: &str = "unknown";

/// Extends the original paths by their tag and an additional prefix.
/// 
/// A glimpse into the code:
/// ```rs
/// format!("{}/{}{}", prefix, tag, path) 
/// ```
/// 
/// Example: A `/random` path that has a `cat` tag will be turned into `/cat/random`. With an `/api` prefix it would be turned into `/api/cat/random`. 
fn extend_paths_by_tags(mut paths: Paths, prefix: &str) -> Paths {
    let existing_paths = paths.paths.clone();

    // auughhh
    paths.paths.clear();
    let mut builder = PathsBuilder::from(paths);

    for (name, item) in existing_paths {
        // info!("Prev path: {}", name);

        // should work as long as rule 2 is followed
        let tag = item.operations.first_key_value()
            .and_then(|(_operation_type, operation)| {
                // why tf is it an Option<Vec<_>> ??? it's always a 1-element vec anyway
                operation.tags.as_ref()?.first().map(String::as_str)
            }).unwrap_or(UNKNOWN_TAG);

        // adjustment converts paths like `/dogs/` into `/dogs`
        let adjusted_name = if name == "/" { "" } else { &name };
        let new_name = format!("{}/{}{}", prefix, tag, adjusted_name);
        // info!("New path:  {}", new_name);

        // Update the builder with the new path
        builder = builder.path(new_name, item);
    }

    builder.build()
}

fn merge_paths(paths1: Paths, paths2: Paths) -> Paths {
    let merged_paths = merge_btreemaps(paths1.paths, paths2.paths);
    let merged_extensions = kind_option_lift(merge_hashmaps)(paths1.extensions, paths2.extensions);

    let mut builder = PathsBuilder::new()
        .extensions(merged_extensions);

    for (name, path_item) in merged_paths {
        builder = builder.path(name, path_item);
    }

   builder.build()
}

fn merge_and_extend_paths(paths1: Paths, paths2: Paths, prefix: &str) -> Paths {
    // to merge successfully i need to first extend them, so that theyre unique
    let extended1 = extend_paths_by_tags(paths1, prefix);
    let extended2 = extend_paths_by_tags(paths2, prefix);

    // this is inefficient but once again, it's just a temporary substitute for the upcoming `nest` macro
    merge_paths(extended1, extended2)
}

pub fn show_paths(paths: &Paths) {
    for (name, _item) in &paths.paths {
        info!("Name: {}", name);
    }
}




fn merge_components(comp1: Components, comp2: Components) -> Components {
    let merged_schemas = merge_btreemaps(comp1.schemas, comp2.schemas);
    let merged_responses = merge_btreemaps(comp1.responses, comp2.responses);
    let merged_security_schemes = merge_btreemaps(comp1.security_schemes, comp2.security_schemes);

    let mut builder = ComponentsBuilder::new();

    for (name, schema) in merged_schemas {
        builder = builder.schema(name, schema);
    }

    for (name, response) in merged_responses {
        builder = builder.response(name, response);
    }

    for (name, security_scheme) in merged_security_schemes {
        builder = builder.security_scheme(name, security_scheme);
    }

    builder.build()
}


/// This is not very safe, as there's no traits to ensure that the api's infos are fine, but this is probably the simplest way to achieve this before the nest macro is releaed.
///
/// Rules:
/// 1. The 2 apis need to have tags and each route has to belong to a tag. (might not be necessary, still it's best to do it)
/// 2. The tags have to be the same for the same sub-api. For example any endpoint unter the `/quotes` route has to use the `quotes` tag. (Especially for 2 different methods on the same endpoint)
pub fn nest_openapis_at_prefix(api1: OpenApi, api2: OpenApi, prefix: &str) -> OpenApi {
    let merged_tags = kind_option_lift(merge_vecs)(api1.tags.clone(), api2.tags);
    let merged_components = kind_option_lift(merge_components)(api1.components.clone(), api2.components);
    let merged_and_extended_paths = merge_and_extend_paths(api1.paths.clone(), api2.paths, prefix);
    
    let mut new_api = OpenApiBuilder::from(api1);

    new_api = new_api.paths(Paths::default());

    new_api = new_api
        .tags(merged_tags)
        .components(merged_components)
        .paths(merged_and_extended_paths);
    
    
    new_api.build()
}