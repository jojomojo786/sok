use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

#[derive(Template)]
#[template(path = "categories.html")]
pub struct CategoriesTemplate;

#[derive(Template)]
#[template(path = "channels.html")]
pub struct ChannelsTemplate;
