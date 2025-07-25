use egui::{NumExt as _, emath::RectTransform};
use glam::{Affine3A, Quat, Vec3};
use web_time::Instant;

use macaw::BoundingBox;
use re_log_types::EntityPath;
use re_renderer::{
    LineDrawableBuilder, Size,
    view_builder::{Projection, TargetConfiguration, ViewBuilder},
};
use re_types::{
    blueprint::{
        archetypes::{Background, LineGrid3D},
        components::GridSpacing,
    },
    components::{ViewCoordinates, Visible},
    view_coordinates::SignedAxis3,
};
use re_ui::{ContextExt as _, Help, IconText, MouseButtonText, UiExt as _, icons};
use re_view::controls::{
    DRAG_PAN3D_BUTTON, ROLL_MOUSE_ALT, ROLL_MOUSE_MODIFIER, ROTATE3D_BUTTON, RuntimeModifiers,
    SPEED_UP_3D_MODIFIER, TRACKED_OBJECT_RESTORE_KEY,
};
use re_viewer_context::{
    Item, ItemContext, ViewClassExt as _, ViewContext, ViewQuery, ViewSystemExecutionError,
    ViewerContext, gpu_bridge,
};
use re_viewport_blueprint::ViewProperty;

use crate::{
    SpatialView3D,
    scene_bounding_boxes::SceneBoundingBoxes,
    space_camera_3d::SpaceCamera3D,
    transform_cache::query_view_coordinates_at_closest_ancestor,
    ui::{SpatialViewState, create_labels},
    view_kind::SpatialViewKind,
    visualizers::{CamerasVisualizer, collect_ui_labels, image_view_coordinates},
};

use super::eye::{Eye, ViewEye};

// ---

#[derive(Clone)]
pub struct View3DState {
    pub view_eye: Option<ViewEye>,

    /// Used to show the orbit center of the eye-camera when the user interacts.
    /// None: user has never interacted with the eye-camera.
    pub last_eye_interaction: Option<Instant>,

    /// Currently tracked entity.
    ///
    /// If this is a camera, it takes over the camera pose, otherwise follows the entity.
    pub tracked_entity: Option<EntityPath>,

    /// Eye pose just before we started following an entity [`Self::tracked_entity`].
    camera_before_tracked_entity: Option<Eye>,

    eye_interpolation: Option<EyeInterpolation>,

    /// Last known view coordinates.
    /// Used to detect changes in view coordinates, in which case we reset the camera eye.
    pub scene_view_coordinates: Option<ViewCoordinates>,

    // options:
    spin: bool,
    pub show_axes: bool,
    pub show_bbox: bool,
    pub show_smoothed_bbox: bool,

    eye_interact_fade_in: bool,
    eye_interact_fade_change_time: f64,
}

impl Default for View3DState {
    fn default() -> Self {
        Self {
            view_eye: Default::default(),
            last_eye_interaction: None,
            tracked_entity: None,
            camera_before_tracked_entity: None,
            eye_interpolation: Default::default(),
            scene_view_coordinates: None,
            spin: false,
            show_axes: false,
            show_bbox: false,
            show_smoothed_bbox: false,
            eye_interact_fade_in: false,
            eye_interact_fade_change_time: f64::NEG_INFINITY,
        }
    }
}

fn ease_out(t: f32) -> f32 {
    1. - (1. - t) * (1. - t)
}

impl View3DState {
    pub fn reset_camera(
        &mut self,
        scene_bbox: &SceneBoundingBoxes,
        scene_view_coordinates: Option<ViewCoordinates>,
    ) {
        // Mark as interaction since we want to stop doing any automatic interpolations,
        // even if this is caused by a full reset.
        self.last_eye_interaction = Some(Instant::now());
        self.interpolate_to_view_eye(default_eye(&scene_bbox.current, scene_view_coordinates));
        self.tracked_entity = None;
        self.camera_before_tracked_entity = None;
    }

