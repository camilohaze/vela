/// Decorador @connect para conectar widgets al store global
/// Inspirado en React-Redux y Flutter Provider
use vela_state_management::Store;
use std::sync::Arc;
use crate::{Widget, BuildContext, VDomNode, Key};

/// Trait para widgets conectados al store
pub trait ConnectedWidget: Widget {
    /// Método para obtener el estado global
    fn store(&self) -> Arc<Store<Box<dyn std::any::Any + Send + Sync>>>;
}

/// Macro para aplicar el decorador @connect
#[macro_export]
macro_rules! connect {
    ($widget:ident, $store:expr) => {
        {
            struct ConnectedImpl {
                widget: $widget,
                store: std::sync::Arc<vela_state_management::Store<std::boxed::Box<dyn std::any::Any + Send + Sync>>>,
            }
            impl Widget for ConnectedImpl {
                fn build(&self, context: &BuildContext) -> VDomNode {
                    // El widget recibe el estado global como prop
                    self.widget.build(context)
                }
                fn key(&self) -> Option<Key> {
                    self.widget.key()
                }
            }
            ConnectedImpl {
                widget: $widget,
                store: $store,
            }
        }
    };
}
