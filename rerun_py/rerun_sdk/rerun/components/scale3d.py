# DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/python/mod.rs
# Based on "crates/store/re_types/definitions/rerun/components/scale3d.fbs".

# You can extend this class by creating a "Scale3DExt" class in "scale3d_ext.py".

from __future__ import annotations

from .. import datatypes
from .._baseclasses import (
    ComponentBatchMixin,
    ComponentMixin,
)
from .scale3d_ext import Scale3DExt

__all__ = ["Scale3D", "Scale3DBatch"]


class Scale3D(Scale3DExt, datatypes.Vec3D, ComponentMixin):
    """
    **Component**: A 3D scale factor.

    A scale of 1.0 means no scaling.
    A scale of 2.0 means doubling the size.
    Each component scales along the corresponding axis.
    """

    _BATCH_TYPE = None
    # __init__ can be found in scale3d_ext.py

    # Note: there are no fields here because Scale3D delegates to datatypes.Vec3D


class Scale3DBatch(datatypes.Vec3DBatch, ComponentBatchMixin):
    _COMPONENT_TYPE: str = "rerun.components.Scale3D"


# This is patched in late to avoid circular dependencies.
Scale3D._BATCH_TYPE = Scale3DBatch  # type: ignore[assignment]