    fn update_eye(
        &mut self,
        response: &egui::Response,
        bounding_boxes: &SceneBoundingBoxes,
        space_cameras: &[SpaceCamera3D],
        scene_view_coordinates: Option<ViewCoordinates>,
    ) -> ViewEye {
        // If the user has not interacted with the eye-camera yet, continue to
        // interpolate to the new default eye. This gives much better robustness
        // with scenes that change over time.
        if self.last_eye_interaction.is_none() {
            self.interpolate_to_view_eye(default_eye(
                &bounding_boxes.current,
                scene_view_coordinates,
            ));
        }

        // Detect live changes to view coordinates, and interpolate to the new up axis as needed.
        if scene_view_coordinates != self.scene_view_coordinates {
            self.interpolate_to_view_eye(default_eye(
                &bounding_boxes.current,
                scene_view_coordinates,
            ));
        }
        self.scene_view_coordinates = scene_view_coordinates;

        // Follow tracked object.
        if let Some(tracked_entity) = self.tracked_entity.clone() {
            if let Some(target_eye) = find_camera(space_cameras, &tracked_entity) {
                // For cameras, we want to exactly track the camera pose once we're done interpolating.
                if let Some(eye_interpolation) = &mut self.eye_interpolation {
                    eye_interpolation.target_view_eye = None;
                    eye_interpolation.target_eye = Some(target_eye);
                } else if let Some(view_eye) = &mut self.view_eye {
                    view_eye.copy_from_eye(&target_eye);
                }
            } else {
                // For other entities we keep interpolating, so when the entity jumps, we follow smoothly.
                self.interpolate_eye_to_entity(&tracked_entity, bounding_boxes, space_cameras);
            }
        }

        let view_eye = self
            .view_eye
            .get_or_insert_with(|| default_eye(&bounding_boxes.current, scene_view_coordinates));

        if self.spin {
            view_eye.rotate(egui::vec2(
                -response.ctx.input(|i| i.stable_dt).at_most(0.1) * 150.0,
                0.0,
            ));
            response.ctx.request_repaint();
        }

        if let Some(cam_interpolation) = &mut self.eye_interpolation {
            cam_interpolation.elapsed_time += response.ctx.input(|i| i.stable_dt).at_most(0.1);

            let t = cam_interpolation.elapsed_time / cam_interpolation.target_time;
            let t = t.clamp(0.0, 1.0);
            let t = ease_out(t);

            if let Some(target_orbit) = &cam_interpolation.target_view_eye {
                *view_eye = cam_interpolation.start.lerp(target_orbit, t);
            } else if let Some(target_eye) = &cam_interpolation.target_eye {
                let camera = cam_interpolation.start.to_eye().lerp(target_eye, t);
                view_eye.copy_from_eye(&camera);
            } else {
                self.eye_interpolation = None;
            }

            if t < 1.0 {
                // There's more frames to render to finish interpolation.
                response.ctx.request_repaint();
            } else {
                // We have arrived at our target
                self.eye_interpolation = None;
            }
        }

        // If we're tracking a camera right now, we want to make it slightly sticky,
        // so that a click on some entity doesn't immediately break the tracked state.
        // (Threshold is in amount of ui points the mouse was moved.)
        let view_eye_drag_threshold = if self.tracked_entity.is_some() {
            4.0
        } else {
            0.0
        };

        if view_eye.update(response, view_eye_drag_threshold, bounding_boxes) {
            self.last_eye_interaction = Some(Instant::now());
            self.eye_interpolation = None;
            self.tracked_entity = None;
            self.camera_before_tracked_entity = None;
        }

        *view_eye
    }

    fn interpolate_to_eye(&mut self, target: Eye) {
        if let Some(start) = self.view_eye.as_mut() {
            // the user wants to move the camera somewhere, so stop spinning
            self.spin = false;

            if let Some(target_time) = EyeInterpolation::target_time(&start.to_eye(), &target) {
                self.eye_interpolation = Some(EyeInterpolation {
                    elapsed_time: 0.0,
                    target_time,
                    start: *start,
                    target_view_eye: None,
                    target_eye: Some(target),
                });
            } else {
                start.copy_from_eye(&target);
            }
        } else {
            // shouldn't really happen (`self.view_eye` is only `None` for the first frame).
        }
    }

