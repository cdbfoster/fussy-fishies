use std::marker::PhantomData;

use bevy::core_pipeline::{self, draw_2d_graph, Transparent2d};
use bevy::prelude::*;
use bevy::render::camera::{ActiveCamera, CameraTypePlugin};
use bevy::render::render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotValue};
use bevy::render::render_phase::RenderPhase;
use bevy::render::renderer::RenderContext;
use bevy::render::{RenderApp, RenderStage};

#[derive(Default)]
pub struct AdditionalPassPlugin<T> {
    pass_name: &'static str,
    before_node: Option<&'static str>,
    _marker: PhantomData<T>,
}

impl<T> AdditionalPassPlugin<T> {
    pub fn new(pass_name: &'static str, before_node: Option<&'static str>) -> Self {
        Self {
            pass_name,
            before_node,
            _marker: PhantomData,
        }
    }
}

impl<T: Component + Default> Plugin for AdditionalPassPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_plugin(CameraTypePlugin::<T>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_system_to_stage(
            RenderStage::Extract,
            extract_additional_pass_camera_phases::<T>,
        );

        let additional_pass_driver = AdditionalPassDriver::<T>::new(&mut render_app.world);

        let mut graph = render_app.world.resource_mut::<RenderGraph>();

        graph.add_node(self.pass_name, additional_pass_driver);

        graph
            .add_node_edge(core_pipeline::node::MAIN_PASS_DEPENDENCIES, self.pass_name)
            .expect("could not add node edge");

        graph
            .add_node_edge(core_pipeline::node::CLEAR_PASS_DRIVER, self.pass_name)
            .expect("could not add node edge");

        if let Some(before_node) = self.before_node {
            graph
                .add_node_edge(self.pass_name, before_node)
                .expect("could not add node edge");
        }

        graph
            .add_node_edge(self.pass_name, core_pipeline::node::MAIN_PASS_DRIVER)
            .expect("could not add node edge");
    }
}

fn extract_additional_pass_camera_phases<T: Component>(
    mut commands: Commands,
    active: Res<ActiveCamera<T>>,
) {
    if let Some(entity) = active.get() {
        commands
            .get_or_spawn(entity)
            .insert(RenderPhase::<Transparent2d>::default());
    }
}

struct AdditionalPassDriver<T: Component> {
    query: QueryState<Entity, With<T>>,
}

impl<T: Component> AdditionalPassDriver<T> {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}

impl<T: Component> Node for AdditionalPassDriver<T> {
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
