// DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/cpp/mod.rs
// Based on "crates/store/re_types/definitions/rerun/archetypes/mesh3d.fbs".

#pragma once

#include "../collection.hpp"
#include "../component_batch.hpp"
#include "../component_column.hpp"
#include "../components/albedo_factor.hpp"
#include "../components/class_id.hpp"
#include "../components/color.hpp"
#include "../components/image_buffer.hpp"
#include "../components/image_format.hpp"
#include "../components/position3d.hpp"
#include "../components/texcoord2d.hpp"
#include "../components/triangle_indices.hpp"
#include "../components/vector3d.hpp"
#include "../result.hpp"

#include <cstdint>
#include <optional>
#include <utility>
#include <vector>

namespace rerun::archetypes {
    /// **Archetype**: A 3D triangle mesh as specified by its per-mesh and per-vertex properties.
    ///
    /// See also `archetypes::Asset3D`.
    ///
    /// If there are multiple `archetypes::InstancePoses3D` instances logged to the same entity as a mesh,
    /// an instance of the mesh will be drawn for each transform.
    ///
    /// ## Examples
    ///
    /// ### Simple indexed 3D mesh
    /// ![image](https://static.rerun.io/mesh3d_indexed/57c70dc992e6dc0bd9c5222ca084f5b6240cea75/full.png)
    ///
    /// ```cpp
    /// #include <rerun.hpp>
    ///
    /// #include <vector>
    ///
    /// int main() {
    ///     const auto rec = rerun::RecordingStream("rerun_example_mesh3d_indexed");
    ///     rec.spawn().exit_on_failure();
    ///
    ///     const rerun::Position3D vertex_positions[3] = {
    ///         {0.0f, 1.0f, 0.0f},
    ///         {1.0f, 0.0f, 0.0f},
    ///         {0.0f, 0.0f, 0.0f},
    ///     };
    ///     const rerun::Color vertex_colors[3] = {
    ///         {0, 0, 255},
    ///         {0, 255, 0},
    ///         {255, 0, 0},
    ///     };
    ///
    ///     rec.log(
    ///         "triangle",
    ///         rerun::Mesh3D(vertex_positions)
    ///             .with_vertex_normals({{0.0, 0.0, 1.0}})
    ///             .with_vertex_colors(vertex_colors)
    ///             .with_triangle_indices({{2, 1, 0}})
    ///     );
    /// }
    /// ```
    ///
    /// ### 3D mesh with instancing
    /// ![image](https://static.rerun.io/mesh3d_leaf_transforms3d/c2d0ee033129da53168f5705625a9b033f3a3d61/full.png)
    ///
    /// ```cpp
    /// #include <rerun.hpp>
    ///
    /// int main() {
    ///     const auto rec = rerun::RecordingStream("rerun_example_mesh3d_instancing");
    ///     rec.spawn().exit_on_failure();
    ///
    ///     rec.set_time_sequence("frame", 0);
    ///     rec.log(
    ///         "shape",
    ///         rerun::Mesh3D(
    ///             {{1.0f, 1.0f, 1.0f}, {-1.0f, -1.0f, 1.0f}, {-1.0f, 1.0f, -1.0f}, {1.0f, -1.0f, -1.0f}}
    ///         )
    ///             .with_triangle_indices({{0, 1, 2}, {0, 1, 3}, {0, 2, 3}, {1, 2, 3}})
    ///             .with_vertex_colors({0xFF0000FF, 0x00FF00FF, 0x00000FFFF, 0xFFFF00FF})
    ///     );
    ///     // This box will not be affected by its parent's instance poses!
    ///     rec.log("shape/box", rerun::Boxes3D::from_half_sizes({{5.0f, 5.0f, 5.0f}}));
    ///
    ///     for (int i = 0; i <100; ++i) {
    ///         rec.set_time_sequence("frame", i);
    ///         rec.log(
    ///             "shape",
    ///             rerun::InstancePoses3D()
    ///                 .with_translations(
    ///                     {{2.0f, 0.0f, 0.0f},
    ///                      {0.0f, 2.0f, 0.0f},
    ///                      {0.0f, -2.0f, 0.0f},
    ///                      {-2.0f, 0.0f, 0.0f}}
    ///                 )
    ///                 .with_rotation_axis_angles({rerun::RotationAxisAngle(
    ///                     {0.0f, 0.0f, 1.0f},
    ///                     rerun::Angle::degrees(static_cast<float>(i) * 2.0f)
    ///                 )})
    ///         );
    ///     }
    /// }
    /// ```
    struct Mesh3D {
        /// The positions of each vertex.
        ///
        /// If no `triangle_indices` are specified, then each triplet of positions is interpreted as a triangle.
        std::optional<ComponentBatch> vertex_positions;

        /// Optional indices for the triangles that make up the mesh.
        std::optional<ComponentBatch> triangle_indices;

        /// An optional normal for each vertex.
        std::optional<ComponentBatch> vertex_normals;

        /// An optional color for each vertex.
        std::optional<ComponentBatch> vertex_colors;

