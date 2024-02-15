pub mod services;

pub mod service {
    use crate::services::video;

    #[derive(Debug, Clone, Copy)]
    pub enum Service {
        Video(video::VideoService),
    }
}