    fn interpolate_eye_to_entity(
        &mut self,
        entity_path: &EntityPath,
        bounding_boxes: &SceneBoundingBoxes,
        space_cameras: &[SpaceCamera3D],
    ) {
        // Note that we may want to focus on an _instance_ instead in the future:
        // The problem with that is that there may be **many** instances (think point cloud)
        // and they may not be consistent over time.
        // -> we don't know the bounding box of every instance (right now)
        // -> tracking instances over time may not be desired
        //    (this can happen with entities as well, but is less likely).
        //
        // For future reference, it's also worth pointing out that for interactions in the view we
        // already have the 3D position:
        // if let Some(SelectedSpaceContext::ThreeD {
        //     pos: Some(clicked_point),
        //     ..
        // }) = ctx.selection_state().hovered_space_context()

        if let Some(tracked_camera) = find_camera(space_cameras, entity_path) {
            self.interpolate_to_eye(tracked_camera);
        } else if let Some(entity_bbox) = bounding_boxes.per_entity.get(&entity_path.hash()) {
            let Some(mut new_view_eye) = self.view_eye else {
                // Happens only the first frame when there's no eye set up yet.
                return;
            };

            let radius = entity_bbox.centered_bounding_sphere_radius() * 1.5;
            let orbit_radius = if radius < 0.0001 {
                // Handle zero-sized bounding boxes:
                (bounding_boxes.current.centered_bounding_sphere_radius() * 1.5).at_least(0.01)
            } else {
                radius
            };

            new_view_eye.set_orbit_center_and_radius(entity_bbox.center(), orbit_radius);

            self.interpolate_to_view_eye(new_view_eye);
        }
    }

    /// The taregt mode will be ignored, and the mode of the current eye will be kept unchanged.
    fn interpolate_to_view_eye(&mut self, mut target: ViewEye) {
        if let Some(view_eye) = &self.view_eye {
            target.set_mode(view_eye.mode());
        }

        // the user wants to move the camera somewhere, so stop spinning
        self.spin = false;

        if self.view_eye == Some(target) {
            return; // We're already there.
        }

        // Don't restart interpolation if we're already on it.
        if let Some(eye_interpolation) = &self.eye_interpolation {
            if eye_interpolation.target_view_eye == Some(target) {
                return;
            }
        }

        if let Some(start) = self.view_eye {
            if let Some(target_time) =
                EyeInterpolation::target_time(&start.to_eye(), &target.to_eye())
            {
                self.eye_interpolation = Some(EyeInterpolation {
                    elapsed_time: 0.0,
                    target_time,
                    start,
                    target_view_eye: Some(target),
                    target_eye: None,
                });
            } else {
                self.view_eye = Some(target);
            }
        } else {
            self.view_eye = Some(target);
        }
    }

    fn track_entity(
        &mut self,
        entity_path: &EntityPath,
        bounding_boxes: &SceneBoundingBoxes,
        space_cameras: &[SpaceCamera3D],
    ) {
        if self.tracked_entity == Some(entity_path.clone()) {
            return; // already tracking this entity.
        }

        re_log::debug!("3D view tracks now {:?}", entity_path);
        self.tracked_entity = Some(entity_path.clone());
        self.camera_before_tracked_entity = self.view_eye.map(|eye| eye.to_eye());

        self.interpolate_eye_to_entity(entity_path, bounding_boxes, space_cameras);
    }

    pub fn spin(&self) -> bool {
        self.spin
    }

    pub fn set_spin(&mut self, spin: bool) {
        if spin != self.spin {
            self.spin = spin;
            self.last_eye_interaction = Some(Instant::now());
        }
    }
}

#[derive(Clone, PartialEq)]
struct EyeInterpolation {
    elapsed_time: f32,
    target_time: f32,
    start: ViewEye,
    target_view_eye: Option<ViewEye>,
    target_eye: Option<Eye>,
}

impl EyeInterpolation {
    pub fn target_time(start: &Eye, stop: &Eye) -> Option<f32> {
        // Take more time if the rotation is big:
        let angle_difference = start
            .world_from_rub_view
            .rotation()
            .angle_between(stop.world_from_rub_view.rotation());

        // Threshold to avoid doing pointless interpolations that trigger frame requests.
        let distance = start.pos_in_world().distance(stop.pos_in_world());
        if angle_difference < 0.01 && distance < 0.0001 {
            None
        } else {
            Some(egui::remap_clamp(
                angle_difference,
                0.0..=std::f32::consts::PI,
                0.2..=0.7,
            ))
        }
    }
}

