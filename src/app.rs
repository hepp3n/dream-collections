// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::gql::{Data, Vars};
use crate::items::{AllSets, ClassSets, Item, ItemHasOption, ItemOptionType, SetItems};
use crate::{fl, gql};
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, about::About, menu, nav_bar};
use gql_client::Client;
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const ENDPOINT: &str = "https://mudream.online/api/graphql";

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),

    UpdateCurrentSet(usize),
    UpdateItem(Arc<Mutex<Item>>, ItemOptionType, ItemHasOption),

    SaveCollections,
    SearchMarket(Arc<Mutex<Item>>),
    ClearOffers,

    MarketSearchResult((String, Data)),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCollection {
    pub collection: Vec<ClassSets>,
}

impl Default for PlayerCollection {
    fn default() -> Self {
        PlayerCollection {
            collection: vec![
                ClassSets::DarkWizard(vec![
                    SetItems::new(AllSets::Pad),
                    SetItems::new(AllSets::Bone),
                    SetItems::new(AllSets::Sphinx),
                    SetItems::new(AllSets::Legendary),
                    SetItems::new(AllSets::GrandSoul),
                    SetItems::new(AllSets::DarkSoul),
                    SetItems::new(AllSets::VenomMist),
                ]),
                ClassSets::DarkKnight(vec![
                    SetItems::new(AllSets::Leather),
                    SetItems::new(AllSets::Bronze),
                    SetItems::new(AllSets::Scale),
                    SetItems::new(AllSets::Brass),
                    SetItems::new(AllSets::Plate),
                    SetItems::new(AllSets::Dragon),
                    SetItems::new(AllSets::BlackDragon),
                    SetItems::new(AllSets::DarkPhoenix),
                    SetItems::new(AllSets::GreatDragon),
                    SetItems::new(AllSets::DragonKnight),
                ]),
                ClassSets::Elf(vec![
                    SetItems::new(AllSets::Vine),
                    SetItems::new(AllSets::Silk),
                    SetItems::new(AllSets::Wind),
                    SetItems::new(AllSets::Spirit),
                    SetItems::new(AllSets::Guardian),
                    SetItems::new(AllSets::HolySpirit),
                    SetItems::new(AllSets::RedSpirit),
                ]),
                ClassSets::Summoner(vec![
                    SetItems::new(AllSets::ViolentWind),
                    SetItems::new(AllSets::RedWinged),
                    SetItems::new(AllSets::Ancient),
                    SetItems::new(AllSets::Demonic),
                    SetItems::new(AllSets::StormBlitz),
                    SetItems::new(AllSets::Succubus),
                ]),
                ClassSets::MagicGladiator(vec![
                    SetItems::new(AllSets::Pad),
                    SetItems::new(AllSets::Leather),
                    SetItems::new(AllSets::Bronze),
                    SetItems::new(AllSets::Bone),
                    SetItems::new(AllSets::Scale),
                    SetItems::new(AllSets::Sphinx),
                    SetItems::new(AllSets::Brass),
                    SetItems::new(AllSets::Plate),
                    SetItems::new(AllSets::Legendary),
                    SetItems::new(AllSets::Dragon),
                    SetItems::new(AllSets::StormCrow),
                    SetItems::new(AllSets::ThunderHawk),
                    SetItems::new(AllSets::Hurricane),
                    SetItems::new(AllSets::Volcano),
                ]),
                ClassSets::DarkLord(vec![
                    SetItems::new(AllSets::Leather),
                    SetItems::new(AllSets::Bronze),
                    SetItems::new(AllSets::Scale),
                    SetItems::new(AllSets::LightPlate),
                    SetItems::new(AllSets::Adamantine),
                    SetItems::new(AllSets::DarkSteel),
                    SetItems::new(AllSets::DarkMaster),
                    SetItems::new(AllSets::Sunlight),
                ]),
                ClassSets::RageFighter(vec![
                    SetItems::new(AllSets::Leather),
                    SetItems::new(AllSets::Scale),
                    SetItems::new(AllSets::Brass),
                    SetItems::new(AllSets::Plate),
                    SetItems::new(AllSets::SacredFire),
                    SetItems::new(AllSets::StormZahard),
                    SetItems::new(AllSets::PiercingGrove),
                    SetItems::new(AllSets::PhoenixSoul),
                ]),
            ],
        }
    }
}