        /// An optional uv texture coordinate for each vertex.
        std::optional<ComponentBatch> vertex_texcoords;

        /// A color multiplier applied to the whole mesh.
        std::optional<ComponentBatch> albedo_factor;

        /// Optional albedo texture.
        ///
        /// Used with the `components::Texcoord2D` of the mesh.
        ///
        /// Currently supports only sRGB(A) textures, ignoring alpha.
        /// (meaning that the tensor must have 3 or 4 channels and use the `u8` format)
        std::optional<ComponentBatch> albedo_texture_buffer;

        /// The format of the `albedo_texture_buffer`, if any.
        std::optional<ComponentBatch> albedo_texture_format;

        /// Optional class Ids for the vertices.
        ///
        /// The `components::ClassId` provides colors and labels if not specified explicitly.
        std::optional<ComponentBatch> class_ids;

      public:
        /// The name of the archetype as used in `ComponentDescriptor`s.
        static constexpr const char ArchetypeName[] = "rerun.archetypes.Mesh3D";

        /// `ComponentDescriptor` for the `vertex_positions` field.
        static constexpr auto Descriptor_vertex_positions = ComponentDescriptor(
            ArchetypeName, "Mesh3D:vertex_positions",
            Loggable<rerun::components::Position3D>::ComponentType
        );
        /// `ComponentDescriptor` for the `triangle_indices` field.
        static constexpr auto Descriptor_triangle_indices = ComponentDescriptor(
            ArchetypeName, "Mesh3D:triangle_indices",
            Loggable<rerun::components::TriangleIndices>::ComponentType
        );
        /// `ComponentDescriptor` for the `vertex_normals` field.
        static constexpr auto Descriptor_vertex_normals = ComponentDescriptor(
            ArchetypeName, "Mesh3D:vertex_normals",
            Loggable<rerun::components::Vector3D>::ComponentType
        );
        /// `ComponentDescriptor` for the `vertex_colors` field.
        static constexpr auto Descriptor_vertex_colors = ComponentDescriptor(
            ArchetypeName, "Mesh3D:vertex_colors", Loggable<rerun::components::Color>::ComponentType
        );
        /// `ComponentDescriptor` for the `vertex_texcoords` field.
        static constexpr auto Descriptor_vertex_texcoords = ComponentDescriptor(
            ArchetypeName, "Mesh3D:vertex_texcoords",
            Loggable<rerun::components::Texcoord2D>::ComponentType
        );
        /// `ComponentDescriptor` for the `albedo_factor` field.
        static constexpr auto Descriptor_albedo_factor = ComponentDescriptor(
            ArchetypeName, "Mesh3D:albedo_factor",
            Loggable<rerun::components::AlbedoFactor>::ComponentType
        );
        /// `ComponentDescriptor` for the `albedo_texture_buffer` field.
        static constexpr auto Descriptor_albedo_texture_buffer = ComponentDescriptor(
            ArchetypeName, "Mesh3D:albedo_texture_buffer",
            Loggable<rerun::components::ImageBuffer>::ComponentType
        );
        /// `ComponentDescriptor` for the `albedo_texture_format` field.
        static constexpr auto Descriptor_albedo_texture_format = ComponentDescriptor(
            ArchetypeName, "Mesh3D:albedo_texture_format",
            Loggable<rerun::components::ImageFormat>::ComponentType
        );
        /// `ComponentDescriptor` for the `class_ids` field.
        static constexpr auto Descriptor_class_ids = ComponentDescriptor(
            ArchetypeName, "Mesh3D:class_ids", Loggable<rerun::components::ClassId>::ComponentType
        );

      public:
        Mesh3D() = default;
        Mesh3D(Mesh3D&& other) = default;
        Mesh3D(const Mesh3D& other) = default;
        Mesh3D& operator=(const Mesh3D& other) = default;
        Mesh3D& operator=(Mesh3D&& other) = default;

        explicit Mesh3D(Collection<rerun::components::Position3D> _vertex_positions)
            : vertex_positions(ComponentBatch::from_loggable(
                                   std::move(_vertex_positions), Descriptor_vertex_positions
              )
                                   .value_or_throw()) {}

        /// Update only some specific fields of a `Mesh3D`.
        static Mesh3D update_fields() {
            return Mesh3D();
        }

        /// Clear all the fields of a `Mesh3D`.
        static Mesh3D clear_fields();

        /// The positions of each vertex.
        ///
        /// If no `triangle_indices` are specified, then each triplet of positions is interpreted as a triangle.
        Mesh3D with_vertex_positions(
            const Collection<rerun::components::Position3D>& _vertex_positions
        ) && {
            vertex_positions =
                ComponentBatch::from_loggable(_vertex_positions, Descriptor_vertex_positions)
                    .value_or_throw();
            return std::move(*this);
        }

        /// Optional indices for the triangles that make up the mesh.
        Mesh3D with_triangle_indices(
            const Collection<rerun::components::TriangleIndices>& _triangle_indices
        ) && {
            triangle_indices =
                ComponentBatch::from_loggable(_triangle_indices, Descriptor_triangle_indices)
                    .value_or_throw();
            return std::move(*this);
        }