fn find_camera(space_cameras: &[SpaceCamera3D], needle: &EntityPath) -> Option<Eye> {
    let mut found_camera = None;

    for camera in space_cameras {
        if &camera.ent_path == needle {
            if found_camera.is_some() {
                return None; // More than one camera
            } else {
                found_camera = Some(camera);
            }
        }
    }

    found_camera.and_then(Eye::from_camera)
}

// ----------------------------------------------------------------------------

pub fn help(os: egui::os::OperatingSystem) -> Help {
    Help::new("3D view")
        .docs_link("https://rerun.io/docs/reference/types/views/spatial3d_view")
        .control("Pan", (MouseButtonText(DRAG_PAN3D_BUTTON), "+", "drag"))
        .control("Zoom", icons::SCROLL)
        .control("Rotate", (MouseButtonText(ROTATE3D_BUTTON), "+", "drag"))
        .control(
            "Roll",
            IconText::from_modifiers_and(os, ROLL_MOUSE_MODIFIER, MouseButtonText(ROLL_MOUSE_ALT)),
        )
        .control("Navigate", ("WASD", "/", "QE"))
        .control(
            "Slow down / speed up",
            (
                IconText::from_modifiers(os, RuntimeModifiers::slow_down(&os)),
                "/",
                IconText::from_modifiers(os, SPEED_UP_3D_MODIFIER),
            ),
        )
        .control("Focus", ("double", icons::LEFT_MOUSE_CLICK, "object"))
        .control(
            "Reset view",
            ("double", icons::LEFT_MOUSE_CLICK, "background"),
        )
}

