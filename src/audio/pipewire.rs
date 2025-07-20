use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;
use pipewire::{
  context::Context,
  main_loop::MainLoop,
  node::{Node, NodeListener},
  proxy::{ProxyListener, ProxyT},
  spa::{
    param::ParamType,
    pod::{Value, ValueArray, deserialize::PodDeserializer},
  },
  types::ObjectType,
};

const VOLUME_ID: u32 = 65544;
const MUTE_ID: u32 = 65540;

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
  Source,
  Sink,
}

impl NodeType {
  fn from_class(class: &str) -> Option<Self> {
    match class {
      "Audio/Source" => Some(Self::Source),
      "Audio/Sink" => Some(Self::Sink),
      _ => None,
    }
  }
}

#[derive(Debug)]
pub struct NodeUpdate {
  pub id: u32,
  pub type_: NodeType,
  pub name: String,
  pub volume: f32,
  pub mute: bool,
}

#[allow(dead_code)]
pub struct Listener {
  node: Node,
  node_listener: NodeListener,
  proxy_listener: ProxyListener,
}

pub fn run(tx: futures::channel::mpsc::UnboundedSender<NodeUpdate>) -> Result<()> {
  let mainloop = MainLoop::new(None)?;
  let context = Context::new(&mainloop)?;
  let core = context.connect(None)?;

  let registry = Rc::new(core.get_registry()?);
  let registry_weak = Rc::downgrade(&registry);

  let listeners: Rc<RefCell<HashMap<u32, Listener>>> = Rc::new(RefCell::new(HashMap::new()));

  let _listener = registry
    .add_listener_local()
    .global(move |global| {
      if global.type_ != ObjectType::Node {
        return;
      }

      let Some(props) = global.props.as_ref() else {
        return;
      };

      let Some(class) = props.get("media.class").and_then(NodeType::from_class) else {
        return;
      };

      let Some(name) = props
        .get("node.nick")
        .or_else(|| props.get("node.description"))
        .or_else(|| props.get("node.name"))
        .map(|v| v.to_string())
      else {
        return;
      };

      let name = Rc::new(name);

      let Some(registry) = registry_weak.upgrade() else {
        return;
      };

      let node_id = global.id;

      let node: Node = registry.bind(global).unwrap();
      node.subscribe_params(&[ParamType::Props]);

      let tx = tx.clone();
      let node_listener = node
        .add_listener_local()
        .param(move |_sq, _id, _index, _next, param| {
          let Some(param) = param else {
            return;
          };

          let Ok((_, v)) = PodDeserializer::deserialize_from::<Value>(param.as_bytes())
            .inspect_err(|e| println!("Error deserializing param: {e:?}"))
          else {
            return;
          };

          let Value::Object(o) = v else {
            return;
          };

          let mut volume: Option<f32> = None;
          let mut mute: Option<bool> = None;

          for p in o.properties.iter() {
            if p.key == VOLUME_ID {
              let Value::ValueArray(ValueArray::Float(ref channels)) = p.value else {
                continue;
              };

              if let Some(found) = channels
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
              {
                volume = Some(*found);
              }
            }

            if p.key == MUTE_ID
              && let Value::Bool(b) = p.value
            {
              mute = Some(b);
            }
          }

          if volume.is_none() && mute.is_none() {
            return;
          }

          if let Err(err) = tx.unbounded_send(NodeUpdate {
            id: node_id,
            volume: volume.unwrap_or(0.0),
            mute: mute.unwrap_or_default(),
            name: (*name).clone(),
            type_: class,
          }) {
            eprintln!("Error sending node update: {err:?}");
          }
        })
        .register();

      let proxy = node.upcast_ref();
      let proxy_id = proxy.id();

      let proxy_listener = proxy
        .add_listener_local()
        .removed({
          let listeners = Rc::downgrade(&listeners);
          move || {
            if let Some(listeners) = listeners.upgrade() {
              listeners.borrow_mut().remove(&proxy_id);
            }
          }
        })
        .register();

      listeners.borrow_mut().insert(
        proxy_id,
        Listener {
          node,
          node_listener,
          proxy_listener,
        },
      );
    })
    .register();

  mainloop.run();

  Ok(())
}
