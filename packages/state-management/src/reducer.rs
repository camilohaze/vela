/*
Implementación de Reducer functions para el sistema de Store Redux-style

Historia: VELA-035R (EPIC-03D State Management)
Tarea: TASK-035T
Fecha: 2025-12-09

Descripción:
Funciones reductoras puras que transforman el estado basado en acciones.
Implementa pattern matching y composición funcional para reducers Redux-style.
*/

use std::sync::Arc;

/// Type alias para funciones reductoras puras
/// Una función reductora toma el estado actual y una acción, y retorna el nuevo estado
pub type Reducer<State, A> = Arc<dyn Fn(&State, &A) -> State + Send + Sync>;

/// Builder para crear reducers compuestos
pub struct ReducerBuilder<State> {
    reducers: Vec<Box<dyn Fn(&State, &dyn std::any::Any) -> Option<State> + Send + Sync>>,
}

impl<State> ReducerBuilder<State>
where
    State: Clone + Send + Sync + 'static,
{
    /// Crear nuevo builder
    pub fn new() -> Self {
        ReducerBuilder {
            reducers: Vec::new(),
        }
    }

    /// Agregar un reducer para un tipo específico de acción
    pub fn add_reducer<A>(mut self, reducer_fn: impl Fn(&State, &A) -> State + Send + Sync + 'static) -> Self
    where
        A: 'static,
    {
        let boxed_reducer = Box::new(move |state: &State, action: &dyn std::any::Any| {
            action.downcast_ref::<A>().map(|a| reducer_fn(state, a))
        });
        self.reducers.push(boxed_reducer);
        self
    }

    /// Construir el reducer compuesto
    pub fn build(self) -> Reducer<State, Box<dyn std::any::Any + Send + Sync>> {
        let reducers = Arc::new(self.reducers);

        Arc::new(move |state: &State, action: &Box<dyn std::any::Any + Send + Sync>| {
            for reducer in reducers.iter() {
                if let Some(new_state) = reducer(state, action.as_ref()) {
                    return new_state;
                }
            }
            // Si ningún reducer maneja la acción, retornar estado sin cambios
            state.clone()
        })
    }
}

/// Función helper para crear reducers simples
pub fn create_reducer<State, A>(
    reducer_fn: impl Fn(&State, &A) -> State + Send + Sync + 'static,
) -> Reducer<State, A>
where
    State: 'static,
    A: 'static,
{
    Arc::new(reducer_fn)
}

/// Función helper para combinar múltiples reducers
pub fn combine_reducers<State>(
    reducers: Vec<Reducer<State, Box<dyn std::any::Any + Send + Sync>>>,
) -> Reducer<State, Box<dyn std::any::Any + Send + Sync>>
where
    State: Clone + Send + Sync + 'static,
{
    Arc::new(move |state: &State, action: &Box<dyn std::any::Any + Send + Sync>| {
        let mut current_state = state.clone();
        for reducer in &reducers {
            current_state = reducer(&current_state, action);
        }
        current_state
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    #[derive(Debug, Clone, PartialEq)]
    struct CounterState {
        count: i32,
    }

    #[derive(Debug)]
    struct Increment;

    #[derive(Debug)]
    struct Decrement;

    #[derive(Debug)]
    struct SetValue {
        value: i32,
    }

    impl CounterState {
        fn new() -> Self {
            CounterState { count: 0 }
        }
    }

    #[test]
    fn test_simple_reducer() {
        let initial_state = CounterState::new();
        let reducer = create_reducer(|state: &CounterState, _action: &Increment| {
            CounterState {
                count: state.count + 1,
            }
        });

        let action = Increment;
        let new_state = reducer(&initial_state, &action);

        assert_eq!(new_state.count, 1);
    }

    #[test]
    fn test_reducer_builder() {
        let builder = ReducerBuilder::new()
            .add_reducer(|state: &CounterState, _action: &Increment| {
                CounterState {
                    count: state.count + 1,
                }
            })
            .add_reducer(|state: &CounterState, _action: &Decrement| {
                CounterState {
                    count: state.count - 1,
                }
            })
            .add_reducer(|state: &CounterState, action: &SetValue| {
                CounterState {
                    count: action.value,
                }
            });

        let reducer = builder.build();
        let initial_state = CounterState::new();

        // Test increment
        let action: Box<dyn Any + Send + Sync> = Box::new(Increment);
        let new_state = reducer(&initial_state, &action);
        assert_eq!(new_state.count, 1);

        // Test decrement
        let action: Box<dyn Any + Send + Sync> = Box::new(Decrement);
        let new_state = reducer(&new_state, &action);
        assert_eq!(new_state.count, 0);

        // Test set value
        let action: Box<dyn Any + Send + Sync> = Box::new(SetValue { value: 42 });
        let new_state = reducer(&new_state, &action);
        assert_eq!(new_state.count, 42);
    }

    #[test]
    fn test_combine_reducers() {
        // Crear reducers que manejan el mismo tipo de acción
        let increment_reducer: Reducer<CounterState, Box<dyn Any + Send + Sync>> = Arc::new(|state: &CounterState, action: &Box<dyn Any + Send + Sync>| {
            if let Some(_inc) = action.downcast_ref::<Increment>() {
                CounterState {
                    count: state.count + 1,
                }
            } else {
                state.clone()
            }
        });

        let multiply_reducer: Reducer<CounterState, Box<dyn Any + Send + Sync>> = Arc::new(|state: &CounterState, action: &Box<dyn Any + Send + Sync>| {
            if let Some(_inc) = action.downcast_ref::<Increment>() {
                CounterState {
                    count: state.count * 2,
                }
            } else {
                state.clone()
            }
        });

        let combined = combine_reducers(vec![increment_reducer, multiply_reducer]);
        let initial_state = CounterState { count: 5 };

        let action: Box<dyn Any + Send + Sync> = Box::new(Increment);
        let new_state = combined(&initial_state, &action);

        // Primero incrementa (5 + 1 = 6), luego multiplica (6 * 2 = 12)
        assert_eq!(new_state.count, 12);
    }

    #[test]
    fn test_reducer_immutability() {
        let initial_state = CounterState::new();
        let reducer = create_reducer(|state: &CounterState, _action: &Increment| {
            CounterState {
                count: state.count + 1,
            }
        });

        let action = Increment;
        let new_state = reducer(&initial_state, &action);

        // Estado original no debe cambiar
        assert_eq!(initial_state.count, 0);
        assert_eq!(new_state.count, 1);
    }
}