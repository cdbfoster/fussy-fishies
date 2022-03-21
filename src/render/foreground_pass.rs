use bevy::core_pipeline::{self, draw_2d_graph, Transparent2d};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::camera::{ActiveCamera, CameraTypePlugin};
use bevy::render::render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotValue};
use bevy::render::render_phase::RenderPhase;
use bevy::render::renderer::RenderContext;
use bevy::render::{RenderApp, RenderStage};

use crate::render::cameras::ForegroundCamera;

pub const FOREGROUND_COLOR_TEXTURE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 0xDEADBEEF);

#[derive(Default)]
pub struct ForegroundPassPlugin;

impl Plugin for ForegroundPassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CameraTypePlugin::<ForegroundCamera>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_system_to_stage(RenderStage::Extract, extract_foreground_camera_phases);

        let foreground_pass_driver = ForegroundPassDriver::new(&mut render_app.world);

        let mut graph = render_app.world.resource_mut::<RenderGraph>();

        graph.add_node(node::FOREGROUND_PASS_DRIVER, foreground_pass_driver);

        graph
            .add_node_edge(
                core_pipeline::node::MAIN_PASS_DEPENDENCIES,
                node::FOREGROUND_PASS_DRIVER,
            )
            .expect("could not add node edge");

        graph
            .add_node_edge(
                core_pipeline::node::CLEAR_PASS_DRIVER,
                node::FOREGROUND_PASS_DRIVER,
            )
            .expect("could not add node edge");

        graph
            .add_node_edge(
                node::FOREGROUND_PASS_DRIVER,
                core_pipeline::node::MAIN_PASS_DRIVER,
            )
            .expect("could not add node edge");
    }
}

fn extract_foreground_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<ForegroundCamera>>,
) {
    if let Some(entity) = active.get() {
        commands
            .get_or_spawn(entity)
            .insert(RenderPhase::<Transparent2d>::default());
    }
}

mod node {
    pub const FOREGROUND_PASS_DRIVER: &str = "foreground_pass_driver";
}

struct ForegroundPassDriver {
    query: QueryState<Entity, With<ForegroundCamera>>,
}

impl ForegroundPassDriver {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}

impl Node for ForegroundPassDriver {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        for camera in self.query.iter_manual(world) {
            graph.run_sub_graph(draw_2d_graph::NAME, vec![SlotValue::Entity(camera)])?;
        }
        Ok(())
    }
}
