use std::sync::Arc;

use arrow::{
    array::{
        Array as ArrowArray, ArrayRef as ArrowArrayRef, ArrowPrimitiveType,
        BooleanArray as ArrowBooleanArray, FixedSizeListArray as ArrowFixedSizeListArray,
        ListArray as ArrowListArray, PrimitiveArray as ArrowPrimitiveArray,
        StringArray as ArrowStringArray, StructArray as ArrowStructArray,
    },
    buffer::{BooleanBuffer as ArrowBooleanBuffer, ScalarBuffer as ArrowScalarBuffer},
    datatypes::ArrowNativeType,
};
use itertools::{Either, Itertools as _, izip};

use re_arrow_util::{ArrowArrayDowncastRef as _, offsets_lengths};
use re_log_types::{TimeInt, TimePoint, TimelineName};
use re_span::Span;
use re_types_core::{ArrowString, Component, ComponentDescriptor};

use crate::{Chunk, RowId, TimeColumn};

// ---

// NOTE: Regarding the use of (recursive) `Either` in this file: it is _not_ arbitrary.
//
// They _should_ all follow this model:
// * The first layer is always the emptiness layer: `Left` is empty, `Right` is non-empty.
// * The second layer is the temporarily layer: `Left` is static, `Right` is temporal.
// * Any layers beyond that follow the same pattern: `Left` doesn't have something, while `Right` does.

impl Chunk {
    /// Return the raw component list array values for a given component.
    ///
    /// Use with great care: Component data may have arbitrary gaps.
    pub fn raw_component_array(
        &self,
        component_descr: &ComponentDescriptor,
    ) -> Option<&ArrowArrayRef> {
        self.components
            .get(component_descr)
            .map(|list_array| list_array.values())
    }

    /// Returns an iterator over the indices (`(TimeInt, RowId)`) of a [`Chunk`], for a given timeline.
    ///
    /// If the chunk is static, `timeline` will be ignored.
    ///
    /// See also:
    /// * [`Self::iter_component_indices`].
    /// * [`Self::iter_indices_owned`].
    #[inline]
    pub fn iter_indices(
        &self,
        timeline: &TimelineName,
    ) -> impl Iterator<Item = (TimeInt, RowId)> + '_ + use<'_> {
        if self.is_static() {
            Either::Right(Either::Left(izip!(
                std::iter::repeat(TimeInt::STATIC),
                self.row_ids()
            )))
        } else {
            let Some(time_column) = self.timelines.get(timeline) else {
                return Either::Left(std::iter::empty());
            };

            Either::Right(Either::Right(izip!(time_column.times(), self.row_ids())))
        }
    }

    /// Returns an iterator over the indices (`(TimeInt, RowId)`) of a [`Chunk`], for a given
    /// timeline and component.
    ///
    /// If the chunk is static, `timeline` will be ignored.
    ///
    /// This is different than [`Self::iter_indices`] in that it will only yield indices for rows
    /// at which there is data for the specified component.
    ///
    /// See also [`Self::iter_indices`].
    pub fn iter_component_indices(
        &self,
        timeline: &TimelineName,
        component_descr: &ComponentDescriptor,
    ) -> impl Iterator<Item = (TimeInt, RowId)> + '_ + use<'_> {
        let Some(list_array) = self.components.get(component_descr) else {
            return Either::Left(std::iter::empty());
        };

        if self.is_static() {
            let indices = izip!(std::iter::repeat(TimeInt::STATIC), self.row_ids());

            if let Some(validity) = list_array.nulls() {
                Either::Right(Either::Left(Either::Left(
                    indices
                        .enumerate()
                        .filter_map(|(i, o)| validity.is_valid(i).then_some(o)),
                )))
            } else {
                Either::Right(Either::Left(Either::Right(indices)))
            }
        } else {
            let Some(time_column) = self.timelines.get(timeline) else {
                return Either::Left(std::iter::empty());
            };

            let indices = izip!(time_column.times(), self.row_ids());

            if let Some(validity) = list_array.nulls() {
                Either::Right(Either::Right(Either::Left(
                    indices
                        .enumerate()
                        .filter_map(|(i, o)| validity.is_valid(i).then_some(o)),
                )))
            } else {
                Either::Right(Either::Right(Either::Right(indices)))
            }
        }
    }

    /// Returns an iterator over the [`TimePoint`]s of a [`Chunk`].
    ///
    /// See also:
    /// * [`Self::iter_component_timepoints`].
    #[inline]
    pub fn iter_timepoints(&self) -> impl Iterator<Item = TimePoint> + '_ {
        let mut timelines = self
            .timelines
            .values()
            .map(|time_column| (time_column.timeline, time_column.times()))
            .collect_vec();