impl SpatialView3D {
    pub fn view_3d(
        &self,
        ctx: &ViewerContext<'_>,
        ui: &mut egui::Ui,
        state: &mut SpatialViewState,
        query: &ViewQuery<'_>,
        system_output: re_viewer_context::SystemExecutionOutput,
    ) -> Result<(), ViewSystemExecutionError> {
        re_tracing::profile_function!();

        let highlights = &query.highlights;
        let space_cameras = &system_output
            .view_systems
            .get::<CamerasVisualizer>()?
            .space_cameras;
        let scene_view_coordinates = query_view_coordinates_at_closest_ancestor(
            query.space_origin,
            ctx.recording(),
            &ctx.current_query(),
        );

        let (ui_rect, mut response) =
            ui.allocate_at_least(ui.available_size(), egui::Sense::click_and_drag());

        if !ui_rect.is_positive() {
            return Ok(()); // protect against problems with zero-sized views
        }

        let view_eye = state.state_3d.update_eye(
            &response,
            &state.bounding_boxes,
            space_cameras,
            scene_view_coordinates,
        );
        let eye = view_eye.to_eye();

        // Determine view port resolution and position.
        let resolution_in_pixel =
            gpu_bridge::viewport_resolution_in_pixels(ui_rect, ui.ctx().pixels_per_point());
        if resolution_in_pixel[0] == 0 || resolution_in_pixel[1] == 0 {
            return Ok(());
        }

        let target_config = TargetConfiguration {
            name: query.space_origin.to_string().into(),

            resolution_in_pixel,

            view_from_world: eye.world_from_rub_view.inverse(),
            projection_from_view: Projection::Perspective {
                vertical_fov: eye.fov_y.unwrap_or(Eye::DEFAULT_FOV_Y),
                near_plane_distance: eye.near(),
                aspect_ratio: resolution_in_pixel[0] as f32 / resolution_in_pixel[1] as f32,
            },
            viewport_transformation: re_renderer::RectTransform::IDENTITY,

            pixels_per_point: ui.ctx().pixels_per_point(),

            outline_config: query
                .highlights
                .any_outlines()
                .then(|| re_view::outline_config(ui.ctx())),
            blend_with_background: false,
        };

        // Various ui interactions draw additional lines.
        let mut line_builder = LineDrawableBuilder::new(ctx.render_ctx());
        line_builder.radius_boost_in_ui_points_for_outlines(
            re_view::SIZE_BOOST_IN_POINTS_FOR_LINE_OUTLINES,
        );
        // We don't know ahead of time how many lines we need, but it's not gonna be a huge amount!
        line_builder.reserve_strips(32)?;
        line_builder.reserve_vertices(64)?;

        // Origin gizmo if requested.
        // TODO(andreas): Move this to the transform3d_arrow scene part.
        //              As of #2522 state is now longer accessible there, move the property to a context?
        if state.state_3d.show_axes {
            let axis_length = 1.0; // The axes are also a measuring stick
            crate::visualizers::add_axis_arrows(
                ctx.tokens(),
                &mut line_builder,
                glam::Affine3A::IDENTITY,
                None,
                axis_length,
                re_renderer::OutlineMaskPreference::NONE,
            );

            // If we are showing the axes for the space, then add the space origin to the bounding box.
            state.bounding_boxes.current.extend(glam::Vec3::ZERO);
        }

        let mut view_builder = ViewBuilder::new(ctx.render_ctx(), target_config);

        // Create labels now since their shapes participate are added to scene.ui for picking.
        let (label_shapes, ui_rects) = create_labels(
            collect_ui_labels(&system_output.view_systems),
            RectTransform::from_to(ui_rect, ui_rect),
            &eye,
            ui,
            highlights,
            SpatialViewKind::ThreeD,
        );

        if let Some(pointer_pos_ui) = response.hover_pos() {
            // There's no panning & zooming, so this is an identity transform.
            let ui_pan_and_zoom_from_ui = RectTransform::from_to(ui_rect, ui_rect);

            let picking_context = crate::picking::PickingContext::new(
                pointer_pos_ui,
                ui_pan_and_zoom_from_ui,
                ui.ctx().pixels_per_point(),
                &eye,
            );
            response = crate::picking_ui::picking(
                ctx,
                &picking_context,
                ui,
                response,
                &mut view_builder,
                state,
                &system_output,
                &ui_rects,
                query,
                SpatialViewKind::ThreeD,
            )?;
        } else {
            state.previous_picking_result = None;
        }

        // Track focused entity if any.
        if let Some(focused_item) = ctx.focused_item {
            let focused_entity = match focused_item {
                Item::AppId(_)
                | Item::DataSource(_)
                | Item::StoreId(_)
                | Item::Container(_)
                | Item::RedapEntry(_)
                | Item::RedapServer(_)
                | Item::TableId(_) => None,

                Item::View(view_id) => {
                    if view_id == &query.view_id {
                        state
                            .state_3d
                            .reset_camera(&state.bounding_boxes, scene_view_coordinates);
                    }
                    None
                }

                Item::ComponentPath(component_path) => Some(&component_path.entity_path),

                Item::InstancePath(instance_path) => Some(&instance_path.entity_path),

                Item::DataResult(view_id, instance_path) => {
                    if *view_id == query.view_id {
                        Some(&instance_path.entity_path)
                    } else {
                        None
                    }
                }
            };
            if let Some(entity_path) = focused_entity {
                state.state_3d.last_eye_interaction = Some(Instant::now());

                // TODO(#4812): We currently only track cameras on double click since tracking arbitrary entities was deemed too surprising.
                if find_camera(space_cameras, entity_path).is_some() {
                    state
                        .state_3d
                        .track_entity(entity_path, &state.bounding_boxes, space_cameras);
                } else {
                    state.state_3d.interpolate_eye_to_entity(
                        entity_path,
                        &state.bounding_boxes,
                        space_cameras,
                    );
                }
            }

            // Make sure focus consequences happen in the next frames.
            ui.ctx().request_repaint();
        }

        // Allow to restore the camera state with escape if a camera was tracked before.
        if response.hovered() && ui.input(|i| i.key_pressed(TRACKED_OBJECT_RESTORE_KEY)) {
            if let Some(camera_before_tracked_entity) = state.state_3d.camera_before_tracked_entity
            {
                state
                    .state_3d
                    .interpolate_to_eye(camera_before_tracked_entity);
                state.state_3d.camera_before_tracked_entity = None;
                state.state_3d.tracked_entity = None;
            }
        }

        for selected_context in ctx.selection_state().selection_item_contexts() {
            show_projections_from_2d_space(
                &mut line_builder,
                space_cameras,
                state,
                selected_context,
                ui.ctx().selection_stroke().color,
            );
        }
        if let Some(hovered_context) = ctx.selection_state().hovered_item_context() {
            show_projections_from_2d_space(
                &mut line_builder,
                space_cameras,
                state,
                hovered_context,
                ui.ctx().hover_stroke().color,
            );
        }

        // TODO(andreas): Make configurable. Could pick up default radius for this view?
        let box_line_radius = Size(*re_types::components::Radius::default().0);

        if state.state_3d.show_bbox {
            line_builder
                .batch("scene_bbox_current")
                .add_box_outline(&state.bounding_boxes.current)
                .map(|lines| {
                    lines
                        .radius(box_line_radius)
                        .color(ui.tokens().frustum_color)
                });
        }
        if state.state_3d.show_smoothed_bbox {
            line_builder
                .batch("scene_bbox_smoothed")
                .add_box_outline(&state.bounding_boxes.smoothed)
                .map(|lines| {
                    lines
                        .radius(box_line_radius)
                        .color(ctx.tokens().frustum_color)
                });
        }

        show_orbit_eye_center(
            ui.ctx(),
            &mut state.state_3d,
            &mut line_builder,
            &view_eye,
            scene_view_coordinates,
        );

        for draw_data in system_output.draw_data {
            view_builder.queue_draw(draw_data);
        }

        let view_ctx = self.view_context(ctx, query.view_id, state);

        // Optional 3D line grid.
        let grid_config = ViewProperty::from_archetype::<LineGrid3D>(
            ctx.blueprint_db(),
            ctx.blueprint_query,
            query.view_id,
        );
        if let Some(draw_data) = self.setup_grid_3d(&view_ctx, &grid_config)? {
            view_builder.queue_draw(draw_data);
        }

        // Commit ui induced lines.
        view_builder.queue_draw(line_builder.into_draw_data()?);

        let background = ViewProperty::from_archetype::<Background>(
            ctx.blueprint_db(),
            ctx.blueprint_query,
            query.view_id,
        );
        let (background_drawable, clear_color) =
            crate::configure_background(&view_ctx, &background, self)?;
        if let Some(background_drawable) = background_drawable {
            view_builder.queue_draw(background_drawable);
        }

        ui.painter().add(gpu_bridge::new_renderer_callback(
            view_builder,
            ui_rect,
            clear_color,
        ));

        // Add egui-rendered spinners/loaders on top of re_renderer content:
        crate::ui::paint_loading_spinners(
            ui,
            RectTransform::from_to(ui_rect, ui_rect),
            &eye,
            &system_output.view_systems,
        );

        // Add egui-rendered labels on top of everything else:
        let painter = ui.painter().with_clip_rect(ui.max_rect());
        painter.extend(label_shapes);

        Ok(())
    }

