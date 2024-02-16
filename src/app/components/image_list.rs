use std::path::{PathBuf, Path};
use anyhow::Result;

use crate::fl;
use crate::app::{
    models,
    factories::image::{
        ImageModel,
        ImageInput,
    },
};
use core_dompeg as core;

use relm4::{
    prelude::*,
    gtk::prelude::*,
    factory::AsyncFactoryVecDeque,
    component::{
        AsyncComponent,
        AsyncComponentSender,
        AsyncComponentParts,
    },
};
use relm4_icons::icon_name;

#[derive(Debug)]
pub struct ImageListModel {
    image_list_factory: AsyncFactoryVecDeque<ImageModel>,
    total_videos: usize,
    processed_videos: usize,
    thumbnail_size: i32,
}

impl ImageListModel {
    pub fn new(image_list_factory: AsyncFactoryVecDeque<ImageModel>) -> Self {
        Self {
            image_list_factory,
            total_videos: 0,
            processed_videos: 0,
            thumbnail_size: models::image::THUMBNAIL_SIZE,
        }
    }
}

#[derive(Debug)]
pub enum ImageListInput {
    ZoomIn,
    ZoomOut,
    ClearImageList,
    ConvertVideos(Vec<String>, models::LayoutType, PathBuf),
    ViewImage(usize),
    SearchEntry(String),
}

#[derive(Debug)]
pub enum ImageListOutput {
    VideoConversionCompleted,
    ImageCount(usize),
    Notify(String, u32),
}

#[derive(Debug)]
pub enum ImageListCommandOutput {
    VideoConversionCompleted(Result<models::Image>),
}

#[relm4::component(pub async)]
impl AsyncComponent for ImageListModel {
    type Init = ();
    type Input = ImageListInput;
    type Output = ImageListOutput;
    type CommandOutput = ImageListCommandOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_bottom: 3,
            set_margin_end: 6,
            set_margin_start: 6,
            set_margin_top: 4,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_margin_bottom: 4,
                set_spacing: 6,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_width_request: 212,
                    set_hexpand: false,
                    set_halign: gtk::Align::Start,
                    set_spacing: 6,

                    gtk::Button {
                        set_icon_name: icon_name::ERASER,
                        set_tooltip: fl!("clear-image-list"),
                        add_css_class: "flat",
                        connect_clicked => ImageListInput::ClearImageList,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_margin_start: 12,
                    set_margin_end: 12,
                    set_halign: gtk::Align::Center,
                    set_spacing: 12,
    
                    gtk::Button {
                        set_icon_name: icon_name::MINUS_LARGE,
                        set_tooltip: fl!("zoom-out"),
                        add_css_class: "circular",
                        connect_clicked => ImageListInput::ZoomOut,
                    },
    
                    gtk::Button {
                        set_icon_name: icon_name::PLUS_LARGE,
                        set_tooltip: fl!("zoom-in"),
                        add_css_class: "circular",
                        connect_clicked => ImageListInput::ZoomIn,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_width_request: 212,
                    set_hexpand: false,
                    set_spacing: 6,

                    gtk::SearchEntry {
                        set_placeholder_text: Some(fl!("search")),
                        set_search_delay: 500,
                        connect_search_changed[sender] => move |entry| {
                            let query = entry.text().as_str().to_string();
                            sender.input(ImageListInput::SearchEntry(query));
                        }
                    },
                }
            },

            gtk::Frame {
                set_css_classes: &["view"],

                gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_hexpand: true,
                    set_vexpand: true,

                    #[local_ref]
                    image_list_widget -> gtk::FlowBox {
                        set_css_classes: &["list-padding-bottom"],
                        set_valign: gtk::Align::Start,
                        set_max_children_per_line: 16,
                        set_selection_mode: gtk::SelectionMode::None,
                        set_activate_on_single_click: false,
                        connect_child_activated[sender] => move |_, child| {
                            let index = child.index() as usize;
                            sender.input(ImageListInput::ViewImage(index));
                        },
                    },
                }
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let image_list_factory = AsyncFactoryVecDeque::builder()
            .launch_default()
            .detach();