impl PlayerCollection {
    pub fn update_class_item(
        &mut self,
        item: Arc<Mutex<Item>>,
        option: ItemOptionType,
        enabled: ItemHasOption,
    ) {
        let item_guard = item.lock().unwrap();

        item_guard.options.lock().unwrap().0.insert(option, enabled);
    }
}

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Configuration data that persists between application runs.
    config: Config,
    // collections
    config_dir: PathBuf,
    collections: PlayerCollection,
    current_set_index: usize,

    offers: (String, Vec<gql::Item>),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "dev.heppen.dream.collections";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create a nav bar with three page items.
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text("Dark Wizard")
            .data::<Page>(Page::DarkWizard)
            .activate();

        nav.insert()
            .text("Dark Knight")
            .data::<Page>(Page::DarkKnight);

        nav.insert().text("Elf").data::<Page>(Page::Elf);

        nav.insert().text("Summoner").data::<Page>(Page::Summoner);

        nav.insert()
            .text("Magic Gladiator")
            .data::<Page>(Page::MagicGladiator);

        nav.insert().text("Dark Lord").data::<Page>(Page::DarkLord);

        nav.insert()
            .text("Rage Figher")
            .data::<Page>(Page::RageFighter);

        // Create the about widget
        let about = About::default()
            .name(fl!("app-title"))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        let config_dir = dirs::config_dir().unwrap_or_default();

        let app_dir = config_dir.join("dream_collections");

        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir).unwrap();
        }

        let file_path = app_dir.join("collections.ron");

        if !file_path.exists() {
            std::fs::File::create(&file_path).unwrap();
        }

        let collections: PlayerCollection = {
            let data = std::fs::read_to_string(&file_path).unwrap_or_default();

            if data.is_empty() {
                PlayerCollection::default()
            } else {
                ron::from_str(&data).unwrap_or_default()
            }
        };

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            nav,
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            config_dir: file_path,
            collections,
            current_set_index: 0,

            offers: (String::new(), vec![]),
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<_> = match self.nav.active_data::<Page>().unwrap() {
            Page::DarkWizard => {
                let set_items = self
                    .collections
                    .collection
                    .iter()
                    .find_map(|class_set| {
                        if let ClassSets::DarkWizard(sets) = class_set {
                            Some(sets)
                        } else {
                            None
                        }
                    })
                    .unwrap();

                self.view_collections("Dark Wizard", set_items)
            }

            Page::DarkKnight => {
                let set_items = self
                    .collections
                    .collection
                    .iter()
                    .find_map(|class_set| {
                        if let ClassSets::DarkKnight(sets) = class_set {
                            Some(sets)
                        } else {
                            None
                        }
                    })
                    .unwrap();

                self.view_collections("Dark Knight", set_items)
            }

            Page::Elf => {
                let set_items = self
                    .collections
                    .collection
                    .iter()
                    .find_map(|class_set| {
                        if let ClassSets::Elf(sets) = class_set {
                            Some(sets)
                        } else {
                            None
                        }
                    })
                    .unwrap();

                self.view_collections("Elf", set_items)
            }

            Page::Summoner => {
                let set_items = self
                    .collections
                    .collection
                    .iter()
                    .find_map(|class_set| {
                        if let ClassSets::Summoner(sets) = class_set {
                            Some(sets)
                        } else {
                            None
                        }
                    })
                    .unwrap();

                self.view_collections("Summoner", set_items)
            }
            Page::MagicGladiator => {
                let set_items = self
                    .collections
                    .collection
                    .iter()
                    .find_map(|class_set| {
                        if let ClassSets::MagicGladiator(sets) = class_set {
                            Some(sets)
                        } else {
                            None
                        }
                    })
                    .unwrap();

                self.view_collections("Magic Gladiator", set_items)
            }
            Page::DarkLord => {
                let set_items = self
                    .collections
                    .collection
                    .iter()
                    .find_map(|class_set| {
                        if let ClassSets::DarkLord(sets) = class_set {
                            Some(sets)
                        } else {
                            None
                        }
                    })
                    .unwrap();

                self.view_collections("Dark Lord", set_items)
            }
            Page::RageFighter => {
                let set_items = self
                    .collections
                    .collection
                    .iter()
                    .find_map(|class_set| {
                        if let ClassSets::RageFighter(sets) = class_set {
                            Some(sets)
                        } else {
                            None
                        }
                    })
                    .unwrap();

                self.view_collections("Rage Fighter", set_items)
            }
        };

        widget::container(content)
            .width(600)
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ];

        // Conditionally enables a timer that emits a message every second.
        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },

            Message::UpdateCurrentSet(index) => {
                self.current_set_index = index;
            }
            Message::UpdateItem(item, option, enabled) => {
                self.collections.update_class_item(item, option, enabled);
            }
            Message::SaveCollections => {
                let data = to_string_pretty(&self.collections, PrettyConfig::new()).unwrap();

                if let Err(err) = std::fs::write(&self.config_dir, data) {
                    eprintln!("failed to save collections: {err}");
                }
            }
            Message::ClearOffers => {
                self.offers.1.clear();
            }
            Message::SearchMarket(item) => {
                let item_guard = item.lock().unwrap();
                let query = item_guard.generate_market_query();
                let vars = item_guard.generate_gql_vars();

                let item_title = format!(
                    "{} {}",
                    item_guard.name.clone().unwrap_or_default(),
                    item_guard.item_type.clone().unwrap_or_default()
                );

                return Task::future(async move {
                    let client = Client::new(ENDPOINT);

                    let result = client
                        .query_with_vars::<Data, Vars>(&query, vars)
                        .await
                        .unwrap();

                    cosmic::Action::App(Message::MarketSearchResult((
                        item_title,
                        result.expect("No data"),
                    )))
                });
            }

            Message::MarketSearchResult((item, data)) => {
                // Handle market search results here
                println!("Received market search results");

                for lot in data.lots.lots {
                    self.offers.0 = item.clone();
                    self.offers.1.push(lot);
                }
            }
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Activate the page in the model.

        self.offers.1.clear();

        self.nav.activate(id);

        self.update_title()
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    pub fn view_collections(&self, title: &str, sets: &[SetItems]) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;

        let header = widget::row::with_capacity(2)
            .push(widget::text::title1(title.to_string()))
            .push(
                widget::button::standard("Zapisz kolekcje do bazy")
                    .on_press(Message::SaveCollections),
            )
            .push(widget::button::standard("Wyczyść oferty").on_press(Message::ClearOffers))
            .align_y(Alignment::End)
            .spacing(space_s);

        let dropdown_sets = sets
            .iter()
            .map(|set| set.to_string())
            .collect::<Vec<String>>();

        let set_selector = cosmic::widget::settings::section().add(
            cosmic::widget::settings::item::builder("Set").control(widget::dropdown::dropdown(
                dropdown_sets,
                Some(self.current_set_index),
                Message::UpdateCurrentSet,
            )),
        );

        let set_parts = cosmic::widget::scrollable({
            let mut col = cosmic::widget::column().spacing(space_s);

            let items = sets[self.current_set_index].items.clone();

            for item in items {
                let title = item.lock().unwrap().clone().item_type.unwrap().to_string();

                col = col.push(cosmic::widget::container({
                    let mut section = cosmic::widget::settings::section().title(title);

                    let options = item.lock().unwrap().clone();

                    for (option, enabled) in options.options.lock().unwrap().0.clone() {
                        let value = item.clone();
                        section = section.add(
                            cosmic::widget::settings::item::builder(option.to_string()).control(
                                widget::toggler(enabled).on_toggle(move |enabled| {
                                    Message::UpdateItem(value.clone(), option.clone(), enabled)
                                }),
                            ),
                        );
                    }

                    section = section.add(
                        cosmic::widget::settings::item::builder("Przeszukaj market").control(
                            widget::button::standard("Start")
                                .on_press(Message::SearchMarket(item.clone())),
                        ),
                    );

                    section
                }))
            }

            widget::container(col)
                .padding(10.0)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
        })
        .spacing(space_s);

        widget::column()
            .push(header)
            .push(self.view_offers())
            .push_maybe(if self.offers.1.is_empty() {
                set_selector.into()
            } else {
                None
            })
            .push_maybe(if self.offers.1.is_empty() {
                set_parts.into()
            } else {
                None
            })
            .spacing(space_s)
            .into()
    }

    pub fn view_offers(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;

        let list = cosmic::widget::scrollable({
            let mut col = cosmic::widget::column().spacing(space_s);

            for item in self.offers.1.iter() {
                let title = item.gear_score.unwrap_or_default().to_string();

                col = col.push(cosmic::widget::container(
                    cosmic::widget::settings::section()
                        .title(format!("Gear score: {} ", title))
                        .add(widget::settings::item_row({
                            let mut row = vec![];

                            for price in item.prices.iter() {
                                let currency = &price.currency;
                                let value = price.value.unwrap_or_default();

                                let currency_title = currency.title.as_deref().unwrap_or("Unknown");

                                row.push(
                                    widget::text::body(format!("{}: {:.2}", currency_title, value))
                                        .into(),
                                );
                            }

                            row
                        })),
                ))
            }

            widget::container(col)
                .padding(10.0)
                .align_x(Horizontal::Center)
        })
        .spacing(space_s);

        widget::column::with_capacity(2)
            .push(if self.offers.1.is_empty() {
                cosmic::widget::settings::section().title("Brak ofert. Szukaj dalej.")
            } else {
                cosmic::widget::settings::section()
                    .title(format!("Oferty z marketu dla: {}", self.offers.0))
            })
            .push(list)
            .spacing(space_s)
            .into()
    }
}

/// The page to display in the application.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Page {
    DarkWizard,
    DarkKnight,
    Elf,
    Summoner,
    MagicGladiator,
    DarkLord,
    RageFighter,
}

impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Page::DarkWizard => "Dark Wizard",
            Page::DarkKnight => "Dark Knight",
            Page::Elf => "Elf",
            Page::Summoner => "Summoner",
            Page::MagicGladiator => "Magic Gladiator",
            Page::DarkLord => "Dark Lord",
            Page::RageFighter => "Rage Fighter",
        };

        write!(f, "{name}")
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}