        std::iter::from_fn(move || {
            let mut timepoint = TimePoint::default();
            for (timeline, times) in &mut timelines {
                timepoint.insert(*timeline, times.next()?);
            }
            Some(timepoint)
        })
    }

    /// Returns an iterator over the [`TimePoint`]s of a [`Chunk`], for a given component.
    ///
    /// This is different than [`Self::iter_timepoints`] in that it will only yield timepoints for rows
    /// at which there is data for the specified component.
    ///
    /// See also [`Self::iter_timepoints`].
    pub fn iter_component_timepoints(
        &self,
        component_descr: &ComponentDescriptor,
    ) -> impl Iterator<Item = TimePoint> + '_ + use<'_> {
        let Some(list_array) = self.components.get(component_descr) else {
            return Either::Left(std::iter::empty());
        };

        if let Some(validity) = list_array.nulls() {
            let mut timelines = self
                .timelines
                .values()
                .map(|time_column| {
                    (
                        time_column.timeline,
                        time_column
                            .times()
                            .enumerate()
                            .filter(|(i, _)| validity.is_valid(*i))
                            .map(|(_, time)| time),
                    )
                })
                .collect_vec();

            Either::Right(Either::Left(std::iter::from_fn(move || {
                let mut timepoint = TimePoint::default();
                for (timeline, times) in &mut timelines {
                    timepoint.insert(*timeline, times.next()?);
                }
                Some(timepoint)
            })))
        } else {
            let mut timelines = self
                .timelines
                .values()
                .map(|time_column| (time_column.timeline, time_column.times()))
                .collect_vec();

            Either::Right(Either::Right(std::iter::from_fn(move || {
                let mut timepoint = TimePoint::default();
                for (timeline, times) in &mut timelines {
                    timepoint.insert(*timeline, times.next()?);
                }
                Some(timepoint)
            })))
        }
    }

    /// Returns an iterator over the offsets (`(offset, len)`) of a [`Chunk`], for a given
    /// component.
    ///
    /// I.e. each `(offset, len)` pair describes the position of a component batch in the
    /// underlying arrow array of values.
    pub fn iter_component_offsets<'a>(
        &'a self,
        component_descriptor: &ComponentDescriptor,
    ) -> impl Iterator<Item = Span<usize>> + 'a + use<'a> {
        let Some(list_array) = self.components.get(component_descriptor) else {
            return Either::Left(std::iter::empty());
        };

        let offsets = list_array.offsets().iter().map(|idx| *idx as usize);
        let lengths = offsets_lengths(list_array.offsets());

        if let Some(validity) = list_array.nulls() {
            Either::Right(Either::Left(
                izip!(offsets, lengths)
                    .enumerate()
                    .filter_map(|(i, o)| validity.is_valid(i).then_some(o))
                    .map(|(start, len)| Span { start, len }),
            ))
        } else {
            Either::Right(Either::Right(
                izip!(offsets, lengths).map(|(start, len)| Span { start, len }),
            ))
        }
    }

    /// Returns an iterator over the all the sliced component batches in a [`Chunk`]'s column, for
    /// a given component.
    ///
    /// The generic `S` parameter will decide the type of data returned. It is _very_ permissive.
    /// See [`ChunkComponentSlicer`] for all the available implementations.
    ///
    /// This is a very fast path: the entire column will be downcasted at once, and then every
    /// component batch will be a slice reference into that global slice.
    ///
    /// See also [`Self::iter_slices_from_struct_field`].
    #[inline]
    pub fn iter_slices<'a, S: 'a + ChunkComponentSlicer>(
        &'a self,
        component_descriptor: ComponentDescriptor,
    ) -> impl Iterator<Item = S::Item<'a>> + 'a + use<'a, S> {
        let Some(list_array) = self.components.get(&component_descriptor) else {
            return Either::Left(std::iter::empty());
        };

        let component_offset_values = self.iter_component_offsets(&component_descriptor);

        Either::Right(S::slice(
            component_descriptor,
            &**list_array.values() as _,
            component_offset_values,
        ))
    }

    /// Returns an iterator over the all the sliced component batches in a [`Chunk`]'s column, for
    /// a specific struct field of given component.
    ///
    /// The target component must be a `StructArray`.
    ///
    /// The generic `S` parameter will decide the type of data returned. It is _very_ permissive.
    /// See [`ChunkComponentSlicer`] for all the available implementations.
    ///
    /// This is a very fast path: the entire column will be downcasted at once, and then every
    /// component batch will be a slice reference into that global slice.
    ///
    /// See also [`Self::iter_slices_from_struct_field`].
    pub fn iter_slices_from_struct_field<'a, S: 'a + ChunkComponentSlicer>(
        &'a self,
        component_descriptor: ComponentDescriptor,
        field_name: &'a str,
    ) -> impl Iterator<Item = S::Item<'a>> + 'a {
        let Some(list_array) = self.components.get(&component_descriptor) else {
            return Either::Left(std::iter::empty());
        };

        let Some(struct_array) = list_array.values().downcast_array_ref::<ArrowStructArray>()
        else {
            if cfg!(debug_assertions) {
                panic!("downcast failed for {component_descriptor}, data discarded");
            } else {
                re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
            }
            return Either::Left(std::iter::empty());
        };

        let Some(field_idx) = struct_array
            .fields()
            .iter()
            .enumerate()
            .find_map(|(i, field)| (field.name() == field_name).then_some(i))
        else {
            if cfg!(debug_assertions) {
                panic!("field {field_name} not found for {component_descriptor}, data discarded");
            } else {
                re_log::error_once!(
                    "field {field_name} not found for {component_descriptor}, data discarded"
                );
            }
            return Either::Left(std::iter::empty());
        };

        if field_idx >= struct_array.num_columns() {
            if cfg!(debug_assertions) {
                panic!("field {field_name} not found for {component_descriptor}, data discarded");
            } else {
                re_log::error_once!(
                    "field {field_name} not found for {component_descriptor}, data discarded"
                );
                return Either::Left(std::iter::empty());
            }
        }

        let component_offset_values = self.iter_component_offsets(&component_descriptor);

        Either::Right(S::slice(
            component_descriptor,
            struct_array.column(field_idx),
            component_offset_values,
        ))
    }
}

