// DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/cpp/mod.rs
// Based on "crates/store/re_types/definitions/rerun/blueprint/archetypes/container_blueprint.fbs".

#pragma once

#include "../../blueprint/components/active_tab.hpp"
#include "../../blueprint/components/column_share.hpp"
#include "../../blueprint/components/container_kind.hpp"
#include "../../blueprint/components/grid_columns.hpp"
#include "../../blueprint/components/included_content.hpp"
#include "../../blueprint/components/row_share.hpp"
#include "../../collection.hpp"
#include "../../component_batch.hpp"
#include "../../component_column.hpp"
#include "../../components/name.hpp"
#include "../../components/visible.hpp"
#include "../../result.hpp"

#include <cstdint>
#include <optional>
#include <utility>
#include <vector>

namespace rerun::blueprint::archetypes {
    /// **Archetype**: The description of a container.
    ///
    /// ⚠ **This type is _unstable_ and may change significantly in a way that the data won't be backwards compatible.**
    ///
    struct ContainerBlueprint {
        /// The class of the view.
        std::optional<ComponentBatch> container_kind;

        /// The name of the container.
        std::optional<ComponentBatch> display_name;

        /// `ContainerId`s or `ViewId`s that are children of this container.
        std::optional<ComponentBatch> contents;

        /// The layout shares of each column in the container.
        ///
        /// For `components::ContainerKind::Horizontal` containers, the length of this list should always match the number of contents.
        ///
        /// Ignored for `components::ContainerKind::Vertical` containers.
        std::optional<ComponentBatch> col_shares;

        /// The layout shares of each row of the container.
        ///
        /// For `components::ContainerKind::Vertical` containers, the length of this list should always match the number of contents.
        ///
        /// Ignored for `components::ContainerKind::Horizontal` containers.
        std::optional<ComponentBatch> row_shares;

        /// Which tab is active.
        ///
        /// Only applies to `Tabs` containers.
        std::optional<ComponentBatch> active_tab;

        /// Whether this container is visible.
        ///
        /// Defaults to true if not specified.
        std::optional<ComponentBatch> visible;

        /// How many columns this grid should have.
        ///
        /// If unset, the grid layout will be auto.
        ///
        /// Ignored for `components::ContainerKind::Horizontal`/`components::ContainerKind::Vertical` containers.
        std::optional<ComponentBatch> grid_columns;

      public:
        /// The name of the archetype as used in `ComponentDescriptor`s.
        static constexpr const char ArchetypeName[] =
            "rerun.blueprint.archetypes.ContainerBlueprint";

        /// `ComponentDescriptor` for the `container_kind` field.
        static constexpr auto Descriptor_container_kind = ComponentDescriptor(
            ArchetypeName, "ContainerBlueprint:container_kind",
            Loggable<rerun::blueprint::components::ContainerKind>::ComponentType
        );
        /// `ComponentDescriptor` for the `display_name` field.
        static constexpr auto Descriptor_display_name = ComponentDescriptor(
            ArchetypeName, "ContainerBlueprint:display_name",
            Loggable<rerun::components::Name>::ComponentType
        );
        /// `ComponentDescriptor` for the `contents` field.
        static constexpr auto Descriptor_contents = ComponentDescriptor(
            ArchetypeName, "ContainerBlueprint:contents",
            Loggable<rerun::blueprint::components::IncludedContent>::ComponentType
        );
        /// `ComponentDescriptor` for the `col_shares` field.
        static constexpr auto Descriptor_col_shares = ComponentDescriptor(
            ArchetypeName, "ContainerBlueprint:col_shares",
            Loggable<rerun::blueprint::components::ColumnShare>::ComponentType
        );
        /// `ComponentDescriptor` for the `row_shares` field.
        static constexpr auto Descriptor_row_shares = ComponentDescriptor(
            ArchetypeName, "ContainerBlueprint:row_shares",
            Loggable<rerun::blueprint::components::RowShare>::ComponentType
        );
        /// `ComponentDescriptor` for the `active_tab` field.
        static constexpr auto Descriptor_active_tab = ComponentDescriptor(
            ArchetypeName, "ContainerBlueprint:active_tab",
            Loggable<rerun::blueprint::components::ActiveTab>::ComponentType
        );
        /// `ComponentDescriptor` for the `visible` field.
        static constexpr auto Descriptor_visible = ComponentDescriptor(
            ArchetypeName, "ContainerBlueprint:visible",
            Loggable<rerun::components::Visible>::ComponentType
        );
        /// `ComponentDescriptor` for the `grid_columns` field.
        static constexpr auto Descriptor_grid_columns = ComponentDescriptor(
            ArchetypeName, "ContainerBlueprint:grid_columns",
            Loggable<rerun::blueprint::components::GridColumns>::ComponentType
        );

