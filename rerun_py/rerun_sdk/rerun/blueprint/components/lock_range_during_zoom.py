# DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/python/mod.rs
# Based on "crates/store/re_types/definitions/rerun/blueprint/components/lock_range_during_zoom.fbs".

# You can extend this class by creating a "LockRangeDuringZoomExt" class in "lock_range_during_zoom_ext.py".

from __future__ import annotations

from ... import datatypes
from ..._baseclasses import (
    ComponentBatchMixin,
    ComponentMixin,
)

__all__ = ["LockRangeDuringZoom", "LockRangeDuringZoomBatch"]


class LockRangeDuringZoom(datatypes.Bool, ComponentMixin):
    """
    **Component**: Indicate whether the range should be locked when zooming in on the data.

    Default is `false`, i.e. zoom will change the visualized range.

    ⚠️ **This type is _unstable_ and may change significantly in a way that the data won't be backwards compatible.**
    """

    _BATCH_TYPE = None
    # You can define your own __init__ function as a member of LockRangeDuringZoomExt in lock_range_during_zoom_ext.py

    # Note: there are no fields here because LockRangeDuringZoom delegates to datatypes.Bool


class LockRangeDuringZoomBatch(datatypes.BoolBatch, ComponentBatchMixin):
    _COMPONENT_TYPE: str = "rerun.blueprint.components.LockRangeDuringZoom"


# This is patched in late to avoid circular dependencies.
LockRangeDuringZoom._BATCH_TYPE = LockRangeDuringZoomBatch  # type: ignore[assignment]