// ---

/// A `ChunkComponentSlicer` knows how to efficiently slice component batches out of a Chunk column.
///
/// See [`Chunk::iter_slices`] and [`Chunk::iter_slices_from_struct_field`].
pub trait ChunkComponentSlicer {
    type Item<'a>;

    fn slice<'a>(
        // TODO(#10460): A reference to component descriptor should be enough since the returned iterator doesn't depend on it being alive.
        // However, I wasn't able to get this idea across to the borrow checker.
        component_descriptor: ComponentDescriptor,
        array: &'a dyn ArrowArray,
        component_spans: impl Iterator<Item = Span<usize>> + 'a,
    ) -> impl Iterator<Item = Self::Item<'a>> + 'a;
}

/// The actual implementation of `impl_native_type!`, so that we don't have to work in a macro.
#[expect(clippy::needless_pass_by_value)] // The simplest way to avoid lifetime issues.
fn slice_as_native<'a, P, T>(
    component_descriptor: ComponentDescriptor,
    array: &'a dyn ArrowArray,
    component_spans: impl Iterator<Item = Span<usize>> + 'a,
) -> impl Iterator<Item = &'a [T]> + 'a
where
    P: ArrowPrimitiveType<Native = T>,
    T: ArrowNativeType,
{
    let Some(values) = array.downcast_array_ref::<ArrowPrimitiveArray<P>>() else {
        if cfg!(debug_assertions) {
            panic!("downcast failed for {component_descriptor}, data discarded");
        } else {
            re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
        }
        return Either::Left(std::iter::empty());
    };
    let values = values.values().as_ref();

    // NOTE: No need for validity checks here, `iter_offsets` already takes care of that.
    Either::Right(component_spans.map(move |range| &values[range.range()]))
}

// We use a macro instead of a blanket impl because this violates orphan rules.
macro_rules! impl_native_type {
    ($arrow_primitive_type:ty, $native_type:ty) => {
        impl ChunkComponentSlicer for $native_type {
            type Item<'a> = &'a [$native_type];

            fn slice<'a>(
                component_descriptor: ComponentDescriptor,
                array: &'a dyn ArrowArray,
                component_spans: impl Iterator<Item = Span<usize>> + 'a,
            ) -> impl Iterator<Item = Self::Item<'a>> + 'a {
                slice_as_native::<$arrow_primitive_type, $native_type>(
                    component_descriptor,
                    array,
                    component_spans,
                )
            }
        }
    };
}

impl_native_type!(arrow::array::types::UInt8Type, u8);
impl_native_type!(arrow::array::types::UInt16Type, u16);
impl_native_type!(arrow::array::types::UInt32Type, u32);
impl_native_type!(arrow::array::types::UInt64Type, u64);
// impl_native_type!(arrow::array::types::UInt128Type, u128);
impl_native_type!(arrow::array::types::Int8Type, i8);
impl_native_type!(arrow::array::types::Int16Type, i16);
impl_native_type!(arrow::array::types::Int32Type, i32);
impl_native_type!(arrow::array::types::Int64Type, i64);
// impl_native_type!(arrow::array::types::Int128Type, i128);
impl_native_type!(arrow::array::types::Float16Type, half::f16);
impl_native_type!(arrow::array::types::Float32Type, f32);
impl_native_type!(arrow::array::types::Float64Type, f64);

