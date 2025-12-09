/// Decorador @select para optimización de re-renders
/// Solo re-renderiza si el selector cambia
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::{Widget, BuildContext, VDomNode, Key};
use vela_state_management::Store;

/// Trait para widgets con selección optimizada
pub trait SelectableWidget: Widget {
    /// Selector que extrae la parte relevante del estado
    fn selector(&self) -> String;

    /// Hash del estado seleccionado para comparación
    fn selected_hash(&self) -> u64;
}

/// Macro para aplicar el decorador @select
#[macro_export]
macro_rules! select {
    ($widget:ident, $selector:expr) => {
        {
            struct SelectedImpl<W: Widget> {
                widget: W,
                selector: String,
                last_hash: std::cell::RefCell<Option<u64>>,
            }

            impl<W: Widget> SelectedImpl<W> {
                fn new(widget: W, selector: String) -> Self {
                    Self {
                        widget,
                        selector,
                        last_hash: std::cell::RefCell::new(None),
                    }
                }

                fn should_rerender(&self, new_hash: u64) -> bool {
                    let mut last_hash = self.last_hash.borrow_mut();
                    if let Some(old_hash) = *last_hash {
                        if old_hash != new_hash {
                            *last_hash = Some(new_hash);
                            true
                        } else {
                            false
                        }
                    } else {
                        *last_hash = Some(new_hash);
                        true
                    }
                }
            }

            impl<W: Widget> Widget for SelectedImpl<W> {
                fn build(&self, context: &BuildContext) -> VDomNode {
                    // Calcular hash del estado seleccionado
                    let state_hash = self.selected_hash();
                    if self.should_rerender(state_hash) {
                        self.widget.build(context)
                    } else {
                        // Devolver VDomNode vacío o cached
                        VDomNode::Empty
                    }
                }

                fn key(&self) -> Option<Key> {
                    self.widget.key()
                }
            }

            impl<W: Widget> SelectableWidget for SelectedImpl<W> {
                fn selector(&self) -> String {
                    self.selector.clone()
                }

                fn selected_hash(&self) -> u64 {
                    // Hash simple del selector (en implementación real sería del estado)
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    self.selector.hash(&mut hasher);
                    hasher.finish()
                }
            }

            SelectedImpl::new($widget, $selector.to_string())
        }
    };
}