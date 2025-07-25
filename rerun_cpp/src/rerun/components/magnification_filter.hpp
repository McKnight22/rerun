// DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/cpp/mod.rs
// Based on "crates/store/re_types/definitions/rerun/components/magnification_filter.fbs".

#pragma once

#include "../result.hpp"

#include <cstdint>
#include <memory>

namespace arrow {
    /// \private
    template <typename T>
    class NumericBuilder;

    class Array;
    class DataType;
    class UInt8Type;
    using UInt8Builder = NumericBuilder<UInt8Type>;
} // namespace arrow

namespace rerun::components {
    /// **Component**: Filter used when magnifying an image/texture such that a single pixel/texel is displayed as multiple pixels on screen.
    enum class MagnificationFilter : uint8_t {

        /// Show the nearest pixel value.
        ///
        /// This will give a blocky appearance when zooming in.
        /// Used as default when rendering 2D images.
        Nearest = 1,

        /// Linearly interpolate the nearest neighbors, creating a smoother look when zooming in.
        ///
        /// Used as default for mesh rendering.
        Linear = 2,
    };
} // namespace rerun::components

namespace rerun {
    template <typename T>
    struct Loggable;

    /// \private
    template <>
    struct Loggable<components::MagnificationFilter> {
        static constexpr std::string_view ComponentType = "rerun.components.MagnificationFilter";

        /// Returns the arrow data type this type corresponds to.
        static const std::shared_ptr<arrow::DataType>& arrow_datatype();

        /// Serializes an array of `rerun::components::MagnificationFilter` into an arrow array.
        static Result<std::shared_ptr<arrow::Array>> to_arrow(
            const components::MagnificationFilter* instances, size_t num_instances
        );

        /// Fills an arrow array builder with an array of this type.
        static rerun::Error fill_arrow_array_builder(
            arrow::UInt8Builder* builder, const components::MagnificationFilter* elements,
            size_t num_elements
        );
    };
} // namespace rerun