/// The actual implementation of `impl_array_native_type!`, so that we don't have to work in a macro.
#[expect(clippy::needless_pass_by_value)] // The simplest way to avoid lifetime issues.
fn slice_as_array_native<'a, const N: usize, P, T>(
    component_descriptor: ComponentDescriptor,
    array: &'a dyn ArrowArray,
    component_spans: impl Iterator<Item = Span<usize>> + 'a,
) -> impl Iterator<Item = &'a [[T; N]]> + 'a
where
    [T; N]: bytemuck::Pod,
    P: ArrowPrimitiveType<Native = T>,
    T: ArrowNativeType + bytemuck::Pod,
{
    let Some(fixed_size_list_array) = array.downcast_array_ref::<ArrowFixedSizeListArray>() else {
        if cfg!(debug_assertions) {
            panic!("downcast failed for {component_descriptor}, data discarded");
        } else {
            re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
        }
        return Either::Left(std::iter::empty());
    };

    let Some(values) = fixed_size_list_array
        .values()
        .downcast_array_ref::<ArrowPrimitiveArray<P>>()
    else {
        if cfg!(debug_assertions) {
            panic!("downcast failed for {component_descriptor}, data discarded");
        } else {
            re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
        }
        return Either::Left(std::iter::empty());
    };

    let size = fixed_size_list_array.value_length() as usize;
    let values = values.values().as_ref();

    // NOTE: No need for validity checks here, `component_spans` already takes care of that.
    Either::Right(
        component_spans.map(move |span| bytemuck::cast_slice(&values[(span * size).range()])),
    )
}

// We use a macro instead of a blanket impl because this violates orphan rules.
macro_rules! impl_array_native_type {
    ($arrow_primitive_type:ty, $native_type:ty) => {
        impl<const N: usize> ChunkComponentSlicer for [$native_type; N]
        where
            [$native_type; N]: bytemuck::Pod,
        {
            type Item<'a> = &'a [[$native_type; N]];

            fn slice<'a>(
                component_descriptor: ComponentDescriptor,
                array: &'a dyn ArrowArray,
                component_spans: impl Iterator<Item = Span<usize>> + 'a,
            ) -> impl Iterator<Item = Self::Item<'a>> + 'a {
                slice_as_array_native::<N, $arrow_primitive_type, $native_type>(
                    component_descriptor,
                    array,
                    component_spans,
                )
            }
        }
    };
}

impl_array_native_type!(arrow::array::types::UInt8Type, u8);
impl_array_native_type!(arrow::array::types::UInt16Type, u16);
impl_array_native_type!(arrow::array::types::UInt32Type, u32);
impl_array_native_type!(arrow::array::types::UInt64Type, u64);
// impl_array_native_type!(arrow::array::types::UInt128Type, u128);
impl_array_native_type!(arrow::array::types::Int8Type, i8);
impl_array_native_type!(arrow::array::types::Int16Type, i16);
impl_array_native_type!(arrow::array::types::Int32Type, i32);
impl_array_native_type!(arrow::array::types::Int64Type, i64);
// impl_array_native_type!(arrow::array::types::Int128Type, i128);
impl_array_native_type!(arrow::array::types::Float16Type, half::f16);
impl_array_native_type!(arrow::array::types::Float32Type, f32);
impl_array_native_type!(arrow::array::types::Float64Type, f64);

/// The actual implementation of `impl_buffer_native_type!`, so that we don't have to work in a macro.
#[expect(clippy::needless_pass_by_value)] // The simplest way to avoid lifetime issues.
fn slice_as_buffer_native<'a, P, T>(
    component_descriptor: ComponentDescriptor,
    array: &'a dyn ArrowArray,
    component_spans: impl Iterator<Item = Span<usize>> + 'a,
) -> impl Iterator<Item = Vec<ArrowScalarBuffer<T>>> + 'a
where
    P: ArrowPrimitiveType<Native = T>,
    T: ArrowNativeType,
{
    let Some(inner_list_array) = array.downcast_array_ref::<ArrowListArray>() else {
        if cfg!(debug_assertions) {
            panic!("downcast failed for {component_descriptor}, data discarded");
        } else {
            re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
        }
        return Either::Left(std::iter::empty());
    };

    let Some(values) = inner_list_array
        .values()
        .downcast_array_ref::<ArrowPrimitiveArray<P>>()
    else {
        if cfg!(debug_assertions) {
            panic!("downcast failed for {component_descriptor}, data discarded");
        } else {
            re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
        }
        return Either::Left(std::iter::empty());
    };

    let values = values.values();
    let offsets = inner_list_array.offsets();
    let lengths = offsets_lengths(inner_list_array.offsets()).collect_vec();

    // NOTE: No need for validity checks here, `component_spans` already takes care of that.
    Either::Right(component_spans.map(move |span| {
        let offsets = &offsets[span.range()];
        let lengths = &lengths[span.range()];
        izip!(offsets, lengths)
            // NOTE: Not an actual clone, just a refbump of the underlying buffer.
            .map(|(&idx, &len)| values.clone().slice(idx as _, len))
            .collect_vec()
    }))
}

