use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use arrow::array::RecordBatchReader;
use arrow::datatypes::Schema;
use arrow::pyarrow::PyArrowType;
use arrow::record_batch::RecordBatchIterator;
use datafusion::catalog::TableProvider;
use datafusion::prelude::SessionContext;
use datafusion_ffi::table_provider::FFI_TableProvider;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::PyAnyMethods as _;
use pyo3::types::{PyCapsule, PyDict, PyTuple};
use pyo3::{Bound, Py, PyAny, PyRef, PyResult, Python, pyclass, pymethods};
use tracing::instrument;

use re_chunk::ComponentIdentifier;
use re_chunk_store::{QueryExpression, SparseFillStrategy, ViewContentsSelector};
use re_dataframe::{QueryCache, QueryEngine};
use re_datafusion::DataframeQueryTableProvider;
use re_log_types::{EntityPath, EntityPathFilter, ResolvedTimeRange};
use re_sdk::ComponentDescriptor;
use re_sorbet::ColumnDescriptor;

use crate::catalog::{PyDatasetEntry, to_py_err};
use crate::utils::{get_tokio_runtime, wait_for_future};

/// View into a remote dataset acting as DataFusion table provider.
#[pyclass(name = "DataframeQueryView")]
pub struct PyDataframeQueryView {
    dataset: Py<PyDatasetEntry>,

    query_expression: QueryExpression,

    /// Limit the query to these partition ids.
    ///
    /// If empty, use the whole dataset.
    partition_ids: Vec<String>,
}

impl PyDataframeQueryView {
    #[instrument(skip(dataset, contents, py))]
    pub fn new(
        dataset: Py<PyDatasetEntry>,
        index: Option<String>,
        contents: Py<PyAny>,
        include_semantically_empty_columns: bool,
        include_tombstone_columns: bool,
        py: Python<'_>,
    ) -> PyResult<Self> {
        // Static only implies:
        // - we include only static columns in the contents
        // - we only return one row per partition, with the static data
        let static_only = index.is_none();

        // We get the schema from the store since we need it to resolve our columns
        // TODO(jleibs): This is way too slow -- maybe we cache it somewhere?
        let schema = PyDatasetEntry::fetch_arrow_schema(&dataset.borrow(py))?;

        // TODO(jleibs): Check schema for the index name

        let view_contents = extract_contents_expr(contents.bind(py), &schema)?;

        Ok(Self {
            dataset,

            query_expression: QueryExpression {
                view_contents: Some(view_contents),
                include_semantically_empty_columns,
                include_tombstone_columns,
                include_static_columns: if static_only {
                    re_chunk_store::StaticColumnSelection::StaticOnly
                } else {
                    re_chunk_store::StaticColumnSelection::Both
                },
                filtered_index: index.map(Into::into),
                filtered_index_range: None,
                filtered_index_values: None,
                using_index_values: None,
                filtered_is_not_null: None,
                sparse_fill_strategy: SparseFillStrategy::None,
                selection: None,
            },
            partition_ids: vec![],
        })
    }

    fn clone_with_new_query(
        &self,
        py: Python<'_>,
        mutation_fn: impl FnOnce(&mut QueryExpression),
    ) -> Self {
        let mut copy = Self {
            dataset: self.dataset.clone_ref(py),
            query_expression: self.query_expression.clone(),
            partition_ids: self.partition_ids.clone(),
        };

        mutation_fn(&mut copy.query_expression);

        copy
    }
}

