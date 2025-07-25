// DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/rust/api.rs
// Based on "crates/store/re_types/definitions/rerun/blueprint/views/time_series.fbs".

#![allow(unused_braces)]
#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::cloned_instead_of_copied)]
#![allow(clippy::map_flatten)]
#![allow(clippy::needless_question_mark)]
#![allow(clippy::new_without_default)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]

use ::re_types_core::try_serialize_field;
use ::re_types_core::SerializationResult;
use ::re_types_core::{ComponentBatch as _, SerializedComponentBatch};
use ::re_types_core::{ComponentDescriptor, ComponentType};
use ::re_types_core::{DeserializationError, DeserializationResult};

/// **View**: A time series view for scalars over time, for use with [`archetypes::Scalars`][crate::archetypes::Scalars].
///
/// ⚠️ **This type is _unstable_ and may change significantly in a way that the data won't be backwards compatible.**
#[derive(Clone, Debug)]
pub struct TimeSeriesView {
    /// Configures the horizontal axis of the plot.
    pub axis_x: crate::blueprint::archetypes::TimeAxis,

    /// Configures the vertical axis of the plot.
    pub axis_y: crate::blueprint::archetypes::ScalarAxis,

    /// Configures the legend of the plot.
    pub plot_legend: crate::blueprint::archetypes::PlotLegend,

    /// Configures which range on each timeline is shown by this view (unless specified differently per entity).
    ///
    /// If not specified, the default is to show the entire timeline.
    /// If a timeline is specified more than once, the first entry will be used.
    pub time_ranges: crate::blueprint::archetypes::VisibleTimeRanges,
}

impl ::re_types_core::View for TimeSeriesView {
    #[inline]
    fn identifier() -> ::re_types_core::ViewClassIdentifier {
        "TimeSeries".into()
    }
}

impl ::re_byte_size::SizeBytes for TimeSeriesView {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        self.axis_x.heap_size_bytes()
            + self.axis_y.heap_size_bytes()
            + self.plot_legend.heap_size_bytes()
            + self.time_ranges.heap_size_bytes()
    }

    #[inline]
    fn is_pod() -> bool {
        <crate::blueprint::archetypes::TimeAxis>::is_pod()
            && <crate::blueprint::archetypes::ScalarAxis>::is_pod()
            && <crate::blueprint::archetypes::PlotLegend>::is_pod()
            && <crate::blueprint::archetypes::VisibleTimeRanges>::is_pod()
    }
}