        let model = ImageListModel::new(image_list_factory);
        let image_list_widget = model.image_list_factory.widget();
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ImageListInput::ZoomIn => {
                self.apply_video_zoom(true).await;
            }
            ImageListInput::ZoomOut => {
                self.apply_video_zoom(false).await;
            }
            ImageListInput::ClearImageList => {
                self.image_list_factory.guard().clear();
                sender.output(ImageListOutput::ImageCount(0)).unwrap_or_default();
            }
            ImageListInput::ConvertVideos(video_list, layout_type, dst_path) => {
                self.total_videos = video_list.len();
                self.processed_videos = 0;
                self.on_convert_videos(video_list, layout_type, dst_path, &sender).await;
            }
            ImageListInput::SearchEntry(query) => {
               self.apply_image_filter(&query).await;
            }
            ImageListInput::ViewImage(index) => {
                self.on_view_image(index, &sender).await;
            }
        }    
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ImageListCommandOutput::VideoConversionCompleted(result) => {
                match result {
                    Ok(img) => {
                        let mut guard = self.image_list_factory.guard();
                        guard.push_back(img);
                        let count = guard.len();
                        sender.output(ImageListOutput::ImageCount(count)).unwrap_or_default();
                    }
                    Err(err) => {
                        let msg_error = format!("{}: {}", fl!("generic-error"), err);
                        tracing::error!(msg_error);
                        sender.output(ImageListOutput::Notify(msg_error, 3)).unwrap_or_default();
                    }
                }

                self.processed_videos += 1;
                if self.processed_videos == self.total_videos {
                    sender.output(ImageListOutput::VideoConversionCompleted).unwrap_or_default();
                }
            }
        }
    }
}

impl ImageListModel {
    async fn on_convert_videos(
        &mut self,
        video_list: Vec<String>,
        layout_type: models::LayoutType,
        dst_path: PathBuf,
        sender: &AsyncComponentSender<ImageListModel>,
    ) {
        let cols = layout_type.value().1;
        let rows = layout_type.value().0;
        let layout_name = layout_type.to_string();

        for video_path in video_list.iter() {
            if let Some(filename) = Path::new(video_path).file_stem() {
                let filename = format!("{}_{}", layout_name, filename.to_str().unwrap_or_default());
                let mut image_path = dst_path.clone().join(filename);
                image_path.set_extension("jpeg");

                let image_path = image_path.to_str().unwrap().to_string(); 
                let video_path = video_path.to_owned();
                let thumbnail_size = self.thumbnail_size;

                sender.oneshot_command(async move {
                    match core::video::dump_frames_to_image(
                        video_path, 
                        image_path.clone(), 
                        cols, 
                        rows,
                    ).await {
                        Ok(_) => {
                            let filename = if let Some(filename) = Path::new(&image_path).file_name() {
                                filename.to_str().unwrap().to_string()
                            } else {
                                String::new()
                            };
                            let img = models::Image {
                                name: filename,
                                path: image_path,
                                thumbnail_size: thumbnail_size,
                            };
                            ImageListCommandOutput::VideoConversionCompleted(Ok(img))
                        }
                        Err(err) => ImageListCommandOutput::VideoConversionCompleted(Err(err))
                    }
                });
            }
        }
    }

    async fn apply_image_filter(&mut self, query: &str) {
        let image_widget = self.image_list_factory.widget();

        for image_model in self.image_list_factory.iter() {
            let image_model = image_model.unwrap();
            let image = &image_model.image;
            let is_visible = if query.is_empty() {
                true
            } else {
                image.name.to_lowercase().contains(&query.to_lowercase())
            };
            let index = image_model.index.current_index() as i32;
            image_widget
                .child_at_index(index)
                .as_ref()
                .unwrap()
                .set_visible(is_visible);
        }
    }

    async fn on_view_image(
        &mut self,
        index: usize,
        sender: &AsyncComponentSender<ImageListModel>,
    ) {
        if let Some(image_model) = self.image_list_factory.guard().get(index) {
            let image_name = image_model.image.path.as_str();
            match open::that(image_name) {
                Err(_) => {
                    let msg = format!("{} {}", 
                        fl!("open-image-error"), image_model.image.name.as_str());
                    sender.output(ImageListOutput::Notify(msg, 3)).unwrap_or_default();
                }
                _ => {}
            }
        }
    }

    async fn apply_video_zoom(&mut self, is_zoom_in: bool) {
        use models::image::THUMBNAIL_SIZE;
        use models::image::ZOOM_SIZE;

        if is_zoom_in {
            if self.thumbnail_size < 320 {
                self.thumbnail_size += ZOOM_SIZE;
            }
        } else {
            if self.thumbnail_size > THUMBNAIL_SIZE {
                let mut thumb_size = self.thumbnail_size - ZOOM_SIZE;
                if thumb_size < THUMBNAIL_SIZE {
                    thumb_size = THUMBNAIL_SIZE;
                }
                self.thumbnail_size = thumb_size;
            }
        }

        for image_model in self.image_list_factory.iter() {
            let index = image_model.unwrap().index.current_index();
            if is_zoom_in {
                self.image_list_factory.send(index, ImageInput::ZoomIn(self.thumbnail_size));
            } else {
                self.image_list_factory.send(index, ImageInput::ZoomOut(self.thumbnail_size));
            }
        }
    }
}