// We use a macro instead of a blanket impl because this violates orphan rules.
macro_rules! impl_buffer_native_type {
    ($primitive_type:ty, $native_type:ty) => {
        impl ChunkComponentSlicer for &[$native_type] {
            type Item<'a> = Vec<ArrowScalarBuffer<$native_type>>;

            fn slice<'a>(
                component_descriptor: ComponentDescriptor,
                array: &'a dyn ArrowArray,
                component_spans: impl Iterator<Item = Span<usize>> + 'a,
            ) -> impl Iterator<Item = Self::Item<'a>> + 'a {
                slice_as_buffer_native::<$primitive_type, $native_type>(
                    component_descriptor,
                    array,
                    component_spans,
                )
            }
        }
    };
}

impl_buffer_native_type!(arrow::array::types::UInt8Type, u8);
impl_buffer_native_type!(arrow::array::types::UInt16Type, u16);
impl_buffer_native_type!(arrow::array::types::UInt32Type, u32);
impl_buffer_native_type!(arrow::array::types::UInt64Type, u64);
// impl_buffer_native_type!(arrow::array::types::UInt128Type, u128);
impl_buffer_native_type!(arrow::array::types::Int8Type, i8);
impl_buffer_native_type!(arrow::array::types::Int16Type, i16);
impl_buffer_native_type!(arrow::array::types::Int32Type, i32);
impl_buffer_native_type!(arrow::array::types::Int64Type, i64);
// impl_buffer_native_type!(arrow::array::types::Int128Type, i128);
impl_buffer_native_type!(arrow::array::types::Float16Type, half::f16);
impl_buffer_native_type!(arrow::array::types::Float32Type, f32);
impl_buffer_native_type!(arrow::array::types::Float64Type, f64);

/// The actual implementation of `impl_array_list_native_type!`, so that we don't have to work in a macro.
#[expect(clippy::needless_pass_by_value)] // The simplest way to avoid lifetime issues.
fn slice_as_array_list_native<'a, const N: usize, P, T>(
    component_descriptor: ComponentDescriptor,
    array: &'a dyn ArrowArray,
    component_spans: impl Iterator<Item = Span<usize>> + 'a,
) -> impl Iterator<Item = Vec<&'a [[T; N]]>> + 'a
where
    [T; N]: bytemuck::Pod,
    P: ArrowPrimitiveType<Native = T>,
    T: ArrowNativeType + bytemuck::Pod,
{
    let Some(inner_list_array) = array.downcast_array_ref::<ArrowListArray>() else {
        if cfg!(debug_assertions) {
            panic!("downcast failed for {component_descriptor}, data discarded");
        } else {
            re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
        }
        return Either::Left(std::iter::empty());
    };

    let inner_offsets = inner_list_array.offsets();
    let inner_lengths = offsets_lengths(inner_list_array.offsets()).collect_vec();

    let Some(fixed_size_list_array) = inner_list_array
        .values()
        .downcast_array_ref::<ArrowFixedSizeListArray>()
    else {
        if cfg!(debug_assertions) {
            panic!("downcast failed for {component_descriptor}, data discarded");
        } else {
            re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
        }
        return Either::Left(std::iter::empty());
    };

    let Some(values) = fixed_size_list_array
        .values()
        .downcast_array_ref::<ArrowPrimitiveArray<P>>()
    else {
        if cfg!(debug_assertions) {
            panic!("downcast failed for {component_descriptor}, data discarded");
        } else {
            re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
        }
        return Either::Left(std::iter::empty());
    };

    let size = fixed_size_list_array.value_length() as usize;
    let values = values.values();

    // NOTE: No need for validity checks here, `iter_offsets` already takes care of that.
    Either::Right(component_spans.map(move |span| {
        let inner_offsets = &inner_offsets[span.range()];
        let inner_lengths = &inner_lengths[span.range()];
        izip!(inner_offsets, inner_lengths)
            .map(|(&idx, &len)| {
                let idx = idx as usize;
                bytemuck::cast_slice(&values[idx * size..idx * size + len * size])
            })
            .collect_vec()
    }))
}