    fn setup_grid_3d(
        &self,
        ctx: &ViewContext<'_>,
        grid_config: &ViewProperty,
    ) -> Result<Option<re_renderer::renderer::WorldGridDrawData>, ViewSystemExecutionError> {
        if !**grid_config.component_or_fallback::<Visible>(
            ctx,
            self,
            &LineGrid3D::descriptor_visible(),
        )? {
            return Ok(None);
        }

        let spacing = **grid_config.component_or_fallback::<GridSpacing>(
            ctx,
            self,
            &LineGrid3D::descriptor_spacing(),
        )?;
        let thickness_ui = **grid_config
            .component_or_fallback::<re_types::components::StrokeWidth>(
                ctx,
                self,
                &LineGrid3D::descriptor_stroke_width(),
            )?;
        let color = grid_config.component_or_fallback::<re_types::components::Color>(
            ctx,
            self,
            &LineGrid3D::descriptor_color(),
        )?;
        let plane = grid_config.component_or_fallback::<re_types::components::Plane3D>(
            ctx,
            self,
            &LineGrid3D::descriptor_plane(),
        )?;

        Ok(Some(re_renderer::renderer::WorldGridDrawData::new(
            ctx.render_ctx(),
            &re_renderer::renderer::WorldGridConfiguration {
                color: color.into(),
                plane: plane.into(),
                spacing,
                thickness_ui,
            },
        )))
    }
}

