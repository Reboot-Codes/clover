// This example demonstrates how to use the menu widget

use iced::border::Radius;
use iced::widget::{
  button,
  row,
  text,
};
use iced::{
  Border,
  Color,
  Element,
  Length,
  alignment,
};

use iced_aw::menu::{
  self,
  Item,
  Menu,
};
use iced_aw::style::{
  Status,
  menu_bar::primary,
};
use iced_aw::{
  menu_bar,
  menu_items,
};
use iced_aw::{
  quad,
  widgets::InnerBounds,
};
use iced_fonts::REQUIRED_FONT;
use iced_fonts::required::{
  RequiredIcons,
  icon_to_string,
};

use crate::Message;

pub fn gen_menu_bar<'a>() -> iced::Element<'a, Message> {
  let menu_tpl_1 = |items| Menu::new(items).max_width(180.0).offset(15.0).spacing(5.0);
  let menu_tpl_2 = |items| Menu::new(items).max_width(180.0).offset(0.0).spacing(5.0);

  let file_menu = {
    #[rustfmt::skip]
    let sub1 = menu_tpl_2(menu_items!(
      (debug_button("Item"))
      (debug_button("Item"))
      (debug_button("Item"))
      (debug_button("Item"))
      (debug_button("Item"))
    ))
    .width(220.0);

    #[rustfmt::skip]
    menu_tpl_1(menu_items!(
      (debug_button("New Connection"))
      (debug_button("Item"))
      (submenu_button("A sub menu"), sub1)
      (debug_button("Item"))
      (debug_button("Item"))
      (debug_button("Item"))
    ))
    .width(140.0)
  };

  let mb = menu_bar!((debug_button_s("File"), file_menu))
    .draw_path(menu::DrawPath::Backdrop)
    .style(|theme: &iced::Theme, status: Status| menu::Style {
      path_border: Border {
        radius: Radius::new(6.0),
        ..Default::default()
      },
      ..primary(theme, status)
    });

  mb.into()
}

fn base_button<'a>(
  content: impl Into<Element<'a, Message>>,
  msg: Option<Message>,
) -> button::Button<'a, Message> {
  button(content)
    .padding([4, 8])
    .on_press_maybe(msg)
    .style(iced::widget::button::text)
}

fn debug_button(label: &str) -> button::Button<Message, iced::Theme, iced::Renderer> {
  labeled_button(label, Some(Message::None)).width(Length::Fill)
}

fn debug_button_s(label: &str) -> button::Button<Message, iced::Theme, iced::Renderer> {
  labeled_button(label, Some(Message::None)).width(Length::Shrink)
}

fn labeled_button(
  label: &str,
  msg: Option<Message>,
) -> button::Button<Message, iced::Theme, iced::Renderer> {
  base_button(text(label).align_y(alignment::Vertical::Center), msg)
}

fn submenu_button(label: &str) -> button::Button<Message, iced::Theme, iced::Renderer> {
  base_button(
    row![
      text(label)
        .width(Length::Fill)
        .align_y(alignment::Vertical::Center),
      text(icon_to_string(RequiredIcons::CaretRightFill))
        .font(REQUIRED_FONT)
        .width(Length::Shrink)
        .align_y(alignment::Vertical::Center),
    ]
    .align_y(iced::Alignment::Center),
    Some(Message::None),
  )
  .width(Length::Fill)
}

fn separator() -> quad::Quad {
  quad::Quad {
    quad_color: Color::from([0.5; 3]).into(),
    quad_border: Border {
      radius: Radius::new(4.0),
      ..Default::default()
    },
    inner_bounds: InnerBounds::Ratio(0.98, 0.2),
    height: Length::Fixed(20.0),
    ..Default::default()
  }
}

fn dot_separator<'a>(theme: &iced::Theme) -> Element<'a, Message, iced::Theme, iced::Renderer> {
  row((0..20).map(|_| {
    quad::Quad {
      quad_color: theme.extended_palette().background.base.text.into(),
      inner_bounds: InnerBounds::Square(4.0),
      ..separator()
    }
    .into()
  }))
  .height(20.0)
  .into()
}

fn labeled_separator(label: &'_ str) -> Element<'_, Message, iced::Theme, iced::Renderer> {
  let q_1 = quad::Quad {
    height: Length::Fill,
    ..separator()
  };
  let q_2 = quad::Quad {
    height: Length::Fill,
    ..separator()
  };

  row![
    q_1,
    text(label)
      .height(Length::Fill)
      .align_y(alignment::Vertical::Center),
    q_2,
  ]
  .height(20.0)
  .into()
}

fn circle(color: Color) -> quad::Quad {
  let radius = 10.0;

  quad::Quad {
    quad_color: color.into(),
    inner_bounds: InnerBounds::Square(radius * 2.0),
    quad_border: Border {
      radius: Radius::new(radius),
      ..Default::default()
    },
    height: Length::Fixed(20.0),
    ..Default::default()
  }
}
