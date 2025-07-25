mod assets;
mod config;
mod logging;
mod time;
mod util;

use assets::{Assets, load_embedded_fonts};
use gpui::Application;

fn main() {
  logging::init();

  Application::new().with_assets(Assets).run(|cx| {
    load_embedded_fonts(cx).expect("load fonts");
    config::init(cx).expect("init config");

    time::init(cx);
  });
}