/// Show center of orbit camera when interacting with camera (it's quite helpful).
fn show_orbit_eye_center(
    egui_ctx: &egui::Context,
    state_3d: &mut View3DState,
    line_builder: &mut LineDrawableBuilder<'_>,
    view_eye: &ViewEye,
    scene_view_coordinates: Option<ViewCoordinates>,
) {
    let Some(orbit_center) = view_eye.orbit_center() else {
        return;
    };
    let Some(orbit_radius) = view_eye.orbit_radius() else {
        return;
    };

    const FADE_DURATION: f32 = 0.1;

    let ui_time = egui_ctx.input(|i| i.time);
    let any_mouse_button_down = egui_ctx.input(|i| i.pointer.any_down());

    let should_show_center_of_orbit_camera = state_3d
        .last_eye_interaction
        .is_some_and(|d| d.elapsed().as_secs_f32() < 0.35);

    if !state_3d.eye_interact_fade_in && should_show_center_of_orbit_camera {
        // Any interaction immediately causes fade in to start if it's not already on.
        state_3d.eye_interact_fade_change_time = ui_time;
        state_3d.eye_interact_fade_in = true;
    }
    if state_3d.eye_interact_fade_in
            && !should_show_center_of_orbit_camera
            // Don't start fade-out while dragging, even if mouse is still
            && !any_mouse_button_down
    {
        state_3d.eye_interact_fade_change_time = ui_time;
        state_3d.eye_interact_fade_in = false;
    }

    pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = f32::clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
        t * t * (3.0 - t * 2.0)
    }

    // Compute smooth fade.
    let time_since_fade_change = (ui_time - state_3d.eye_interact_fade_change_time) as f32;
    let orbit_center_fade = if state_3d.eye_interact_fade_in {
        // Fade in.
        smoothstep(0.0, FADE_DURATION, time_since_fade_change)
    } else {
        // Fade out.
        smoothstep(FADE_DURATION, 0.0, time_since_fade_change)
    };

    if orbit_center_fade > 0.001 {
        let half_line_length = orbit_radius * 0.03;
        let half_line_length = half_line_length * orbit_center_fade;

        // We distinguish the eye up-axis from the other two axes:
        // Default to RFU
        let up = view_eye.eye_up().unwrap_or(glam::Vec3::Z);

        // For the other two axes, try to use the scene view coordinates if available:
        let right = scene_view_coordinates
            .and_then(|vc| vc.right())
            .map_or(glam::Vec3::X, Vec3::from);
        let forward = up
            .cross(right)
            .try_normalize()
            .unwrap_or_else(|| up.any_orthogonal_vector());
        let right = forward.cross(up);

        line_builder
            .batch("center orbit orientation help")
            .add_segments(
                [
                    (orbit_center, orbit_center + 0.5 * up * half_line_length),
                    (
                        orbit_center - right * half_line_length,
                        orbit_center + right * half_line_length,
                    ),
                    (
                        orbit_center - forward * half_line_length,
                        orbit_center + forward * half_line_length,
                    ),
                ]
                .into_iter(),
            )
            .radius(Size::new_ui_points(0.75))
            // TODO(andreas): Fade this out.
            .color(egui_ctx.tokens().frustum_color);

        // TODO(andreas): Idea for nice depth perception:
        // Render the lines once with additive blending and depth test enabled
        // and another time without depth test. In both cases it needs to be rendered last,
        // something re_renderer doesn't support yet for primitives within renderers.

        egui_ctx.request_repaint(); // show it for a bit longer.
    }
}

