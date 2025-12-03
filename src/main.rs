// SPDX-License-Identifier: MPL-2.0

use crate::app::AppModel;

mod app;
mod gql;
mod items;

fn main() -> iced::Result {
    iced::application(AppModel::title, AppModel::update, AppModel::view)
        .centered()
        .run()
}
