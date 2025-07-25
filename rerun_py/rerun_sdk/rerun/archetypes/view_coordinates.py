# DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/python/mod.rs
# Based on "crates/store/re_types/definitions/rerun/archetypes/view_coordinates.fbs".

# You can extend this class by creating a "ViewCoordinatesExt" class in "view_coordinates_ext.py".

from __future__ import annotations

from typing import Any

import numpy as np
import pyarrow as pa
from attrs import define, field

from .. import components, datatypes
from .._baseclasses import (
    Archetype,
    ComponentColumnList,
)
from ..error_utils import catch_and_log_exceptions
from .view_coordinates_ext import ViewCoordinatesExt

__all__ = ["ViewCoordinates"]


@define(str=False, repr=False, init=False)
class ViewCoordinates(ViewCoordinatesExt, Archetype):
    """
    **Archetype**: How we interpret the coordinate system of an entity/space.

    For instance: What is "up"? What does the Z axis mean?

    The three coordinates are always ordered as [x, y, z].

    For example [Right, Down, Forward] means that the X axis points to the right, the Y axis points
    down, and the Z axis points forward.

    Make sure that this archetype is logged at or above the origin entity path of your 3D views.

    ⚠ [Rerun does not yet support left-handed coordinate systems](https://github.com/rerun-io/rerun/issues/5032).

    ⚠️ **This type is _unstable_ and may change significantly in a way that the data won't be backwards compatible.**

    Example
    -------
    ### View coordinates for adjusting the eye camera:
    ```python
    import rerun as rr

    rr.init("rerun_example_view_coordinates", spawn=True)

    rr.log("world", rr.ViewCoordinates.RIGHT_HAND_Z_UP, static=True)  # Set an up-axis
    rr.log(
        "world/xyz",
        rr.Arrows3D(
            vectors=[[1, 0, 0], [0, 1, 0], [0, 0, 1]],
            colors=[[255, 0, 0], [0, 255, 0], [0, 0, 255]],
        ),
    )
    ```
    <center>
    <picture>
      <source media="(max-width: 480px)" srcset="https://static.rerun.io/viewcoordinates/0833f0dc8616a676b7b2c566f2a6f613363680c5/480w.png">
      <source media="(max-width: 768px)" srcset="https://static.rerun.io/viewcoordinates/0833f0dc8616a676b7b2c566f2a6f613363680c5/768w.png">
      <source media="(max-width: 1024px)" srcset="https://static.rerun.io/viewcoordinates/0833f0dc8616a676b7b2c566f2a6f613363680c5/1024w.png">
      <source media="(max-width: 1200px)" srcset="https://static.rerun.io/viewcoordinates/0833f0dc8616a676b7b2c566f2a6f613363680c5/1200w.png">
      <img src="https://static.rerun.io/viewcoordinates/0833f0dc8616a676b7b2c566f2a6f613363680c5/full.png" width="640">
    </picture>
    </center>

    """

    def __init__(self: Any, xyz: datatypes.ViewCoordinatesLike) -> None:
        """
        Create a new instance of the ViewCoordinates archetype.

        Parameters
        ----------
        xyz:
            The directions of the [x, y, z] axes.

        """

        # You can define your own __init__ function as a member of ViewCoordinatesExt in view_coordinates_ext.py
        with catch_and_log_exceptions(context=self.__class__.__name__):
            self.__attrs_init__(xyz=xyz)
            return
        self.__attrs_clear__()

    def __attrs_clear__(self) -> None:
        """Convenience method for calling `__attrs_init__` with all `None`s."""
        self.__attrs_init__(
            xyz=None,
        )

    @classmethod
    def _clear(cls) -> ViewCoordinates:
        """Produce an empty ViewCoordinates, bypassing `__init__`."""
        inst = cls.__new__(cls)
        inst.__attrs_clear__()
        return inst

    @classmethod
    def from_fields(
        cls,
        *,
        clear_unset: bool = False,
        xyz: datatypes.ViewCoordinatesLike | None = None,
    ) -> ViewCoordinates:
        """
        Update only some specific fields of a `ViewCoordinates`.

        Parameters
        ----------
        clear_unset:
            If true, all unspecified fields will be explicitly cleared.
        xyz:
            The directions of the [x, y, z] axes.

        """

        inst = cls.__new__(cls)
        with catch_and_log_exceptions(context=cls.__name__):
            kwargs = {
                "xyz": xyz,
            }

            if clear_unset:
                kwargs = {k: v if v is not None else [] for k, v in kwargs.items()}  # type: ignore[misc]

            inst.__attrs_init__(**kwargs)
            return inst

        inst.__attrs_clear__()
        return inst

    @classmethod
    def cleared(cls) -> ViewCoordinates:
        """Clear all the fields of a `ViewCoordinates`."""
        return cls.from_fields(clear_unset=True)

    @classmethod
    def columns(
        cls,
        *,
        xyz: datatypes.ViewCoordinatesArrayLike | None = None,
    ) -> ComponentColumnList:
        """
        Construct a new column-oriented component bundle.

        This makes it possible to use `rr.send_columns` to send columnar data directly into Rerun.

        The returned columns will be partitioned into unit-length sub-batches by default.
        Use `ComponentColumnList.partition` to repartition the data as needed.

        Parameters
        ----------
        xyz:
            The directions of the [x, y, z] axes.

        """

        inst = cls.__new__(cls)
        with catch_and_log_exceptions(context=cls.__name__):
            inst.__attrs_init__(
                xyz=xyz,
            )

        batches = inst.as_component_batches()
        if len(batches) == 0:
            return ComponentColumnList([])

        kwargs = {"ViewCoordinates:xyz": xyz}
        columns = []

        for batch in batches:
            arrow_array = batch.as_arrow_array()

            # For primitive arrays and fixed size list arrays, we infer partition size from the input shape.
            if pa.types.is_primitive(arrow_array.type) or pa.types.is_fixed_size_list(arrow_array.type):
                param = kwargs[batch.component_descriptor().component]  # type: ignore[index]
                shape = np.shape(param)  # type: ignore[arg-type]
                elem_flat_len = int(np.prod(shape[1:])) if len(shape) > 1 else 1  # type: ignore[redundant-expr,misc]

                if pa.types.is_fixed_size_list(arrow_array.type) and arrow_array.type.list_size == elem_flat_len:
                    # If the product of the last dimensions of the shape are equal to the size of the fixed size list array,
                    # we have `num_rows` single element batches (each element is a fixed sized list).
                    # (This should have been already validated by conversion to the arrow_array)
                    batch_length = 1
                else:
                    batch_length = shape[1] if len(shape) > 1 else 1  # type: ignore[redundant-expr,misc]

                num_rows = shape[0] if len(shape) >= 1 else 1  # type: ignore[redundant-expr,misc]
                sizes = batch_length * np.ones(num_rows)
            else:
                # For non-primitive types, default to partitioning each element separately.
                sizes = np.ones(len(arrow_array))

            columns.append(batch.partition(sizes))

        return ComponentColumnList(columns)

    xyz: components.ViewCoordinatesBatch | None = field(
        metadata={"component": True},
        default=None,
        converter=components.ViewCoordinatesBatch._converter,  # type: ignore[misc]
    )
    # The directions of the [x, y, z] axes.
    #
    # (Docstring intentionally commented out to hide this field from the docs)

    __str__ = Archetype.__str__
    __repr__ = Archetype.__repr__  # type: ignore[assignment]


ViewCoordinatesExt.deferred_patch_class(ViewCoordinates)