fn show_projections_from_2d_space(
    line_builder: &mut re_renderer::LineDrawableBuilder<'_>,
    space_cameras: &[SpaceCamera3D],
    state: &SpatialViewState,
    item_context: &ItemContext,
    ray_color: egui::Color32,
) {
    match item_context {
        ItemContext::TwoD { space_2d, pos } => {
            if let Some(cam) = space_cameras.iter().find(|cam| &cam.ent_path == space_2d) {
                if let Some(pinhole) = cam.pinhole.as_ref() {
                    // Render a thick line to the actual z value if any and a weaker one as an extension
                    // If we don't have a z value, we only render the thick one.
                    let depth = if 0.0 < pos.z && pos.z.is_finite() {
                        pos.z
                    } else {
                        cam.picture_plane_distance
                    };
                    let stop_in_image_plane = pinhole.unproject(glam::vec3(pos.x, pos.y, depth));

                    let world_from_image = glam::Affine3A::from(cam.world_from_camera)
                        * glam::Affine3A::from_mat3(
                            cam.pinhole_view_coordinates
                                .from_other(&image_view_coordinates()),
                        );
                    let stop_in_world = world_from_image.transform_point3(stop_in_image_plane);

                    let origin = cam.position();

                    if let Some(dir) = (stop_in_world - origin).try_normalize() {
                        let ray = macaw::Ray3::from_origin_dir(origin, dir);

                        let thick_ray_length = (stop_in_world - origin).length();
                        add_picking_ray(
                            line_builder,
                            ray,
                            &state.bounding_boxes.smoothed,
                            thick_ray_length,
                            ray_color,
                        );
                    }
                }
            }
        }
        ItemContext::ThreeD {
            pos: Some(pos),
            tracked_entity: Some(tracked_entity),
            ..
        } => {
            let current_tracked_entity = state.state_3d.tracked_entity.as_ref();
            if current_tracked_entity != Some(tracked_entity) {
                if let Some(tracked_camera) = space_cameras
                    .iter()
                    .find(|cam| &cam.ent_path == tracked_entity)
                {
                    let cam_to_pos = *pos - tracked_camera.position();
                    let distance = cam_to_pos.length();
                    let ray = macaw::Ray3::from_origin_dir(
                        tracked_camera.position(),
                        cam_to_pos / distance,
                    );
                    add_picking_ray(
                        line_builder,
                        ray,
                        &state.bounding_boxes.current,
                        distance,
                        ray_color,
                    );
                }
            }
        }
        ItemContext::ThreeD { .. }
        | ItemContext::StreamsTree { .. }
        | ItemContext::BlueprintTree { .. } => {}
    }
}

fn add_picking_ray(
    line_builder: &mut re_renderer::LineDrawableBuilder<'_>,
    ray: macaw::Ray3,
    scene_bbox: &BoundingBox,
    thick_ray_length: f32,
    ray_color: egui::Color32,
) {
    let mut line_batch = line_builder.batch("picking ray");

    let origin = ray.point_along(0.0);

    // No harm in making this ray _very_ long. (Infinite messes with things though!)
    //
    // There are some degenerated cases where just taking the scene bounding box isn't enough:
    // For instance, we don't add pinholes & depth images to the bounding box since
    // the default size of a pinhole visualization itself is determined by the bounding box.
    let fallback_ray_end =
        ray.point_along((scene_bbox.size().length() * 10.0).at_least(thick_ray_length * 10.0));
    let main_ray_end = ray.point_along(thick_ray_length);

    line_batch
        .add_segment(origin, main_ray_end)
        .color(ray_color)
        .radius(Size::new_ui_points(1.0));
    line_batch
        .add_segment(main_ray_end, fallback_ray_end)
        .color(ray_color.gamma_multiply(0.7))
        // TODO(andreas): Make this dashed.
        .radius(Size::new_ui_points(0.5));
}

fn default_eye(
    bounding_box: &macaw::BoundingBox,
    scene_view_coordinates: Option<ViewCoordinates>,
) -> ViewEye {
    // Defaults to RFU.
    let scene_view_coordinates = scene_view_coordinates.unwrap_or_default();
    let scene_right = scene_view_coordinates
        .right()
        .unwrap_or(SignedAxis3::POSITIVE_X);
    let scene_forward = scene_view_coordinates
        .forward()
        .unwrap_or(SignedAxis3::POSITIVE_Y);
    let scene_up = scene_view_coordinates
        .up()
        .unwrap_or(SignedAxis3::POSITIVE_Z);

    let mut center = bounding_box.center();
    if !center.is_finite() {
        center = Vec3::ZERO;
    }

    let mut radius = 1.5 * bounding_box.half_size().length();
    if !radius.is_finite() || radius == 0.0 {
        radius = 1.0;
    }

    let eye_up: glam::Vec3 = scene_up.into();

    let eye_dir = {
        // Make sure right is to the right, and up is up:
        let right = scene_right.into();
        let fwd = eye_up.cross(right);
        0.75 * fwd + 0.25 * right - 0.25 * eye_up
    };
    let eye_dir = eye_dir.try_normalize().unwrap_or(scene_forward.into());

    let eye_pos = center - radius * eye_dir;

    ViewEye::new_orbital(
        center,
        radius,
        Quat::from_affine3(&Affine3A::look_at_rh(eye_pos, center, eye_up).inverse()),
        eye_up,
    )
}

#[test]
fn test_help_view() {
    re_viewer_context::test_context::TestContext::test_help_view(help);
}