// We use a macro instead of a blanket impl because this violates orphan rules.
macro_rules! impl_array_list_native_type {
    ($primitive_type:ty, $native_type:ty) => {
        impl<const N: usize> ChunkComponentSlicer for &[[$native_type; N]]
        where
            [$native_type; N]: bytemuck::Pod,
        {
            type Item<'a> = Vec<&'a [[$native_type; N]]>;

            fn slice<'a>(
                component_descriptor: ComponentDescriptor,
                array: &'a dyn ArrowArray,
                component_spans: impl Iterator<Item = Span<usize>> + 'a,
            ) -> impl Iterator<Item = Self::Item<'a>> + 'a {
                slice_as_array_list_native::<N, $primitive_type, $native_type>(
                    component_descriptor,
                    array,
                    component_spans,
                )
            }
        }
    };
}

impl_array_list_native_type!(arrow::array::types::UInt8Type, u8);
impl_array_list_native_type!(arrow::array::types::UInt16Type, u16);
impl_array_list_native_type!(arrow::array::types::UInt32Type, u32);
impl_array_list_native_type!(arrow::array::types::UInt64Type, u64);
// impl_array_list_native_type!(arrow::array::types::UInt128Type, u128);
impl_array_list_native_type!(arrow::array::types::Int8Type, i8);
impl_array_list_native_type!(arrow::array::types::Int16Type, i16);
impl_array_list_native_type!(arrow::array::types::Int32Type, i32);
impl_array_list_native_type!(arrow::array::types::Int64Type, i64);
// impl_array_list_native_type!(arrow::array::types::Int128Type, i128);
impl_array_list_native_type!(arrow::array::types::Float16Type, half::f16);
impl_array_list_native_type!(arrow::array::types::Float32Type, f32);
impl_array_list_native_type!(arrow::array::types::Float64Type, f64);

impl ChunkComponentSlicer for String {
    type Item<'a> = Vec<ArrowString>;

    fn slice<'a>(
        component_descriptor: ComponentDescriptor,
        array: &'a dyn ArrowArray,
        component_spans: impl Iterator<Item = Span<usize>> + 'a,
    ) -> impl Iterator<Item = Vec<ArrowString>> + 'a {
        let Some(utf8_array) = array.downcast_array_ref::<ArrowStringArray>() else {
            if cfg!(debug_assertions) {
                panic!("downcast failed for {component_descriptor}, data discarded");
            } else {
                re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
            }
            return Either::Left(std::iter::empty());
        };

        let values = utf8_array.values().clone();
        let offsets = utf8_array.offsets().clone();
        let lengths = offsets_lengths(utf8_array.offsets()).collect_vec();

        // NOTE: No need for validity checks here, `component_spans` already takes care of that.
        Either::Right(component_spans.map(move |range| {
            let offsets = &offsets[range.range()];
            let lengths = &lengths[range.range()];
            izip!(offsets, lengths)
                .map(|(&idx, &len)| ArrowString::from(values.slice_with_length(idx as _, len)))
                .collect_vec()
        }))
    }
}

impl ChunkComponentSlicer for bool {
    type Item<'a> = ArrowBooleanBuffer;

    fn slice<'a>(
        component_descriptor: ComponentDescriptor,
        array: &'a dyn ArrowArray,
        component_spans: impl Iterator<Item = Span<usize>> + 'a,
    ) -> impl Iterator<Item = Self::Item<'a>> + 'a {
        let Some(values) = array.downcast_array_ref::<ArrowBooleanArray>() else {
            if cfg!(debug_assertions) {
                panic!("downcast failed for {component_descriptor}, data discarded");
            } else {
                re_log::error_once!("downcast failed for {component_descriptor}, data discarded");
            }
            return Either::Left(std::iter::empty());
        };
        let values = values.values().clone();

        // NOTE: No need for validity checks here, `component_spans` already takes care of that.
        Either::Right(
            component_spans.map(move |Span { start, len }| values.clone().slice(start, len)),
        )
    }
}

// ---

pub struct ChunkIndicesIter {
    chunk: Arc<Chunk>,

    time_column: Option<TimeColumn>,
    index: usize,
}

impl Iterator for ChunkIndicesIter {
    type Item = (TimeInt, RowId);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.index;
        self.index += 1;

        let row_id = *self.chunk.row_ids_slice().get(i)?;

        if let Some(time_column) = &self.time_column {
            let time = *time_column.times_raw().get(i)?;
            let time = TimeInt::new_temporal(time);
            Some((time, row_id))
        } else {
            Some((TimeInt::STATIC, row_id))
        }
    }
}

