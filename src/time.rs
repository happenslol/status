use std::{env, time::Duration};

use crate::util::{self, h_flex, v_flex};
use chrono::{DateTime, Local};
use gpui::{
  Anchor, App, Bounds, Context, DisplayId, Entity, FontWeight, Layer, LayerShellSettings,
  SharedString, Size, Window, WindowOptions, div, point, prelude::*, px, rems, rgb,
};
use tracing::{debug, error};

const OPACITY: f32 = 0.25;
const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

pub fn init(cx: &mut App) {
  match env::var("STATUS_DISPLAY").ok() {
    Some(connector) => {
      debug!(connector, "Opening on display");
      util::with_display(cx, connector, open_window)
    }
    None => open_window(cx, None),
  }
}

fn open_window(cx: &mut App, display_id: Option<DisplayId>) {
  let options = WindowOptions {
    titlebar: None,
    display_id,
    window_bounds: Some(gpui::WindowBounds::Windowed(Bounds {
      origin: point(px(0.), px(0.)),
      size: Size::new(px(400.), px(140.)),
    })),
    app_id: Some("status".to_string()),
    window_background: gpui::WindowBackgroundAppearance::Transparent,
    kind: gpui::WindowKind::LayerShell(LayerShellSettings {
      layer: Layer::Top,
      anchor: Anchor::BOTTOM | Anchor::RIGHT,
      exclusive_zone: None,
      margin: Some((px(0.), px(10.), px(5.), px(0.))),
      keyboard_interactivity: gpui::KeyboardInteractivity::None,
      pointer_interactivity: false,
      namespace: "status".to_string(),
    }),
    ..Default::default()
  };

  if let Err(err) = cx.open_window(options, Time::view) {
    error!(?err, "Failed to open window");
    cx.quit();
  }
}

struct Time {
  now: DateTime<Local>,
}

impl Time {
  pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
    cx.new(|cx| Self::new(window, cx))
  }

  fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
    cx.spawn(async move |this, cx| {
      loop {
        if let Err(err) = this.update(cx, Self::update_time) {
          error!(?err, "Failed to update");
        };

        cx.background_executor().timer(UPDATE_INTERVAL).await;
      }
    })
    .detach();

    Self { now: Local::now() }
  }

  fn update_time(&mut self, cx: &mut Context<Self>) {
    self.now = Local::now();
    cx.notify();
  }
}

impl Render for Time {
  fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
    let time = SharedString::new(format!("{}", self.now.format("%H:%M")));
    let date = SharedString::new(format!("{}", self.now.format("%a, %e %b"))).to_uppercase();

    h_flex()
      .justify_end()
      .items_end()
      .font_family("Noto Sans")
      .size_full()
      .text_color(rgb(0xffffff))
      .opacity(OPACITY)
      .child(
        div()
          .text_size(rems(1.4))
          .font_weight(FontWeight::SEMIBOLD)
          .child(date),
      )
      .child(
        div()
          .text_size(rems(3.5))
          .line_height(rems(3.5))
          .font_weight(FontWeight::BOLD)
          .child(time),
      )
  }
}