        /// An optional normal for each vertex.
        Mesh3D with_vertex_normals(const Collection<rerun::components::Vector3D>& _vertex_normals
        ) && {
            vertex_normals =
                ComponentBatch::from_loggable(_vertex_normals, Descriptor_vertex_normals)
                    .value_or_throw();
            return std::move(*this);
        }

        /// An optional color for each vertex.
        Mesh3D with_vertex_colors(const Collection<rerun::components::Color>& _vertex_colors) && {
            vertex_colors = ComponentBatch::from_loggable(_vertex_colors, Descriptor_vertex_colors)
                                .value_or_throw();
            return std::move(*this);
        }

        /// An optional uv texture coordinate for each vertex.
        Mesh3D with_vertex_texcoords(
            const Collection<rerun::components::Texcoord2D>& _vertex_texcoords
        ) && {
            vertex_texcoords =
                ComponentBatch::from_loggable(_vertex_texcoords, Descriptor_vertex_texcoords)
                    .value_or_throw();
            return std::move(*this);
        }

        /// A color multiplier applied to the whole mesh.
        Mesh3D with_albedo_factor(const rerun::components::AlbedoFactor& _albedo_factor) && {
            albedo_factor = ComponentBatch::from_loggable(_albedo_factor, Descriptor_albedo_factor)
                                .value_or_throw();
            return std::move(*this);
        }

        /// This method makes it possible to pack multiple `albedo_factor` in a single component batch.
        ///
        /// This only makes sense when used in conjunction with `columns`. `with_albedo_factor` should
        /// be used when logging a single row's worth of data.
        Mesh3D with_many_albedo_factor(
            const Collection<rerun::components::AlbedoFactor>& _albedo_factor
        ) && {
            albedo_factor = ComponentBatch::from_loggable(_albedo_factor, Descriptor_albedo_factor)
                                .value_or_throw();
            return std::move(*this);
        }

        /// Optional albedo texture.
        ///
        /// Used with the `components::Texcoord2D` of the mesh.
        ///
        /// Currently supports only sRGB(A) textures, ignoring alpha.
        /// (meaning that the tensor must have 3 or 4 channels and use the `u8` format)
        Mesh3D with_albedo_texture_buffer(
            const rerun::components::ImageBuffer& _albedo_texture_buffer
        ) && {
            albedo_texture_buffer = ComponentBatch::from_loggable(
                                        _albedo_texture_buffer,
                                        Descriptor_albedo_texture_buffer
            )
                                        .value_or_throw();
            return std::move(*this);
        }

        /// This method makes it possible to pack multiple `albedo_texture_buffer` in a single component batch.
        ///
        /// This only makes sense when used in conjunction with `columns`. `with_albedo_texture_buffer` should
        /// be used when logging a single row's worth of data.
        Mesh3D with_many_albedo_texture_buffer(
            const Collection<rerun::components::ImageBuffer>& _albedo_texture_buffer
        ) && {
            albedo_texture_buffer = ComponentBatch::from_loggable(
                                        _albedo_texture_buffer,
                                        Descriptor_albedo_texture_buffer
            )
                                        .value_or_throw();
            return std::move(*this);
        }

        /// The format of the `albedo_texture_buffer`, if any.
        Mesh3D with_albedo_texture_format(
            const rerun::components::ImageFormat& _albedo_texture_format
        ) && {
            albedo_texture_format = ComponentBatch::from_loggable(
                                        _albedo_texture_format,
                                        Descriptor_albedo_texture_format
            )
                                        .value_or_throw();
            return std::move(*this);
        }

        /// This method makes it possible to pack multiple `albedo_texture_format` in a single component batch.
        ///
        /// This only makes sense when used in conjunction with `columns`. `with_albedo_texture_format` should
        /// be used when logging a single row's worth of data.
        Mesh3D with_many_albedo_texture_format(
            const Collection<rerun::components::ImageFormat>& _albedo_texture_format
        ) && {
            albedo_texture_format = ComponentBatch::from_loggable(
                                        _albedo_texture_format,
                                        Descriptor_albedo_texture_format
            )
                                        .value_or_throw();
            return std::move(*this);
        }

        /// Optional class Ids for the vertices.
        ///
        /// The `components::ClassId` provides colors and labels if not specified explicitly.
        Mesh3D with_class_ids(const Collection<rerun::components::ClassId>& _class_ids) && {
            class_ids =
                ComponentBatch::from_loggable(_class_ids, Descriptor_class_ids).value_or_throw();
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

} // namespace rerun::archetypes

namespace rerun {
    /// \private
    template <typename T>
    struct AsComponents;

    /// \private
    template <>
    struct AsComponents<archetypes::Mesh3D> {
        /// Serialize all set component batches.
        static Result<Collection<ComponentBatch>> as_batches(const archetypes::Mesh3D& archetype);
    };
} // namespace rerun
