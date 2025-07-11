mod assets;
mod util;

use assets::{Assets, load_embedded_fonts};
use gpui::{
  App, Application, Context, Entity, Size, Window, WindowOptions, actions, div, prelude::*, px,
};

struct Root;

actions!(root, [Quit]);

impl Root {
  pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
    cx.new(|cx| Self::new(window, cx))
  }

  fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
    Self
  }
}

impl Render for Root {
  fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div().font_family("Noto Sans").size_full()
  }
}

fn main() {
  tracing_subscriber::fmt::init();

  Application::new().with_assets(Assets).run(|cx| {
    load_embedded_fonts(cx).unwrap();

    cx.open_window(
      WindowOptions {
        titlebar: None,
        window_min_size: Some(Size::new(px(400.), px(400.))),
        app_id: Some("status".to_string()),
        ..Default::default()
      },
      Root::view,
    )
    .unwrap();

    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.activate(true);
  });
}
