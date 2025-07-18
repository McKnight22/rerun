// DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/cpp/mod.rs
// Based on "crates/store/re_types/definitions/rerun/blueprint/datatypes/selected_columns.fbs".

#pragma once

#include "../../collection.hpp"
#include "../../datatypes/utf8.hpp"
#include "../../result.hpp"
#include "component_column_selector.hpp"

#include <cstdint>
#include <memory>

namespace arrow {
    class Array;
    class DataType;
    class StructBuilder;
} // namespace arrow

namespace rerun::blueprint::datatypes {
    /// **Datatype**: List of selected columns in a dataframe.
    ///
    /// ⚠ **This type is _unstable_ and may change significantly in a way that the data won't be backwards compatible.**
    ///
    struct SelectedColumns {
        /// The time columns to include
        rerun::Collection<rerun::datatypes::Utf8> time_columns;

        /// The component columns to include
        rerun::Collection<rerun::blueprint::datatypes::ComponentColumnSelector> component_columns;

      public:
        SelectedColumns() = default;
    };
} // namespace rerun::blueprint::datatypes

namespace rerun {
    template <typename T>
    struct Loggable;

    /// \private
    template <>
    struct Loggable<blueprint::datatypes::SelectedColumns> {
        static constexpr std::string_view ComponentType =
            "rerun.blueprint.datatypes.SelectedColumns";

        /// Returns the arrow data type this type corresponds to.
        static const std::shared_ptr<arrow::DataType>& arrow_datatype();

        /// Serializes an array of `rerun::blueprint:: datatypes::SelectedColumns` into an arrow array.
        static Result<std::shared_ptr<arrow::Array>> to_arrow(
            const blueprint::datatypes::SelectedColumns* instances, size_t num_instances
        );

        /// Fills an arrow array builder with an array of this type.
        static rerun::Error fill_arrow_array_builder(
            arrow::StructBuilder* builder, const blueprint::datatypes::SelectedColumns* elements,
            size_t num_elements
        );
    };
} // namespace rerun
