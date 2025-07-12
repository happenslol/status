mod assets;
mod logging;
mod time;
mod util;

use assets::{Assets, load_embedded_fonts};
use gpui::Application;

fn main() {
  logging::init();

  Application::new().with_assets(Assets).run(|cx| {
    load_embedded_fonts(cx).unwrap();
    time::init(cx);
  });
}
