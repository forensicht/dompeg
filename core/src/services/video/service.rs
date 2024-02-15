use std::path::Path;

use anyhow::Result;

const MEDIA_TYPE_VIDEOS: &[&str] = &[
    "mpeg", "mpg", "mp4", "avi", "ogg", "webm", "flv",
];

#[derive(Debug, Clone)]
pub struct Video {
    pub name: String,
    pub path: String,
    pub size: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct VideoService;

impl VideoService {
    pub fn search_videos<P: AsRef<Path>>(path: P) -> Result<Vec<Video>> {
        let mut videos = Vec::new();
        let mut dir = tokio::fs::read_dir(path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            let metadata = tokio::fs::metadata(&path).await?;

            if metadata.is_dir() || !Self::is_video(&path) {
                continue;
            }

            let video = Video {
                name: entry.file_name().to_str().unwrap().to_owned(),
                path: entry.path().to_str().unwrap().to_owned(),
                size: (metadata.len() as f64 / 1024.0_f64).round() as usize,
            };

            videos.push(video);
        }
        
        Ok(videos)
    }

    fn is_video(entry: &Path) -> bool {
        match entry.extension() {
            Some(ext) if MEDIA_TYPE_VIDEOS.contains(
                &ext.to_string_lossy().to_lowercase().as_str()
            ) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_videos() {
        let path = Path::new("D:\\Videos");
        match VideoService::search_videos(path) {
            Ok(videos) => println!("{:#?}", videos),
            Err(err) => eprintln!("{:?}", err),
        }
    }
}