impl Chunk {
    /// Returns an iterator over the indices (`(TimeInt, RowId)`) of a [`Chunk`], for a given timeline.
    ///
    /// If the chunk is static, `timeline` will be ignored.
    ///
    /// The returned iterator outlives `self`, thus it can be passed around freely.
    /// The tradeoff is that `self` must be an `Arc`.
    ///
    /// See also [`Self::iter_indices`].
    #[inline]
    pub fn iter_indices_owned(
        self: Arc<Self>,
        timeline: &TimelineName,
    ) -> impl Iterator<Item = (TimeInt, RowId)> + use<> {
        if self.is_static() {
            Either::Left(ChunkIndicesIter {
                chunk: self,
                time_column: None,
                index: 0,
            })
        } else {
            self.timelines.get(timeline).cloned().map_or_else(
                || Either::Right(Either::Left(std::iter::empty())),
                |time_column| {
                    Either::Right(Either::Right(ChunkIndicesIter {
                        chunk: self,
                        time_column: Some(time_column),
                        index: 0,
                    }))
                },
            )
        }
    }
}

// ---

/// The actual iterator implementation for [`Chunk::iter_component`].
pub struct ChunkComponentIter<C, IO> {
    values: Arc<Vec<C>>,
    offsets: IO,
}

/// The underlying item type for [`ChunkComponentIter`].
///
/// This allows us to cheaply carry slices of deserialized data, while working around the
/// limitations of Rust's Iterator trait and ecosystem.
///
/// See [`ChunkComponentIterItem::as_slice`].
#[derive(Clone, PartialEq)]
pub struct ChunkComponentIterItem<C> {
    values: Arc<Vec<C>>,
    span: Span<usize>,
}

impl<C: PartialEq> PartialEq<[C]> for ChunkComponentIterItem<C> {
    fn eq(&self, rhs: &[C]) -> bool {
        self.as_slice().eq(rhs)
    }
}

impl<C: PartialEq> PartialEq<Vec<C>> for ChunkComponentIterItem<C> {
    fn eq(&self, rhs: &Vec<C>) -> bool {
        self.as_slice().eq(rhs)
    }
}

impl<C: Eq> Eq for ChunkComponentIterItem<C> {}

// NOTE: No `C: Default`!
impl<C> Default for ChunkComponentIterItem<C> {
    #[inline]
    fn default() -> Self {
        Self {
            values: Arc::new(Vec::new()),
            span: Span::default(),
        }
    }
}

impl<C> ChunkComponentIterItem<C> {
    #[inline]
    pub fn as_slice(&self) -> &[C] {
        &self.values[self.span.range()]
    }
}

impl<C> std::ops::Deref for ChunkComponentIterItem<C> {
    type Target = [C];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<C: Component, IO: Iterator<Item = Span<usize>>> Iterator for ChunkComponentIter<C, IO> {
    type Item = ChunkComponentIterItem<C>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.offsets.next().map(move |span| ChunkComponentIterItem {
            values: Arc::clone(&self.values),
            span,
        })
    }
}

impl Chunk {
    /// Returns an iterator over the deserialized batches of a [`Chunk`], for a given component.
    ///
    /// This is a dedicated fast path: the entire column will be downcasted and deserialized at
    /// once, and then every component batch will be a slice reference into that global slice.
    /// Use this when working with complex arrow datatypes and performance matters (e.g. ranging
    /// through enum types across many timestamps).
    ///
    /// TODO(#5305): Note that, while this is much faster than deserializing each row individually,
    /// this still uses the old codegen'd deserialization path, which does some very unidiomatic Arrow
    /// things, and is therefore very slow at the moment. Avoid this on performance critical paths.
    ///
    /// See also:
    /// * [`Self::iter_slices`]
    /// * [`Self::iter_slices_from_struct_field`]
    #[inline]
    pub fn iter_component<C: Component>(
        &self,
        component_descriptor: &ComponentDescriptor,
    ) -> ChunkComponentIter<C, impl Iterator<Item = Span<usize>> + '_ + use<'_, C>> {
        debug_assert_eq!(
            component_descriptor.component_type,
            Some(C::name()),
            "component type mismatch"
        );

        let Some(list_array) = self.components.get(component_descriptor) else {
            return ChunkComponentIter {
                values: Arc::new(vec![]),
                offsets: Either::Left(std::iter::empty()),
            };
        };

        let values = arrow::array::ArrayRef::from(list_array.values().clone());
        let values = match C::from_arrow(&values) {
            Ok(values) => values,
            Err(err) => {
                if cfg!(debug_assertions) {
                    panic!(
                        "[DEBUG-ONLY] deserialization failed for {}, data discarded: {}",
                        C::name(),
                        re_error::format_ref(&err),
                    );
                } else {
                    re_log::error_once!(
                        "deserialization failed for {}, data discarded: {}",
                        C::name(),
                        re_error::format_ref(&err),
                    );
                }
                return ChunkComponentIter {
                    values: Arc::new(vec![]),
                    offsets: Either::Left(std::iter::empty()),
                };
            }
        };

        // NOTE: No need for validity checks here, `iter_offsets` already takes care of that.
        ChunkComponentIter {
            values: Arc::new(values),
            offsets: Either::Right(self.iter_component_offsets(component_descriptor)),
        }
    }
}

