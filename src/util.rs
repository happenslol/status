#![allow(dead_code)]
use gpui::{App, DisplayId, Div, Styled, div};
use tracing::warn;

pub trait StyledExt: Styled + Sized {
  fn h_flex(self) -> Self {
    self.flex().flex_row().items_center()
  }

  fn v_flex(self) -> Self {
    self.flex().flex_col()
  }
}

impl StyledExt for Div {}

#[track_caller]
pub fn h_flex() -> Div {
  div().h_flex()
}

#[track_caller]
pub fn v_flex() -> Div {
  div().v_flex()
}

pub fn find_display(cx: &App, connector: &str) -> Option<DisplayId> {
  let display_id = cx
    .displays()
    .iter()
    .find(|d| d.name() == Some(connector))
    .map(|d| d.id());

  if display_id.is_none() {
    warn!(connector, "Failed to find display");
  }

  display_id
}
