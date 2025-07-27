mod assets;
mod config;
mod logging;
mod time;
mod util;

use assets::{Assets, load_embedded_fonts};
use gpui::Application;

fn main() {
  logging::init();
  let config = config::Config::load_sync();

  Application::new().with_assets(Assets).run(move |cx| {
    load_embedded_fonts(cx).unwrap();
    cx.set_global(config);
    time::init(cx);
  });
}
