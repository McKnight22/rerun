# DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/python/mod.rs
# Based on "crates/store/re_types/definitions/rerun/blueprint/components/view_class.fbs".

# You can extend this class by creating a "ViewClassExt" class in "view_class_ext.py".

from __future__ import annotations

from ... import datatypes
from ..._baseclasses import (
    ComponentBatchMixin,
    ComponentMixin,
)

__all__ = ["ViewClass", "ViewClassBatch"]


class ViewClass(datatypes.Utf8, ComponentMixin):
    """
    **Component**: The class identifier of view, e.g. `"2D"`, `"TextLog"`, ….

    ⚠️ **This type is _unstable_ and may change significantly in a way that the data won't be backwards compatible.**
    """

    _BATCH_TYPE = None
    # You can define your own __init__ function as a member of ViewClassExt in view_class_ext.py

    # Note: there are no fields here because ViewClass delegates to datatypes.Utf8


class ViewClassBatch(datatypes.Utf8Batch, ComponentBatchMixin):
    _COMPONENT_TYPE: str = "rerun.blueprint.components.ViewClass"


# This is patched in late to avoid circular dependencies.
ViewClass._BATCH_TYPE = ViewClassBatch  # type: ignore[assignment]
