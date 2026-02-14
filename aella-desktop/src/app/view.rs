use crate::app::types::{Message, State};
use crate::app::ui::{editor::build_editor_area, sidebar::build_sidebar};
use iced::widget::{container, row};
use iced::{Element, Fill};

pub(crate) fn view(state: &State) -> Element<'_, Message> {
    let sidebar = build_sidebar(state);
    let editor_area = build_editor_area(state);
    let content = row![sidebar, editor_area].spacing(0);

    container(content).width(Fill).height(Fill).into()
}
