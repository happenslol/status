use std::time::Duration;

use chrono::{DateTime, Local};
use futures::StreamExt;
use gpui::{
  App, Bounds, Context, DisplayId, Entity, FontWeight, SharedString, Size, Window, WindowOptions,
  div,
  layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
  point, prelude::*, px, rems, rgb,
};
use tracing::error;

use crate::{
  config::ConfigExt,
  util::{self, h_flex, v_flex},
};

const DEFAULT_OPACITY: f32 = 0.25;
const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

pub fn init(cx: &mut App) {
  let display_id = cx
    .config()
    .time
    .display
    .as_deref()
    .and_then(|connector| util::find_display(cx, connector));

  open_window(cx, display_id);
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
    kind: gpui::WindowKind::LayerShell(LayerShellOptions {
      layer: Layer::Top,
      anchor: Anchor::BOTTOM | Anchor::RIGHT,
      exclusive_zone: None,
      margin: Some((px(0.), px(10.), px(5.), px(0.))),
      keyboard_interactivity: KeyboardInteractivity::None,
      namespace: "status".to_string(),
      ..Default::default()
    }),
    ..Default::default()
  };

  if let Err(err) = cx.open_window(options, |window, cx| {
    window.set_input_passthrough();
    Time::view(window, cx)
  }) {
    error!(?err, "Failed to open window");
    cx.quit();
  }
}

#[derive(Clone, Copy)]
struct BatteryState {
  percentage: f64,
  on_battery: bool,
}

struct Time {
  now: DateTime<Local>,
  bat: Option<BatteryState>,
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

    cx.spawn(async move |this, cx| {
      let conn = zbus::Connection::system().await.unwrap();
      let Ok(proxy) = upower_dbus::UPowerProxy::new(&conn).await else {
        return;
      };

      let Ok(device) = proxy.get_display_device().await else {
        return;
      };

      let is_present = device.is_present().await.unwrap_or(false);
      if !is_present {
        return;
      }

      let Ok(on_battery) = proxy.on_battery().await else {
        return;
      };

      let Ok(percentage) = device.percentage().await else {
        return;
      };

      let _ = this.update(cx, |this, cx| {
        this.bat = Some(BatteryState {
          percentage,
          on_battery,
        });

        cx.notify();
      });

      enum BatteryEvent {
        OnBattery(bool),
        Percentage(f64),
      }

      let on_battery = proxy
        .receive_on_battery_changed()
        .await
        .then(async |ev| BatteryEvent::OnBattery(ev.get().await.unwrap()))
        .boxed();

      let percentage = device
        .receive_percentage_changed()
        .await
        .then(async |ev| BatteryEvent::Percentage(ev.get().await.unwrap()))
        .boxed();

      let mut events = futures::stream_select!(on_battery, percentage);

      while let Some(ev) = events.next().await {
        match ev {
          BatteryEvent::OnBattery(on_battery) => {
            let _ = this.update(cx, |this, cx| {
              if let Some(battery) = &mut this.bat {
                battery.on_battery = on_battery;
                cx.notify();
              }
            });
          }
          BatteryEvent::Percentage(percentage) => {
            let _ = this.update(cx, |this, cx| {
              if let Some(battery) = &mut this.bat {
                battery.percentage = percentage;
                cx.notify();
              }
            });
          }
        }
      }
    })
    .detach();

    Self {
      now: Local::now(),
      bat: None,
    }
  }

  fn update_time(&mut self, cx: &mut Context<Self>) {
    self.now = Local::now();
    cx.notify();
  }
}

impl Render for Time {
  fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let time = SharedString::new(format!("{}", self.now.format("%H:%M")));
    let date = SharedString::new(format!("{}", self.now.format("%a, %e %b"))).to_uppercase();

    v_flex()
      .justify_end()
      .items_end()
      .size_full()
      .font_family("Noto Sans")
      .text_color(rgb(0xffffff))
      .opacity(cx.config().time.opacity.unwrap_or(DEFAULT_OPACITY))
      .when_some(self.bat, |this, bat| {
        this.child(
          div()
            .text_size(rems(2.))
            .line_height(rems(1.6))
            .font_weight(FontWeight::SEMIBOLD)
            .child(SharedString::new(format!("{}", bat.percentage))),
        )
      })
      .child(
        h_flex()
          .items_end()
          .gap_2()
          .child(
            div()
              .text_size(rems(1.4))
              .line_height(rems(1.95))
              .font_weight(FontWeight::SEMIBOLD)
              .child(date),
          )
          .child(
            div()
              .text_size(rems(3.5))
              .line_height(rems(3.5))
              .font_weight(FontWeight::BOLD)
              .child(time),
          ),
      )
  }
}
