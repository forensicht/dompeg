use std::path::Path;
use anyhow::Result;

pub mod video {
    use super::*;
    use crate::core::search;
    use crate::core::decoder::video;

    pub async fn search_videos<P>(dir: P) -> Result<Vec<search::Video>>
    where P: AsRef<Path>, 
    {
        search::search_videos(dir).await
    }

    pub async fn thumbnail<P>(video_path: P) -> Result<video::VideoThumb>
    where P: AsRef<Path>,
    {
        video::thumbnail(video_path).await
    }

    pub async fn dump_frame<P>(
        video_path: P, 
        image_path: P,
        cols: usize, 
        rows: usize,
    ) -> Result<()>
    where P: AsRef<Path>,
    {
        let nframes = rows * cols;
        let dump = video::dump_frame(video_path, nframes).await?;
        video::concat_frames(dump, image_path, cols, rows).await?;

        Ok(())
    }
}
