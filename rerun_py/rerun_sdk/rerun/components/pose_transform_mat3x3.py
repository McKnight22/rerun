# DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/python/mod.rs
# Based on "crates/store/re_types/definitions/rerun/components/transform_mat3x3.fbs".

# You can extend this class by creating a "PoseTransformMat3x3Ext" class in "pose_transform_mat3x3_ext.py".

from __future__ import annotations

from .. import datatypes
from .._baseclasses import (
    ComponentBatchMixin,
    ComponentMixin,
)

__all__ = ["PoseTransformMat3x3", "PoseTransformMat3x3Batch"]


class PoseTransformMat3x3(datatypes.Mat3x3, ComponentMixin):
    """
    **Component**: A 3x3 transformation matrix Matrix that doesn't propagate in the transform hierarchy.

    3x3 matrixes are able to represent any affine transformation in 3D space,
    i.e. rotation, scaling, shearing, reflection etc.

    Matrices in Rerun are stored as flat list of coefficients in column-major order:
    ```text
                column 0       column 1       column 2
           -------------------------------------------------
    row 0 | flat_columns[0] flat_columns[3] flat_columns[6]
    row 1 | flat_columns[1] flat_columns[4] flat_columns[7]
    row 2 | flat_columns[2] flat_columns[5] flat_columns[8]
    ```

    However, construction is done from a list of rows, which follows NumPy's convention:
    ```python
    np.testing.assert_array_equal(
        rr.components.PoseTransformMat3x3([1, 2, 3, 4, 5, 6, 7, 8, 9]).flat_columns, np.array([1, 4, 7, 2, 5, 8, 3, 6, 9], dtype=np.float32)
    )
    np.testing.assert_array_equal(
        rr.components.PoseTransformMat3x3([[1, 2, 3], [4, 5, 6], [7, 8, 9]]).flat_columns,
        np.array([1, 4, 7, 2, 5, 8, 3, 6, 9], dtype=np.float32),
    )
    ```
    If you want to construct a matrix from a list of columns instead, use the named `columns` parameter:
    ```python
    np.testing.assert_array_equal(
        rr.components.PoseTransformMat3x3(columns=[1, 2, 3, 4, 5, 6, 7, 8, 9]).flat_columns,
        np.array([1, 2, 3, 4, 5, 6, 7, 8, 9], dtype=np.float32),
    )
    np.testing.assert_array_equal(
        rr.components.PoseTransformMat3x3(columns=[[1, 2, 3], [4, 5, 6], [7, 8, 9]]).flat_columns,
        np.array([1, 2, 3, 4, 5, 6, 7, 8, 9], dtype=np.float32),
    )
    ```
    """

    _BATCH_TYPE = None
    # You can define your own __init__ function as a member of PoseTransformMat3x3Ext in pose_transform_mat3x3_ext.py

    # Note: there are no fields here because PoseTransformMat3x3 delegates to datatypes.Mat3x3


class PoseTransformMat3x3Batch(datatypes.Mat3x3Batch, ComponentBatchMixin):
    _COMPONENT_TYPE: str = "rerun.components.PoseTransformMat3x3"


# This is patched in late to avoid circular dependencies.
PoseTransformMat3x3._BATCH_TYPE = PoseTransformMat3x3Batch  # type: ignore[assignment]
