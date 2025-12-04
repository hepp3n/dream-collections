// SPDX-License-Identifier: MPL-2.0

use crate::gql;
use crate::gql::{Data, Vars};
use crate::items::{AllSets, ClassSets, Item, ItemHasOption, ItemOptionType, SetItems};
use gql_client::Client;
use iced::alignment::Horizontal;
use iced::widget::{Container, container, horizontal_rule, row};
use iced::{Alignment, Color, Element, Font, Length, Pixels, Task, widget};
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const ENDPOINT: &str = "https://mudream.online/api/graphql";

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    ChangePage(Page),
    ChangeSet(String),

    UpdateItem(Arc<Mutex<Item>>, ItemOptionType, ItemHasOption),

    SaveCollections,
    SearchMarket(Arc<Mutex<Item>>),
    ClearOffers,

    MarketSearchResult((String, Option<Data>)),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCollection {
    pub collection: Vec<Arc<Mutex<ClassSets>>>,
}

impl Default for PlayerCollection {
    fn default() -> Self {
        PlayerCollection {
            collection: vec![
                Arc::new(Mutex::new(ClassSets::DarkWizard(vec![
                    SetItems::new(AllSets::Pad),
                    SetItems::new(AllSets::Bone),
                    SetItems::new(AllSets::Sphinx),
                    SetItems::new(AllSets::Legendary),
                    SetItems::new(AllSets::GrandSoul),
                    SetItems::new(AllSets::DarkSoul),
                    SetItems::new(AllSets::VenomMist),
                ]))),
                Arc::new(Mutex::new(ClassSets::DarkKnight(vec![
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
                ]))),
                Arc::new(Mutex::new(ClassSets::Elf(vec![
                    SetItems::new(AllSets::Vine),
                    SetItems::new(AllSets::Silk),
                    SetItems::new(AllSets::Wind),
                    SetItems::new(AllSets::Spirit),
                    SetItems::new(AllSets::Guardian),
                    SetItems::new(AllSets::HolySpirit),
                    SetItems::new(AllSets::RedSpirit),
                ]))),
                Arc::new(Mutex::new(ClassSets::Summoner(vec![
                    SetItems::new(AllSets::ViolentWind),
                    SetItems::new(AllSets::RedWinged),
                    SetItems::new(AllSets::Ancient),
                    SetItems::new(AllSets::Demonic),
                    SetItems::new(AllSets::StormBlitz),
                    SetItems::new(AllSets::Succubus),
                ]))),
                Arc::new(Mutex::new(ClassSets::MagicGladiator(vec![
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
                ]))),
                Arc::new(Mutex::new(ClassSets::DarkLord(vec![
                    SetItems::new(AllSets::Leather),
                    SetItems::new(AllSets::Bronze),
                    SetItems::new(AllSets::Scale),
                    SetItems::new(AllSets::LightPlate),
                    SetItems::new(AllSets::Adamantine),
                    SetItems::new(AllSets::DarkSteel),
                    SetItems::new(AllSets::DarkMaster),
                    SetItems::new(AllSets::Sunlight),
                ]))),
                Arc::new(Mutex::new(ClassSets::RageFighter(vec![
                    SetItems::new(AllSets::Leather),
                    SetItems::new(AllSets::Scale),
                    SetItems::new(AllSets::Brass),
                    SetItems::new(AllSets::Plate),
                    SetItems::new(AllSets::SacredFire),
                    SetItems::new(AllSets::StormZahard),
                    SetItems::new(AllSets::PiercingGrove),
                    SetItems::new(AllSets::PhoenixSoul),
                ]))),
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

pub struct AppModel {
    page: Page,
    config_dir: PathBuf,
    collections: PlayerCollection,
    current_class: Arc<Mutex<ClassSets>>,
    current_set: Option<SetItems>,

    set_options: Vec<SetItems>,
    set_selected: Option<String>,

    offers: (String, Vec<gql::Item>),
}

impl Default for AppModel {
    fn default() -> Self {
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

        let current_class = collections
            .collection
            .iter()
            .find(|c| matches!(*c.lock().unwrap(), ClassSets::DarkWizard(_)))
            .cloned()
            .unwrap_or_else(|| Arc::new(Mutex::new(ClassSets::DarkWizard(vec![]))));

        let set_options = match &*current_class.lock().unwrap() {
            ClassSets::DarkWizard(sets) => sets.clone(),
            ClassSets::DarkKnight(sets) => sets.clone(),
            ClassSets::Elf(sets) => sets.clone(),
            ClassSets::Summoner(sets) => sets.clone(),
            ClassSets::MagicGladiator(sets) => sets.clone(),
            ClassSets::DarkLord(sets) => sets.clone(),
            ClassSets::RageFighter(sets) => sets.clone(),
        };

        // Construct the app model with the runtime's core.
        AppModel {
            page: Page::DarkWizard,
            config_dir: file_path,
            collections,
            current_class,
            current_set: None,

            set_options,
            set_selected: None,

            offers: (String::new(), vec![]),
        }
    }
}

impl AppModel {
    pub fn title(&self) -> String {
        format!("Dream Collections by Nemessis - {}", REPOSITORY)
    }

    pub fn view(&self) -> Element<'_, Message> {
        widget::container(self.view_collections()).into()
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangePage(page) => {
                self.set_options.clear();

                self.page = page;
                self.current_class = self
                    .collections
                    .collection
                    .iter()
                    .find(|c| {
                        matches!(
                            (self.page, &*c.lock().unwrap()),
                            (Page::DarkWizard, ClassSets::DarkWizard(_))
                                | (Page::DarkKnight, ClassSets::DarkKnight(_))
                                | (Page::Elf, ClassSets::Elf(_))
                                | (Page::Summoner, ClassSets::Summoner(_))
                                | (Page::MagicGladiator, ClassSets::MagicGladiator(_))
                                | (Page::DarkLord, ClassSets::DarkLord(_))
                                | (Page::RageFighter, ClassSets::RageFighter(_))
                        )
                    })
                    .cloned()
                    .unwrap();

                self.set_options = match &*self.current_class.lock().unwrap() {
                    ClassSets::DarkWizard(sets) => sets.clone(),
                    ClassSets::DarkKnight(sets) => sets.clone(),
                    ClassSets::Elf(sets) => sets.clone(),
                    ClassSets::Summoner(sets) => sets.clone(),
                    ClassSets::MagicGladiator(sets) => sets.clone(),
                    ClassSets::DarkLord(sets) => sets.clone(),
                    ClassSets::RageFighter(sets) => sets.clone(),
                };
            }
            Message::ChangeSet(set) => {
                self.set_selected = Some(set);
                self.current_set = self
                    .set_options
                    .iter()
                    .find(|s| s.set_string == self.set_selected.clone().unwrap())
                    .cloned();
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
                self.offers.0 = String::new();
                self.offers.1.clear();
            }
            Message::SearchMarket(item) => {
                self.offers.0 = String::new();
                self.offers.1.clear();

                let item_guard = item.lock().unwrap();
                let query = item_guard.generate_market_query();
                let vars = item_guard.generate_gql_vars();

                let item_title = format!(
                    "{} {}",
                    item_guard.name.clone().unwrap_or_default(),
                    item_guard.item_type.clone().unwrap_or_default()
                );

                return iced::Task::future(async move {
                    let client = Client::new(ENDPOINT);

                    let result = client
                        .query_with_vars::<Data, Vars>(&query, vars)
                        .await
                        .unwrap();

                    Message::MarketSearchResult((item_title, result))
                });
            }

            Message::MarketSearchResult((item, data)) => {
                if let Some(data) = data {
                    for lot in data.lots.lots {
                        self.offers.1.push(lot);
                    }
                }
                self.offers.0 = format!("Znaleziono {} ofert dla {}", self.offers.1.len(), item);
            }
        }

        Task::none()
    }

    pub fn view_collections(&self) -> Container<'_, Message> {
        let buttons = container(row(vec![
            widget::pick_list(&Page::ALL[..], Some(self.page), Message::ChangePage)
                .placeholder("Wybierz klasę")
                .into(),
            widget::pick_list(
                self.set_options
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
                self.set_selected.clone(),
                Message::ChangeSet,
            )
            .placeholder("Wybierz set")
            .into(),
            widget::button("Wyczyść oferty")
                .on_press(Message::ClearOffers)
                .into(),
            widget::button("Zapisz kolekcje")
                .on_press(Message::SaveCollections)
                .into(),
        ]))
        .padding(10)
        .center_x(Length::Fill);

        let mut content = widget::column!()
            .push(buttons)
            .push(horizontal_rule(Pixels::from(2)));

        let mut item_parts = widget::column!().spacing(15);

        if let Some(set) = self.current_set.as_ref() {
            for item in set.items.iter() {
                let item_guard = item.lock().unwrap();

                let mut row = widget::row!().spacing(10).width(Length::Fill);

                let item_name = format!(
                    "{} {}",
                    item_guard.name.clone().unwrap_or_default(),
                    item_guard.item_type.clone().unwrap_or_default()
                );

                row = row.push(widget::container(
                    widget::button(widget::text!("{}", item_name).align_x(Alignment::Center))
                        .on_press(Message::SearchMarket(item.clone()))
                        .height(Length::Fixed(200.0))
                        .width(Length::Fixed(150.0)),
                ));

                let options = item_guard.options.lock().unwrap();
                let mut col = widget::column!();

                for (option, has_option) in options.0.clone() {
                    col = col.push(
                        widget::container(
                            widget::checkbox(option.to_string(), has_option)
                                .on_toggle(move |enabled| {
                                    let item_clone = item.clone();
                                    Message::UpdateItem(item_clone, option.clone(), enabled)
                                })
                                .spacing(10),
                        )
                        .height(Length::Fixed(30.0)),
                    );
                }
                row = row.push(widget::container(col).center_y(Length::Fixed(200.0)));
                item_parts = item_parts.push(row);
            }
        }

        let offers_container = self.view_offers();

        let row = widget::row!()
            .spacing(20)
            .push(
                widget::scrollable(item_parts)
                    .width(Length::FillPortion(3))
                    .spacing(16),
            )
            .push(
                widget::scrollable(offers_container)
                    .width(Length::FillPortion(2))
                    .spacing(16),
            );

        content = content.push(row);

        widget::container(content)
            .padding(30)
            .align_x(Horizontal::Center)
    }

    pub fn view_offers(&self) -> Container<'_, Message> {
        let mut col = widget::column!();

        col = col.push(widget::text(self.offers.0.clone()).size(24));

        for item in self.offers.1.iter() {
            col = col.push(widget::container({
                let mut colu = widget::column!().spacing(8);

                colu = colu.push(
                    widget::row!()
                        .spacing(10)
                        .push(widget::text("Gear Score"))
                        .push(
                            widget::text(item.gear_score.unwrap_or_default())
                                .font(Font::MONOSPACE)
                                .color(Color::from_rgb(0.8, 0.2, 0.2)),
                        ),
                );

                let mut row = widget::row!().spacing(10);

                for price in item.prices.iter() {
                    let currency = &price.currency;
                    let value = price.value.unwrap_or_default();

                    let currency_title = currency.title.as_deref().unwrap_or("Unknown");

                    row = row.push(
                        widget::column!()
                            .push(
                                widget::text(currency_title).color(Color::from_rgb(0.2, 0.6, 0.8)),
                            )
                            .push(
                                widget::text(format!("{value}"))
                                    .font(Font::MONOSPACE)
                                    .size(20),
                            ),
                    );
                }
                colu = colu.push(row);

                colu = colu.push(widget::horizontal_rule(Pixels::from(1)));

                colu
            }));
        }

        widget::container(col)
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

impl Page {
    pub const ALL: [Page; 7] = [
        Page::DarkWizard,
        Page::DarkKnight,
        Page::Elf,
        Page::Summoner,
        Page::MagicGladiator,
        Page::DarkLord,
        Page::RageFighter,
    ];
}