#[pymethods]
impl PyDataframeQueryView {
    /// Filter by one or more partition ids. All partition ids are included if not specified.
    #[pyo3(signature = (partition_id, *args))]
    fn filter_partition_id<'py>(
        &self,
        py: Python<'py>,
        partition_id: String,
        args: &Bound<'py, PyTuple>,
    ) -> PyResult<Self> {
        let mut partition_ids = vec![partition_id];

        for i in 0..args.len()? {
            let item = args.get_item(i)?;
            partition_ids.push(item.extract()?);
        }

        Ok(Self {
            dataset: self.dataset.clone_ref(py),
            query_expression: self.query_expression.clone(),
            partition_ids,
        })
    }

    #[allow(rustdoc::private_doc_tests)]
    /// Filter the view to only include data between the given index sequence numbers.
    ///
    /// This range is inclusive and will contain both the value at the start and the value at the end.
    ///
    /// The view must be of a sequential index type to use this method.
    ///
    /// Parameters
    /// ----------
    /// start : int
    ///     The inclusive start of the range.
    /// end : int
    ///     The inclusive end of the range.
    ///
    /// Returns
    /// -------
    /// RecordingView
    ///     A new view containing only the data within the specified range.
    ///
    ///     The original view will not be modified.
    fn filter_range_sequence(&self, py: Python<'_>, start: i64, end: i64) -> PyResult<Self> {
        // TODO(emilk): it would be nice to add a check here that the index type is indeed a sequence.
        match self.query_expression.filtered_index.as_ref() {
            Some(_) => {}

            None => {
                return Err(PyValueError::new_err(
                    "Specify an index to filter on first.".to_owned(),
                ));
            }
        }

        let start = if let Ok(seq) = re_chunk::TimeInt::try_from(start) {
            seq
        } else {
            re_log::error!(
                illegal_value = start,
                new_value = re_chunk::TimeInt::MIN.as_i64(),
                "set_time_sequence() called with illegal value - clamped to minimum legal value"
            );
            re_chunk::TimeInt::MIN
        };

        let end = if let Ok(seq) = re_chunk::TimeInt::try_from(end) {
            seq
        } else {
            re_log::error!(
                illegal_value = end,
                new_value = re_chunk::TimeInt::MAX.as_i64(),
                "set_time_sequence() called with illegal value - clamped to maximum legal value"
            );
            re_chunk::TimeInt::MAX
        };

        let resolved = ResolvedTimeRange::new(start, end);

        Ok(self.clone_with_new_query(py, |query_expression| {
            query_expression.filtered_index_range = Some(resolved);
        }))
    }

    #[allow(rustdoc::private_doc_tests)]
    /// Filter the view to only include data between the given index values expressed as seconds.
    ///
    /// This range is inclusive and will contain both the value at the start and the value at the end.
    ///
    /// The view must be of a temporal index type to use this method.
    ///
    /// Parameters
    /// ----------
    /// start : int
    ///     The inclusive start of the range.
    /// end : int
    ///     The inclusive end of the range.
    ///
    /// Returns
    /// -------
    /// RecordingView
    ///     A new view containing only the data within the specified range.
    ///
    ///     The original view will not be modified.
    fn filter_range_secs(&self, py: Python<'_>, start: f64, end: f64) -> PyResult<Self> {
        // TODO(emilk): it would be nice to add a check here that the index type is indeed temporal
        match self.query_expression.filtered_index.as_ref() {
            Some(_) => {}

            None => {
                return Err(PyValueError::new_err(
                    "Specify an index to filter on first.".to_owned(),
                ));
            }
        }

        let start = re_log_types::Timestamp::from_secs_since_epoch(start);
        let end = re_log_types::Timestamp::from_secs_since_epoch(end);

        let resolved = ResolvedTimeRange::new(start, end);

        Ok(self.clone_with_new_query(py, |query_expression| {
            query_expression.filtered_index_range = Some(resolved);
        }))
    }

    #[allow(rustdoc::private_doc_tests)]
    /// Filter the view to only include data between the given index values expressed as nanoseconds.
    ///
    /// This range is inclusive and will contain both the value at the start and the value at the end.
    ///
    /// The view must be of a temporal index type to use this method.
    ///
    /// Parameters
    /// ----------
    /// start : int
    ///     The inclusive start of the range.
    /// end : int
    ///     The inclusive end of the range.
    ///
    /// Returns
    /// -------
    /// RecordingView
    ///     A new view containing only the data within the specified range.
    ///
    ///     The original view will not be modified.
    fn filter_range_nanos(&self, py: Python<'_>, start: i64, end: i64) -> PyResult<Self> {
        // TODO(emilk): it would be nice to add a check here that the index type is indeed a sequence.
        match self.query_expression.filtered_index.as_ref() {
            Some(_) => {}

            None => {
                return Err(PyValueError::new_err(
                    "Specify an index to filter on first.".to_owned(),
                ));
            }
        }

        let start = re_log_types::Timestamp::from_nanos_since_epoch(start);
        let end = re_log_types::Timestamp::from_nanos_since_epoch(end);

        let resolved = ResolvedTimeRange::new(start, end);

        Ok(self.clone_with_new_query(py, |query_expression| {
            query_expression.filtered_index_range = Some(resolved);
        }))
    }

    #[allow(rustdoc::private_doc_tests)]
    /// Filter the view to only include data at the provided index values.
    ///
    /// The index values returned will be the intersection between the provided values and the
    /// original index values.
    ///
    /// This requires index values to be a precise match. Index values in Rerun are
    /// represented as i64 sequence counts or nanoseconds. This API does not expose an interface
    /// in floating point seconds, as the numerical conversion would risk false mismatches.
    ///
    /// Parameters
    /// ----------
    /// values : IndexValuesLike
    ///     The index values to filter by.
    ///
    /// Returns
    /// -------
    /// RecordingView
    ///     A new view containing only the data at the specified index values.
    ///
    ///     The original view will not be modified.
    fn filter_index_values(
        &self,
        py: Python<'_>,
        values: crate::dataframe::IndexValuesLike<'_>,
    ) -> PyResult<Self> {
        let values = values.to_index_values()?;

        Ok(self.clone_with_new_query(py, |query_expression| {
            query_expression.filtered_index_values = Some(values);
        }))
    }

    #[allow(rustdoc::private_doc_tests)]
    /// Filter the view to only include rows where the given component column is not null.
    ///
    /// This corresponds to rows for index values where this component was provided to Rerun explicitly
    /// via `.log()` or `.send_columns()`.
    ///
    /// Parameters
    /// ----------
    /// column : AnyComponentColumn
    ///     The component column to filter by.
    ///
    /// Returns
    /// -------
    /// RecordingView
    ///     A new view containing only the data where the specified component column is not null.
    ///
    ///     The original view will not be modified.
    fn filter_is_not_null(
        &self,
        py: Python<'_>,
        column: crate::dataframe::AnyComponentColumn,
    ) -> PyResult<Self> {
        let column = column.into_selector()?;

        Ok(self.clone_with_new_query(py, |query_expression| {
            query_expression.filtered_is_not_null = Some(column);
        }))
    }

    #[allow(rustdoc::private_doc_tests)]
    /// Replace the index in the view with the provided values.
    ///
    /// The output view will always have the same number of rows as the provided values, even if
    /// those rows are empty. Use with [`.fill_latest_at()`][rerun.dataframe.RecordingView.fill_latest_at]
    /// to populate these rows with the most recent data.
    ///
    /// This requires index values to be a precise match. Index values in Rerun are
    /// represented as i64 sequence counts or nanoseconds. This API does not expose an interface
    /// in floating point seconds, as the numerical conversion would risk false mismatches.
    ///
    /// Parameters
    /// ----------
    /// values : IndexValuesLike
    ///     The index values to use.
    ///
    /// Returns
    /// -------
    /// RecordingView
    ///     A new view containing the provided index values.
    ///
    ///     The original view will not be modified.
    fn using_index_values(
        &self,
        py: Python<'_>,
        values: crate::dataframe::IndexValuesLike<'_>,
    ) -> PyResult<Self> {
        let values = values.to_index_values()?;

        Ok(self.clone_with_new_query(py, |query_expression| {
            query_expression.using_index_values = Some(values);
        }))
    }

    #[allow(rustdoc::private_doc_tests)]
    /// Populate any null values in a row with the latest valid data according to the index.
    ///
    /// Returns
    /// -------
    /// RecordingView
    ///     A new view with the null values filled in.
    ///
    ///     The original view will not be modified.
    fn fill_latest_at(&self, py: Python<'_>) -> Self {
        self.clone_with_new_query(py, |query_expression| {
            query_expression.sparse_fill_strategy = SparseFillStrategy::LatestAtGlobal;
        })
    }

    /// Returns a DataFusion table provider capsule.
    #[instrument(skip_all)]
    fn __datafusion_table_provider__<'py>(
        self_: PyRef<'py, Self>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyCapsule>> {
        let provider = self_.as_table_provider(py)?;

        let capsule_name = cr"datafusion_table_provider".into();

        let runtime = get_tokio_runtime().handle().clone();
        let provider = FFI_TableProvider::new(provider, false, Some(runtime));

        PyCapsule::new(py, provider, Some(capsule_name))
    }

    /// Convert this view to a [`pyarrow.RecordBatchReader`][].
    #[instrument(skip_all)]
    fn to_arrow_reader<'py>(
        self_: PyRef<'py, Self>,
        py: Python<'py>,
    ) -> PyResult<PyArrowType<Box<dyn RecordBatchReader + Send>>> {
        let table_provider = self_.as_table_provider(py)?;
        let schema = table_provider.schema();

        let session_context = SessionContext::new();
        session_context
            .register_table("__table__", table_provider)
            .map_err(to_py_err)?;

        let record_batches = wait_for_future(py, async move {
            session_context
                .table("__table__")
                .await
                .map_err(to_py_err)?
                .collect()
                .await
                .map_err(to_py_err)
        })?;

        let reader = RecordBatchIterator::new(record_batches.into_iter().map(Result::Ok), schema);

        Ok(PyArrowType(Box::new(reader)))
    }

    /// Register this view to the global DataFusion context and return a DataFrame.
    #[instrument(skip_all)]
    fn df(self_: PyRef<'_, Self>) -> PyResult<Bound<'_, PyAny>> {
        let py = self_.py();

        let dataset = self_.dataset.borrow(py);
        let super_ = dataset.as_super();
        let client = super_.client.borrow(py);
        let ctx = client.ctx(py)?;
        let ctx = ctx.bind(py);

        let uuid = uuid::Uuid::new_v4().simple();
        let name = format!("{}_dataframe_query_{uuid}", super_.name());

        drop(client);
        drop(dataset);

        // We're fine with this failing.
        ctx.call_method1("deregister_table", (name.clone(),))?;

        ctx.call_method1("register_table_provider", (name.clone(), self_))?;

        let df = ctx.call_method1("table", (name,))?;

        Ok(df)
    }

    /// Get the relevant chunk_ids for this view.
    #[instrument(skip_all)]
    fn get_chunk_ids<'py>(
        self_: PyRef<'py, Self>,
        py: Python<'py>,
    ) -> PyResult<PyArrowType<Box<dyn RecordBatchReader + Send>>> {
        let dataset = self_.dataset.borrow(py);
        let entry = dataset.as_super();
        let dataset_id = entry.details.id;
        let connection = entry.client.borrow(py).connection().clone();

        // Fetch relevant chunks
        connection.get_chunk_ids_for_dataframe_query(
            py,
            dataset_id,
            &self_.query_expression,
            self_.partition_ids.as_slice(),
        )
    }
}