      public:
        ContainerBlueprint() = default;
        ContainerBlueprint(ContainerBlueprint&& other) = default;
        ContainerBlueprint(const ContainerBlueprint& other) = default;
        ContainerBlueprint& operator=(const ContainerBlueprint& other) = default;
        ContainerBlueprint& operator=(ContainerBlueprint&& other) = default;

        explicit ContainerBlueprint(rerun::blueprint::components::ContainerKind _container_kind)
            : container_kind(ComponentBatch::from_loggable(
                                 std::move(_container_kind), Descriptor_container_kind
              )
                                 .value_or_throw()) {}

        /// Update only some specific fields of a `ContainerBlueprint`.
        static ContainerBlueprint update_fields() {
            return ContainerBlueprint();
        }

        /// Clear all the fields of a `ContainerBlueprint`.
        static ContainerBlueprint clear_fields();

        /// The class of the view.
        ContainerBlueprint with_container_kind(
            const rerun::blueprint::components::ContainerKind& _container_kind
        ) && {
            container_kind =
                ComponentBatch::from_loggable(_container_kind, Descriptor_container_kind)
                    .value_or_throw();
            return std::move(*this);
        }

        /// The name of the container.
        ContainerBlueprint with_display_name(const rerun::components::Name& _display_name) && {
            display_name = ComponentBatch::from_loggable(_display_name, Descriptor_display_name)
                               .value_or_throw();
            return std::move(*this);
        }

        /// `ContainerId`s or `ViewId`s that are children of this container.
        ContainerBlueprint with_contents(
            const Collection<rerun::blueprint::components::IncludedContent>& _contents
        ) && {
            contents =
                ComponentBatch::from_loggable(_contents, Descriptor_contents).value_or_throw();
            return std::move(*this);
        }

        /// The layout shares of each column in the container.
        ///
        /// For `components::ContainerKind::Horizontal` containers, the length of this list should always match the number of contents.
        ///
        /// Ignored for `components::ContainerKind::Vertical` containers.
        ContainerBlueprint with_col_shares(
            const Collection<rerun::blueprint::components::ColumnShare>& _col_shares
        ) && {
            col_shares =
                ComponentBatch::from_loggable(_col_shares, Descriptor_col_shares).value_or_throw();
            return std::move(*this);
        }

        /// The layout shares of each row of the container.
        ///
        /// For `components::ContainerKind::Vertical` containers, the length of this list should always match the number of contents.
        ///
        /// Ignored for `components::ContainerKind::Horizontal` containers.
        ContainerBlueprint with_row_shares(
            const Collection<rerun::blueprint::components::RowShare>& _row_shares
        ) && {
            row_shares =
                ComponentBatch::from_loggable(_row_shares, Descriptor_row_shares).value_or_throw();
            return std::move(*this);
        }

        /// Which tab is active.
        ///
        /// Only applies to `Tabs` containers.
        ContainerBlueprint with_active_tab(
            const rerun::blueprint::components::ActiveTab& _active_tab
        ) && {
            active_tab =
                ComponentBatch::from_loggable(_active_tab, Descriptor_active_tab).value_or_throw();
            return std::move(*this);
        }

        /// Whether this container is visible.
        ///
        /// Defaults to true if not specified.
        ContainerBlueprint with_visible(const rerun::components::Visible& _visible) && {
            visible = ComponentBatch::from_loggable(_visible, Descriptor_visible).value_or_throw();
            return std::move(*this);
        }

        /// How many columns this grid should have.
        ///
        /// If unset, the grid layout will be auto.
        ///
        /// Ignored for `components::ContainerKind::Horizontal`/`components::ContainerKind::Vertical` containers.
        ContainerBlueprint with_grid_columns(
            const rerun::blueprint::components::GridColumns& _grid_columns
        ) && {
            grid_columns = ComponentBatch::from_loggable(_grid_columns, Descriptor_grid_columns)
                               .value_or_throw();
            return std::move(*this);
        }

        /// Partitions the component data into multiple sub-batches.
        ///
        /// Specifically, this transforms the existing `ComponentBatch` data into `ComponentColumn`s
        /// instead, via `ComponentBatch::partitioned`.
        ///
        /// This makes it possible to use `RecordingStream::send_columns` to send columnar data directly into Rerun.
        ///
        /// The specified `lengths` must sum to the total length of the component batch.
        Collection<ComponentColumn> columns(const Collection<uint32_t>& lengths_);

        /// Partitions the component data into unit-length sub-batches.
        ///
        /// This is semantically similar to calling `columns` with `std::vector<uint32_t>(n, 1)`,
        /// where `n` is automatically guessed.
        Collection<ComponentColumn> columns();
    };

} // namespace rerun::blueprint::archetypes

namespace rerun {
    /// \private
    template <typename T>
    struct AsComponents;

    /// \private
    template <>
    struct AsComponents<blueprint::archetypes::ContainerBlueprint> {
        /// Serialize all set component batches.
        static Result<Collection<ComponentBatch>> as_batches(
            const blueprint::archetypes::ContainerBlueprint& archetype
        );
    };
} // namespace rerun
