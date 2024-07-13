use std::any::TypeId;

use bevy::{
    ecs::{
        archetype::Archetype,
        bundle::DynamicBundle,
        component::{ComponentId, Components, StorageType, Tick},
        query::{FilteredAccess, QueryData, ReadOnlyQueryData, WorldQuery},
        storage::{Table, TableRow},
        world::{unsafe_world_cell::UnsafeWorldCell, DeferredWorld},
    },
    prelude::{Bundle, Component, Entity, World},
    ptr::OwningPtr,
};

pub trait EntityIndex: Clone + Send + Sync + 'static {
    fn on_insert(world: DeferredWorld<'_>, entity: Entity, value: Self);
    fn on_remove(world: DeferredWorld<'_>, entity: Entity, value: Self);
}

pub struct Index<T: EntityIndex>(pub T);

mod private {
    use crate::EntityIndex;

    #[doc(hidden)]
    /// Don't use this :)
    pub struct InternalIndex<T: EntityIndex>(pub T);
}

use private::InternalIndex;

unsafe impl<T: EntityIndex> QueryData for Index<T> {
    type ReadOnly = Self;
}

unsafe impl<T: EntityIndex> ReadOnlyQueryData for Index<T> {}

unsafe impl<T: EntityIndex> WorldQuery for Index<T> {
    type Item<'w> = T;
    type Fetch<'w> = <&'w InternalIndex<T> as WorldQuery>::Fetch<'w>;
    type State = ComponentId;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
        item
    }

    #[inline]
    unsafe fn init_fetch<'w>(
        world: UnsafeWorldCell<'w>,
        &component_id: &ComponentId,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Fetch<'w> {
        <&'w InternalIndex<T> as WorldQuery>::init_fetch(world, &component_id, _last_run, _this_run)
    }

    const IS_DENSE: bool = { <&InternalIndex<T> as WorldQuery>::IS_DENSE };

    #[inline]
    unsafe fn set_archetype<'w>(
        fetch: &mut Self::Fetch<'w>,
        component_id: &ComponentId,
        _archetype: &'w Archetype,
        table: &'w Table,
    ) {
        if Self::IS_DENSE {
            // SAFETY: `set_archetype`'s safety rules are a super set of the `set_table`'s ones.
            unsafe {
                Self::set_table(fetch, component_id, table);
            }
        }
    }

    #[inline]
    unsafe fn set_table<'w>(
        fetch: &mut Self::Fetch<'w>,
        &component_id: &ComponentId,
        table: &'w Table,
    ) {
        <&'w InternalIndex<T> as WorldQuery>::set_table(fetch, &component_id, table)
    }

    #[inline(always)]
    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Self::Item<'w> {
        let item = <&'w InternalIndex<T> as WorldQuery>::fetch(fetch, entity, table_row);
        item.0.clone()
    }

    fn update_component_access(
        &component_id: &ComponentId,
        access: &mut FilteredAccess<ComponentId>,
    ) {
        <&InternalIndex<T> as WorldQuery>::update_component_access(&component_id, access)
    }

    fn init_state(world: &mut World) -> ComponentId {
        world.init_component::<InternalIndex<T>>()
    }

    fn get_state(components: &Components) -> Option<Self::State> {
        components.component_id::<InternalIndex<T>>()
    }

    fn matches_component_set(
        &state: &ComponentId,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        set_contains_id(state)
    }
}

impl<T: EntityIndex> From<Index<T>> for InternalIndex<T> {
    fn from(value: Index<T>) -> Self {
        Self(value.0)
    }
}

impl<T: EntityIndex> Component for InternalIndex<T> {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    fn register_component_hooks(_hooks: &mut bevy::ecs::component::ComponentHooks) {
        _hooks.on_insert(|world, entity, _id| {
            let value = world
                .entity(entity)
                .get::<InternalIndex<T>>()
                .unwrap()
                .0
                .clone();
            T::on_insert(world, entity, value)
        });
        _hooks.on_remove(|world, entity, _id| {
            let value = world
                .entity(entity)
                .get::<InternalIndex<T>>()
                .unwrap()
                .0
                .clone();
            T::on_remove(world, entity, value)
        });
    }
}

impl<T: EntityIndex> DynamicBundle for Index<T> {
    fn get_components(
        self,
        func: &mut impl FnMut(bevy::ecs::component::StorageType, bevy::ptr::OwningPtr<'_>),
    ) {
        OwningPtr::make(InternalIndex::<T>::from(self), |ptr| {
            func(InternalIndex::<T>::STORAGE_TYPE, ptr)
        })
    }
}

/// # Safety:
/// Probably isn't
unsafe impl<T: EntityIndex> Bundle for Index<T> {
    fn get_component_ids(
        components: &bevy::ecs::component::Components,
        ids: &mut impl FnMut(Option<bevy::ecs::component::ComponentId>),
    ) {
        ids(components.get_id(TypeId::of::<InternalIndex<T>>()))
    }

    fn component_ids(
        components: &mut bevy::ecs::component::Components,
        storages: &mut bevy::ecs::storage::Storages,
        ids: &mut impl FnMut(bevy::ecs::component::ComponentId),
    ) {
        ids(components.init_component::<InternalIndex<T>>(storages));
    }

    unsafe fn from_components<C, F>(ctx: &mut C, func: &mut F) -> Self
    where
        // Ensure that the `OwningPtr` is used correctly
        F: for<'a> FnMut(&'a mut C) -> OwningPtr<'a>,
        Self: Sized,
    {
        let ptr = func(ctx);
        // # Safety:
        // Copied from bevy source :)
        unsafe { ptr.read() }
    }
}