impl PyDataframeQueryView {
    fn as_table_provider(&self, py: Python<'_>) -> PyResult<Arc<dyn TableProvider>> {
        let dataset = self.dataset.borrow(py);
        let entry = dataset.as_super();
        let dataset_id = entry.details.id;
        let connection = entry.client.borrow(py).connection().clone();

        //
        // Fetch relevant chunks
        //

        let chunk_stores = connection.get_chunks_for_dataframe_query(
            py,
            dataset_id,
            &self.query_expression,
            self.partition_ids.as_slice(),
        )?;

        let query_engines = chunk_stores
            .into_iter()
            .map(|(partition_id, store_handle)| {
                let query_engine = QueryEngine::new(
                    store_handle.clone(),
                    QueryCache::new_handle(store_handle.clone()),
                );

                (partition_id, query_engine)
            })
            .collect();

        DataframeQueryTableProvider::new(query_engines, &self.query_expression)
            .map_err(to_py_err)?
            .try_into()
            .map_err(to_py_err)
    }
}

/// Convert a `ViewContentsLike` into a `ViewContentsSelector`.
///
/// ```python
/// ViewContentsLike = Union[str, Dict[str, Union[str, Sequence[str]]]]
/// ```
///
/// We cant do this with the normal `FromPyObject` mechanisms because we want access to the
/// `QueryEngine` to resolve the entity paths.
fn extract_contents_expr(
    expr: &Bound<'_, PyAny>,
    schema: &Schema,
) -> PyResult<re_chunk_store::ViewContentsSelector> {
    let descriptors = schema
        .fields()
        .iter()
        .map(|field| ColumnDescriptor::try_from_arrow_field(None, field.as_ref()))
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    let component_descriptors = descriptors
        .iter()
        .filter_map(|descriptor| {
            if let ColumnDescriptor::Component(component) = descriptor {
                Some(component)
            } else {
                None
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut known_components = BTreeMap::<EntityPath, BTreeSet<ComponentDescriptor>>::new();

    for component in &component_descriptors {
        // We need to resolve the component type to the best one in the schema
        // (e.g. "color" -> "rerun.color")
        known_components
            .entry(component.entity_path.clone())
            .or_default()
            .insert(component.into());
    }

    if let Ok(expr) = expr.extract::<String>() {
        // `str`

        let path_filter =
                EntityPathFilter::parse_strict(&expr)
                    .map_err(|err| {
                        PyValueError::new_err(format!(
                            "Could not interpret `contents` as a ViewContentsLike. Failed to parse {expr}: {err}.",
                        ))
                    })?.resolve_without_substitutions();

        // Iterate every entity path in the schema

        let contents = known_components
            .keys()
            .filter(|p| path_filter.matches(p))
            .map(|p| (p.clone(), None))
            .collect();

        Ok(contents)
    } else if let Ok(dict) = expr.downcast::<PyDict>() {
        // `Union[str, Sequence[str]]]`

        let mut contents = ViewContentsSelector::default();

        for (key, value) in dict {
            let key = key.extract::<String>().map_err(|_err| {
                    PyTypeError::new_err(
                        format!("Could not interpret `contents` as a ViewContentsLike. Key: {key} is not a path expression."),
                    )
                })?;

            let path_filter = EntityPathFilter::parse_strict(&key).map_err(|err| {
                    PyValueError::new_err(format!(
                        "Could not interpret `contents` as a ViewContentsLike. Failed to parse {key}: {err}.",
                    ))
                })?.resolve_without_substitutions();

            let component_strs: BTreeSet<ComponentIdentifier> = if let Ok(component) =
                value.extract::<String>()
            {
                std::iter::once(component.into()).collect()
            } else if let Ok(components) = value.extract::<Vec<String>>() {
                components.into_iter().map(Into::into).collect()
            } else {
                return Err(PyTypeError::new_err(format!(
                    "Could not interpret `contents` as a ViewContentsLike. Value: {value} is not a `str` or Sequence[str]."
                )));
            };

            let mut key_contents = known_components
                .keys()
                .filter(|p| path_filter.matches(p))
                .map(|entity_path| (entity_path.clone(), Some(component_strs.clone())))
                .collect();

            contents.append(&mut key_contents);
        }

        Ok(contents)
    } else {
        return Err(PyTypeError::new_err(
            "Could not interpret `contents` as a ViewContentsLike. Top-level type must be a string or a dictionary.",
        ));
    }
}
