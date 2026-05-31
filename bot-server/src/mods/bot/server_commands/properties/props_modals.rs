use serenity::builder::{CreateActionRow, CreateInputText, CreateModal};
use serenity::model::application::InputTextStyle;

pub fn props_modal(title: &str, id: &str, style: InputTextStyle) -> CreateModal {
    let input = CreateActionRow::InputText(CreateInputText::new(
        style,
        format!("Edit {}", title),
        format!("input:{}", title),
    ));
    CreateModal::new(format!("edit_props:{}:{}", title, id), title).components(vec![input])
}
