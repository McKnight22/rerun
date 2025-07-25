// DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/rust/api.rs
// Based on "crates/store/re_types/definitions/rerun/datatypes/rotation_axis_angle.fbs".

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

/// **Datatype**: 3D rotation represented by a rotation around a given axis.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct RotationAxisAngle {
    /// Axis to rotate around.
    ///
    /// This is not required to be normalized.
    /// However, if normalization of the rotation axis fails (typically due to a zero vector)
    /// the rotation is treated as an invalid transform, unless the angle is zero in which case
    /// it is treated as an identity.
    pub axis: crate::datatypes::Vec3D,

    /// How much to rotate around the axis.
    pub angle: crate::datatypes::Angle,
}

::re_types_core::macros::impl_into_cow!(RotationAxisAngle);

impl ::re_types_core::Loggable for RotationAxisAngle {
    #[inline]
    fn arrow_datatype() -> arrow::datatypes::DataType {
        #![allow(clippy::wildcard_imports)]
        use arrow::datatypes::*;
        DataType::Struct(Fields::from(vec![
            Field::new("axis", <crate::datatypes::Vec3D>::arrow_datatype(), false),
            Field::new("angle", <crate::datatypes::Angle>::arrow_datatype(), false),
        ]))
    }

    fn to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<::std::borrow::Cow<'a, Self>>>>,
    ) -> SerializationResult<arrow::array::ArrayRef>
    where
        Self: Clone + 'a,
    {
        #![allow(clippy::wildcard_imports)]
        #![allow(clippy::manual_is_variant_and)]
        use ::re_types_core::{arrow_helpers::as_array_ref, Loggable as _, ResultExt as _};
        use arrow::{array::*, buffer::*, datatypes::*};
        Ok({
            let fields = Fields::from(vec![
                Field::new("axis", <crate::datatypes::Vec3D>::arrow_datatype(), false),
                Field::new("angle", <crate::datatypes::Angle>::arrow_datatype(), false),
            ]);
            let (somes, data): (Vec<_>, Vec<_>) = data
                .into_iter()
                .map(|datum| {
                    let datum: Option<::std::borrow::Cow<'a, Self>> = datum.map(Into::into);
                    (datum.is_some(), datum)
                })
                .unzip();
            let validity: Option<arrow::buffer::NullBuffer> = {
                let any_nones = somes.iter().any(|some| !*some);
                any_nones.then(|| somes.into())
            };
            as_array_ref(StructArray::new(
                fields,
                vec![
                    {
                        let (somes, axis): (Vec<_>, Vec<_>) = data
                            .iter()
                            .map(|datum| {
                                let datum = datum.as_ref().map(|datum| datum.axis.clone());
                                (datum.is_some(), datum)
                            })
                            .unzip();
                        let axis_validity: Option<arrow::buffer::NullBuffer> = {
                            let any_nones = somes.iter().any(|some| !*some);
                            any_nones.then(|| somes.into())
                        };
                        {
                            let axis_inner_data: Vec<_> = axis
                                .into_iter()
                                .map(|datum| datum.map(|datum| datum.0).unwrap_or_default())
                                .flatten()
                                .collect();
                            let axis_inner_validity: Option<arrow::buffer::NullBuffer> =
                                axis_validity.as_ref().map(|validity| {
                                    validity
                                        .iter()
                                        .map(|b| std::iter::repeat(b).take(3usize))
                                        .flatten()
                                        .collect::<Vec<_>>()
                                        .into()
                                });
                            as_array_ref(FixedSizeListArray::new(
                                std::sync::Arc::new(Field::new("item", DataType::Float32, false)),
                                3,
                                as_array_ref(PrimitiveArray::<Float32Type>::new(
                                    ScalarBuffer::from(
                                        axis_inner_data.into_iter().collect::<Vec<_>>(),
                                    ),
                                    axis_inner_validity,
                                )),
                                axis_validity,
                            ))
                        }
                    },
                    {
                        let (somes, angle): (Vec<_>, Vec<_>) = data
                            .iter()
                            .map(|datum| {
                                let datum = datum.as_ref().map(|datum| datum.angle.clone());
                                (datum.is_some(), datum)
                            })
                            .unzip();
                        let angle_validity: Option<arrow::buffer::NullBuffer> = {
                            let any_nones = somes.iter().any(|some| !*some);
                            any_nones.then(|| somes.into())
                        };
                        as_array_ref(PrimitiveArray::<Float32Type>::new(
                            ScalarBuffer::from(
                                angle
                                    .into_iter()
                                    .map(|datum| {
                                        datum.map(|datum| datum.radians).unwrap_or_default()
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                            angle_validity,
                        ))
                    },
                ],
                validity,
            ))
        })
    }

    fn from_arrow_opt(
        arrow_data: &dyn arrow::array::Array,
    ) -> DeserializationResult<Vec<Option<Self>>>
    where
        Self: Sized,
    {
        #![allow(clippy::wildcard_imports)]
        use ::re_types_core::{arrow_zip_validity::ZipValidity, Loggable as _, ResultExt as _};
        use arrow::{array::*, buffer::*, datatypes::*};
        Ok({
            let arrow_data = arrow_data
                .as_any()
                .downcast_ref::<arrow::array::StructArray>()
                .ok_or_else(|| {
                    let expected = Self::arrow_datatype();
                    let actual = arrow_data.data_type().clone();
                    DeserializationError::datatype_mismatch(expected, actual)
                })
                .with_context("rerun.datatypes.RotationAxisAngle")?;
            if arrow_data.is_empty() {
                Vec::new()
            } else {
                let (arrow_data_fields, arrow_data_arrays) =
                    (arrow_data.fields(), arrow_data.columns());
                let arrays_by_name: ::std::collections::HashMap<_, _> = arrow_data_fields
                    .iter()
                    .map(|field| field.name().as_str())
                    .zip(arrow_data_arrays)
                    .collect();
                let axis = {
                    if !arrays_by_name.contains_key("axis") {
                        return Err(DeserializationError::missing_struct_field(
                            Self::arrow_datatype(),
                            "axis",
                        ))
                        .with_context("rerun.datatypes.RotationAxisAngle");
                    }
                    let arrow_data = &**arrays_by_name["axis"];
                    {
                        let arrow_data = arrow_data
                            .as_any()
                            .downcast_ref::<arrow::array::FixedSizeListArray>()
                            .ok_or_else(|| {
                                let expected = DataType::FixedSizeList(
                                    std::sync::Arc::new(Field::new(
                                        "item",
                                        DataType::Float32,
                                        false,
                                    )),
                                    3,
                                );
                                let actual = arrow_data.data_type().clone();
                                DeserializationError::datatype_mismatch(expected, actual)
                            })
                            .with_context("rerun.datatypes.RotationAxisAngle#axis")?;
                        if arrow_data.is_empty() {
                            Vec::new()
                        } else {
                            let offsets = (0..)
                                .step_by(3usize)
                                .zip((3usize..).step_by(3usize).take(arrow_data.len()));
                            let arrow_data_inner = {
                                let arrow_data_inner = &**arrow_data.values();
                                arrow_data_inner
                                    .as_any()
                                    .downcast_ref::<Float32Array>()
                                    .ok_or_else(|| {
                                        let expected = DataType::Float32;
                                        let actual = arrow_data_inner.data_type().clone();
                                        DeserializationError::datatype_mismatch(expected, actual)
                                    })
                                    .with_context("rerun.datatypes.RotationAxisAngle#axis")?
                                    .into_iter()
                                    .collect::<Vec<_>>()
                            };
                            ZipValidity::new_with_validity(offsets, arrow_data.nulls())
                                .map(|elem| {
                                    elem.map(|(start, end): (usize, usize)| {
                                        debug_assert!(end - start == 3usize);
                                        if arrow_data_inner.len() < end {
                                            return Err(DeserializationError::offset_slice_oob(
                                                (start, end),
                                                arrow_data_inner.len(),
                                            ));
                                        }

                                        #[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
                                        let data =
                                            unsafe { arrow_data_inner.get_unchecked(start..end) };
                                        let data =
                                            data.iter().cloned().map(Option::unwrap_or_default);

                                        // NOTE: Unwrapping cannot fail: the length must be correct.
                                        #[allow(clippy::unwrap_used)]
                                        Ok(array_init::from_iter(data).unwrap())
                                    })
                                    .transpose()
                                })
                                .map(|res_or_opt| {
                                    res_or_opt
                                        .map(|res_or_opt| res_or_opt.map(crate::datatypes::Vec3D))
                                })
                                .collect::<DeserializationResult<Vec<Option<_>>>>()?
                        }
                        .into_iter()
                    }
                };
                let angle = {
                    if !arrays_by_name.contains_key("angle") {
                        return Err(DeserializationError::missing_struct_field(
                            Self::arrow_datatype(),
                            "angle",
                        ))
                        .with_context("rerun.datatypes.RotationAxisAngle");
                    }
                    let arrow_data = &**arrays_by_name["angle"];
                    arrow_data
                        .as_any()
                        .downcast_ref::<Float32Array>()
                        .ok_or_else(|| {
                            let expected = DataType::Float32;
                            let actual = arrow_data.data_type().clone();
                            DeserializationError::datatype_mismatch(expected, actual)
                        })
                        .with_context("rerun.datatypes.RotationAxisAngle#angle")?
                        .into_iter()
                        .map(|res_or_opt| {
                            res_or_opt.map(|radians| crate::datatypes::Angle { radians })
                        })
                };
                ZipValidity::new_with_validity(::itertools::izip!(axis, angle), arrow_data.nulls())
                    .map(|opt| {
                        opt.map(|(axis, angle)| {
                            Ok(Self {
                                axis: axis
                                    .ok_or_else(DeserializationError::missing_data)
                                    .with_context("rerun.datatypes.RotationAxisAngle#axis")?,
                                angle: angle
                                    .ok_or_else(DeserializationError::missing_data)
                                    .with_context("rerun.datatypes.RotationAxisAngle#angle")?,
                            })
                        })
                        .transpose()
                    })
                    .collect::<DeserializationResult<Vec<_>>>()
                    .with_context("rerun.datatypes.RotationAxisAngle")?
            }
        })
    }
}

impl ::re_byte_size::SizeBytes for RotationAxisAngle {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        self.axis.heap_size_bytes() + self.angle.heap_size_bytes()
    }

    #[inline]
    fn is_pod() -> bool {
        <crate::datatypes::Vec3D>::is_pod() && <crate::datatypes::Angle>::is_pod()
    }
}
