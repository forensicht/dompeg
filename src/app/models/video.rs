use core_dompeg as core;

pub const ZOOM_SIZE: i32 = 32;
pub const THUMBNAIL_SIZE: i32 = 160;

#[derive(Debug, Default, Clone)]
pub struct Video {
    pub name: String,
    pub path: String,
    pub size: usize,
    pub is_selected: bool,
    pub thumbnail_size: i32,
}

impl From<&core::video::Video> for Video {
    fn from(value: &core::video::Video) -> Self {
        Self { 
            name: value.name.to_owned(), 
            path: value.path.to_owned(), 
            size: value.size, 
            is_selected: false,
            thumbnail_size: THUMBNAIL_SIZE,
        }
    }
}
