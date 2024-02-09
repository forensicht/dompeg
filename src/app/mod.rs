pub mod config;
pub mod components;
pub mod factories;
pub mod models;

use crate::fl;
use crate::app::components::{
    about_dialog::AboutDialog,
    search_bar::{
        SearchBarModel, 
        SearchBarInput,
        SearchBarOutput,
    },
    content::{
        ContentModel, 
        ContentInput,
        ContentOutput,
    },
    preferences::PreferencesModel,
};

use std::path::PathBuf;

use relm4::{
    prelude::*,
    gtk::prelude::*,
    gtk::glib,
    adw,
    adw::Toast,
    component::{
        AsyncComponent, 
        AsyncComponentParts, 
        AsyncController, 
        AsyncComponentController,
    },
    view, AsyncComponentSender,
    loading_widgets::LoadingWidgets,
    actions::{
        ActionGroupName, 
        RelmAction, 
        RelmActionGroup,
    },
    ComponentBuilder, 
    ComponentController, 
    Controller, 
    main_adw_application,
};
use relm4_icons::icon_name;

pub struct App {
    search_bar: AsyncController<SearchBarModel>,
    content: AsyncController<ContentModel>,
    preferences: AsyncController<PreferencesModel>,
    about_dialog: Option<Controller<AboutDialog>>,
    video_count: usize,
    filter_count: usize,
    image_count: usize,
}

impl App {
    pub fn new(
        search_bar: AsyncController<SearchBarModel>,
        content: AsyncController<ContentModel>,
        preferences: AsyncController<PreferencesModel>,
        about_dialog: Option<Controller<AboutDialog>>,
    ) -> Self {
        Self{
            search_bar,
            content,
            preferences,
            about_dialog,
            video_count: 0,
            filter_count: 0,
            image_count: 0,
        }
    }
}

#[derive(Debug)]
pub enum AppInput {
    OpenPreferences,
    StartSearch(PathBuf),
    SearchCompleted(usize),
    FilterCount(usize),
    ImageCount(usize),
    Notify(String, u32),
    Quit,
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");
relm4::new_stateless_action!(QuitAction, WindowActionGroup, "quit");

#[relm4::component(pub async)]
impl AsyncComponent for App {
    type Init = ();
    type Input = AppInput;
    type Output = ();
    type CommandOutput = ();

    menu! {
        primary_menu: {
            section! {
                preferences => PreferencesAction,
                about_domipeg => AboutAction,
                quit => QuitAction,
            }
        }
    }

    view! {
        #[root]
        main_window = adw::ApplicationWindow {
            set_default_size: (1280, 968),
            set_resizable: true,

            connect_close_request[sender] => move |_| {
                sender.input(AppInput::Quit);
                glib::Propagation::Stop
            },

            #[name = "content"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical, 

                #[name = "content_header"]
                append = &adw::HeaderBar {
                    set_hexpand: true,
                    set_css_classes: &["flat"],
                    set_show_start_title_buttons: false,
                    set_show_end_title_buttons: true,

                    pack_end = &gtk::MenuButton {
                        set_tooltip: fl!("menu"),
                        set_valign: gtk::Align::Center,
                        set_css_classes: &["flat"],
                        set_icon_name: icon_name::MENU,
                        set_menu_model: Some(&primary_menu),
                    },

                    #[wrap(Some)]
                    set_title_widget = model.search_bar.widget(),
                },

                #[name(overlay)]
                adw::ToastOverlay {
                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: true,

                        append: model.content.widget(),
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_margin_start: 5,
                    set_margin_end: 5,
                    set_margin_bottom: 5,

                    gtk::Label {
                        #[watch]
                        set_label: &format!("{}: {}/{}", fl!("videos"), model.filter_count, model.video_count),
                        set_margin_start: 6,
                        set_margin_end: 12,
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &format!("{}: {}", fl!("images"), model.image_count),
                        set_margin_start: 12,
                        set_margin_end: 12,
                        set_halign: gtk::Align::Start,
                    },
                },
            }
        }
    }