// ---

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use itertools::{Itertools as _, izip};
    use re_log_types::{
        EntityPath, TimeInt, TimePoint,
        example_components::{MyPoint, MyPoints},
    };

    use crate::{Chunk, RowId, Timeline};

    #[test]
    fn iter_indices_temporal() -> anyhow::Result<()> {
        let entity_path = EntityPath::from("this/that");

        let row_id1 = RowId::new();
        let row_id2 = RowId::new();
        let row_id3 = RowId::new();
        let row_id4 = RowId::new();
        let row_id5 = RowId::new();

        let timeline_frame = Timeline::new_sequence("frame");

        let timepoint1 = [(timeline_frame, 1)];
        let timepoint2 = [(timeline_frame, 3)];
        let timepoint3 = [(timeline_frame, 5)];
        let timepoint4 = [(timeline_frame, 7)];
        let timepoint5 = [(timeline_frame, 9)];

        let points1 = &[MyPoint::new(1.0, 1.0)];
        let points2 = &[MyPoint::new(2.0, 2.0)];
        let points3 = &[MyPoint::new(3.0, 3.0)];
        let points4 = &[MyPoint::new(4.0, 4.0)];
        let points5 = &[MyPoint::new(5.0, 5.0)];

        let chunk = Arc::new(
            Chunk::builder(entity_path.clone())
                .with_component_batches(
                    row_id1,
                    timepoint1,
                    [(MyPoints::descriptor_points(), points1 as _)],
                )
                .with_component_batches(
                    row_id2,
                    timepoint2,
                    [(MyPoints::descriptor_points(), points2 as _)],
                )
                .with_component_batches(
                    row_id3,
                    timepoint3,
                    [(MyPoints::descriptor_points(), points3 as _)],
                )
                .with_component_batches(
                    row_id4,
                    timepoint4,
                    [(MyPoints::descriptor_points(), points4 as _)],
                )
                .with_component_batches(
                    row_id5,
                    timepoint5,
                    [(MyPoints::descriptor_points(), points5 as _)],
                )
                .build()?,
        );

        {
            let got = Arc::clone(&chunk)
                .iter_indices_owned(timeline_frame.name())
                .collect_vec();
            let expected = izip!(
                chunk
                    .timelines
                    .get(timeline_frame.name())
                    .map(|time_column| time_column.times().collect_vec())
                    .unwrap_or_default(),
                chunk.row_ids()
            )
            .collect_vec();

            similar_asserts::assert_eq!(expected, got);
        }

        Ok(())
    }

    #[test]
    fn iter_indices_static() -> anyhow::Result<()> {
        let entity_path = EntityPath::from("this/that");

        let row_id1 = RowId::new();
        let row_id2 = RowId::new();
        let row_id3 = RowId::new();
        let row_id4 = RowId::new();
        let row_id5 = RowId::new();

        let timeline_frame = Timeline::new_sequence("frame");

        let points1 = &[MyPoint::new(1.0, 1.0)];
        let points2 = &[MyPoint::new(2.0, 2.0)];
        let points3 = &[MyPoint::new(3.0, 3.0)];
        let points4 = &[MyPoint::new(4.0, 4.0)];
        let points5 = &[MyPoint::new(5.0, 5.0)];

        let chunk = Arc::new(
            Chunk::builder(entity_path.clone())
                .with_component_batches(
                    row_id1,
                    TimePoint::default(),
                    [(MyPoints::descriptor_points(), points1 as _)],
                )
                .with_component_batches(
                    row_id2,
                    TimePoint::default(),
                    [(MyPoints::descriptor_points(), points2 as _)],
                )
                .with_component_batches(
                    row_id3,
                    TimePoint::default(),
                    [(MyPoints::descriptor_points(), points3 as _)],
                )
                .with_component_batches(
                    row_id4,
                    TimePoint::default(),
                    [(MyPoints::descriptor_points(), points4 as _)],
                )
                .with_component_batches(
                    row_id5,
                    TimePoint::default(),
                    [(MyPoints::descriptor_points(), points5 as _)],
                )
                .build()?,
        );

        {
            let got = Arc::clone(&chunk)
                .iter_indices_owned(timeline_frame.name())
                .collect_vec();
            let expected = izip!(std::iter::repeat(TimeInt::STATIC), chunk.row_ids()).collect_vec();

            similar_asserts::assert_eq!(expected, got);
        }

        Ok(())
    }
}
