mod pipewire;

use std::{collections::HashMap, thread, time::Duration};

use anyhow::Result;
use futures::StreamExt;
use gpui::{
  Anchor, AnyWindowHandle, App, AppContext, Bounds, Context, Entity, Global, InteractiveElement,
  IntoElement, Layer, LayerShellSettings, Render, Size, StatefulInteractiveElement, Styled, Window,
  WindowOptions, div, point, px, rgb,
};
use tracing::error;

pub fn init(cx: &mut App) {
  let audio_store = cx.new(AudioStore::new);
  cx.set_global(GlobalAudioStore(audio_store));
}

#[allow(dead_code)]
struct GlobalAudioStore(Entity<AudioStore>);

impl Global for GlobalAudioStore {}

struct NodeState {
  type_: pipewire::NodeType,
  name: String,
  volume: f32,
  mute: bool,
}

struct AudioStore {
  #[allow(dead_code)]
  handle: thread::JoinHandle<Result<()>>,
  nodes: HashMap<u32, NodeState>,
}

impl AudioStore {
  pub fn new(cx: &mut Context<Self>) -> Self {
    let (tx, mut rx) = futures::channel::mpsc::unbounded();
    let handle = thread::spawn(move || pipewire::run(tx));
    let nodes = HashMap::new();

    cx.spawn(async move |this, cx| {
      while let Some(update) = rx.next().await {
        if let Err(err) = this.update(cx, |this, cx| this.handle_update(cx, update)) {
          eprintln!("Error handling update: {err:?}");
        }
      }

      eprintln!("Audio thread exited");
    })
    .detach();

    Self { handle, nodes }
  }

  fn handle_update(&mut self, cx: &mut Context<Self>, update: pipewire::NodeUpdate) {
    match self.nodes.get_mut(&update.id) {
      Some(node) => {
        if node.volume != update.volume {
          node.volume = update.volume;
        }

        if node.mute != update.mute {
          node.mute = update.mute;
        }

        open_window(cx);
      }
      None => {
        println!("New node: {update:?}");

        self.nodes.insert(
          update.id,
          NodeState {
            type_: update.type_,
            name: update.name,
            volume: update.volume,
            mute: update.mute,
          },
        );
      }
    }
  }
}

fn open_window(cx: &mut App) {
  let options = WindowOptions {
    titlebar: None,
    window_bounds: Some(gpui::WindowBounds::Windowed(Bounds {
      origin: point(px(0.), px(0.)),
      size: Size::new(px(400.), px(140.)),
    })),
    app_id: Some("status-volume".to_string()),
    window_background: gpui::WindowBackgroundAppearance::Transparent,
    kind: gpui::WindowKind::LayerShell(LayerShellSettings {
      layer: Layer::Top,
      anchor: Anchor::BOTTOM,
      exclusive_zone: None,
      margin: Some((px(0.), px(0.), px(0.), px(0.))),
      keyboard_interactivity: gpui::KeyboardInteractivity::None,
      pointer_interactivity: false,
      namespace: "status".to_string(),
    }),
    ..Default::default()
  };

  if let Err(err) = cx.open_window(options, VolumeIndicator::view) {
    error!(?err, "Failed to open window");
    cx.quit();
  }
}

struct VolumeIndicator {}

impl VolumeIndicator {
  pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
    cx.new(|cx| Self::new(window, cx))
  }

  fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
    window
      .spawn(cx, async move |cx| {
        cx.background_executor().timer(Duration::from_secs(1)).await;
        cx.update(|this, cx| {
          this.remove_window();
        })
        .unwrap();
      })
      .detach();

    Self {}
  }
}

impl Render for VolumeIndicator {
  fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
    div().id("audio").size_full().bg(rgb(0xffffff))
  }
}
