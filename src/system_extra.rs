//! Extra system utilities.
//!
//! This modules contains an extension trait for the System trait which adds useful transformation
//! functions.

use ecs::prelude::{Read, System};
use shred::{RunningTime, SystemData};

/// Extra functionality associated systems.
pub trait SystemExtra {
    /// Make a system pausable by tying it to a specific value of a resource.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amethyst::{
    ///     ecs::{System, Write},
    ///     shred::DispatcherBuilder,
    ///     prelude::*,
    /// };
    ///
    /// #[derive(PartialEq, Eq)]
    /// enum State {
    ///     Disabled,
    ///     Enabled,
    /// }
    ///
    /// impl Default for State {
    ///     fn default() -> Self {
    ///         State::Disabled
    ///     }
    /// }
    ///
    /// struct AddNumber(u32);
    ///
    /// impl<'s> System<'s> for AddNumber {
    ///     type SystemData = Write<'s, u32>;
    ///
    ///     fn run(&mut self, mut number: Self::SystemData) {
    ///         *number += self.0;
    ///     }
    /// }
    ///
    /// let mut world = World::new();
    ///
    /// let mut dispatcher = DispatcherBuilder::default()
    ///     .with(AddNumber(1), "set_number", &[])
    ///     .with(AddNumber(2).pausable(State::Enabled), "set_number_2", &[])
    ///     .build();
    ///
    /// dispatcher.setup(&mut world.res);
    ///
    /// // we only expect the u32 resource to be modified _if_ the system is enabled,
    /// // the system should only be enabled on State::Enabled.
    ///
    /// *world.write_resource() = 0u32;
    /// dispatcher.dispatch(&mut world.res);
    /// assert_eq!(1, *world.read_resource::<u32>());
    ///
    /// *world.write_resource() = 0u32;
    /// *world.write_resource() = State::Enabled;
    /// dispatcher.dispatch(&mut world.res);
    /// assert_eq!(1 + 2, *world.read_resource::<u32>());
    /// ```
    fn pausable<V: 'static>(self, value: V) -> Pausable<Self, V>
    where
        Self: Sized,
        V: Send + Sync + Default + Eq;
}

impl<'s, S> SystemExtra for S
where
    S: System<'s>,
{
    fn pausable<V: 'static>(self, value: V) -> Pausable<Self, V>
    where
        Self: Sized,
        V: Send + Sync + Default + Eq,
    {
        Pausable {
            system: self,
            value,
        }
    }
}

/// A system that is enabled when `U` has a specific value.
pub struct Pausable<S, V> {
    system: S,
    value: V,
}

impl<'s, S, V: 'static> System<'s> for Pausable<S, V>
where
    S::SystemData: SystemData<'s>,
    S: System<'s>,
    V: Send + Sync + Default + Eq,
{
    type SystemData = (Read<'s, V>, S::SystemData);

    fn run(&mut self, data: Self::SystemData) {
        if self.value != *data.0 {
            return;
        }

        self.system.run(data.1);
    }

    fn running_time(&self) -> RunningTime {
        self.system.running_time()
    }
}