    fn init_loading_widgets(root: &mut Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local_ref]
            root {
                set_title: Some(fl!("dompeg")),
                set_default_size: (500, 350),
                set_resizable: false,

                #[name = "loading"]
                gtk::CenterBox {
                    set_margin_all: 50,
                    set_orientation: gtk::Orientation::Vertical,

                    #[wrap(Some)]
                    set_center_widget = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 3,
                        set_margin_bottom: 12,

                        gtk::Picture {
                            set_resource: Some("/com/github/forensicht/Dompeg/icons/Dompeg.svg"),
                        },

                        gtk::Label {
                            set_label: fl!("dompeg"),
                            set_css_classes: &["title-1"],
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                        },
                    },

                    #[wrap(Some)]
                    set_end_widget = &gtk::Spinner {
                        start: (),
                        set_size_request: (30, 30),
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                    },
                }
            }
        }

        Some(LoadingWidgets::new(root, loading))
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        relm4::tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let preferences: &str = fl!("preferences");
        let about_domipeg: &str = fl!("about");
        let quit: &str = fl!("quit");

        let mut actions = RelmActionGroup::<WindowActionGroup>::new();

        let search_bar_controller = SearchBarModel::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                SearchBarOutput::StartSearch(path) => AppInput::StartSearch(path),
                SearchBarOutput::Notify(msg, timeout) => AppInput::Notify(msg, timeout),
            });

        let content_controller = ContentModel::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                ContentOutput::SearchCompleted(count) => AppInput::SearchCompleted(count),
                ContentOutput::FilterCount(count) => AppInput::FilterCount(count),
                ContentOutput::ImageCount(count) => AppInput::ImageCount(count),
                ContentOutput::Notify(msg, timeout) => AppInput::Notify(msg, timeout),
            });

        let preferences_controller = PreferencesModel::builder()
            .launch(())
            .detach();

        let mut model = App::new(
            search_bar_controller,
            content_controller,
            preferences_controller,
            None,
        );

        let widgets = view_output!();

        let about_dialog = ComponentBuilder::default()
            .launch(widgets.main_window.upcast_ref::<gtk::Window>().clone())
            .detach();

        model.about_dialog = Some(about_dialog);

        let preferences_action = {
            let sender = sender.clone();
            RelmAction::<PreferencesAction>::new_stateless(move |_| {
                sender.input_sender().send(AppInput::OpenPreferences).unwrap_or_default();
            })
        };

        let about_action = {
            let sender = model.about_dialog.as_ref().unwrap().sender().clone();
            RelmAction::<AboutAction>::new_stateless(move |_| {
                sender.send(()).unwrap_or_default();
            })
        };

        let quit_action = {
            let sender = sender.clone();
            RelmAction::<QuitAction>::new_stateless(move |_| {
                sender.input_sender().send(AppInput::Quit).unwrap_or_default();
            })
        };

        actions.add_action(preferences_action);
        actions.add_action(about_action);
        actions.add_action(quit_action);

        widgets.main_window.insert_action_group(
            WindowActionGroup::NAME,
            Some(&actions.into_action_group()),
        );

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppInput::OpenPreferences => {
                self.preferences.widget().present();
            }
            AppInput::StartSearch(path) => {
                self.content.emit(ContentInput::StartSearch(path));
            }
            AppInput::SearchCompleted(count) => {
                self.video_count = count;
                self.filter_count = count;
                self.search_bar.emit(SearchBarInput::SearchCompleted);
            }
            AppInput::FilterCount(count) => {
                self.filter_count = count;
            }
            AppInput::ImageCount(count) => {
                self.image_count = count;
            }
            AppInput::Notify(msg, timeout) => {
                widgets.overlay.add_toast(toast(msg, timeout));
            }
            AppInput::Quit => {
                main_adw_application().quit();
            }
        }

        self.update_view(widgets, sender);
    }
}

pub fn toast<T: ToString>(title: T, timeout: u32) -> Toast {
    Toast::builder()
        .title(title.to_string().as_str())
        .timeout(timeout)
        .build()
}
