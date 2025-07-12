#![allow(dead_code)]
use std::time::Duration;

use gpui::{App, DisplayId, Div, Styled, div};
use tracing::{error, warn};

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

pub fn with_display(
  cx: &mut App,
  connector: String,
  f: impl FnOnce(&mut App, Option<DisplayId>) + 'static,
) {
  cx.spawn(async move |cx| {
    let mut tries = 0;

    loop {
      match cx.update(|cx| find_display(cx, &connector)) {
        Ok(Some(display_id)) => {
          cx.update(|cx| f(cx, Some(display_id))).unwrap();
          break;
        }
        Ok(None) => {
          tries += 1;
          if tries > 5 {
            warn!("Failed to find display after 5 tries");
            cx.update(|cx| f(cx, None)).unwrap();
            break;
          }
        }
        Err(err) => {
          error!(?err, "Failed to update");
          return;
        }
      }

      cx.background_executor()
        .timer(Duration::from_millis(10))
        .await;
    }
  })
  .detach();
}

fn find_display(cx: &mut App, connector: &str) -> Option<DisplayId> {
  cx.displays()
    .iter()
    .find(|d| d.name().as_deref() == Some(connector))
    .map(|d| d.id())
}
