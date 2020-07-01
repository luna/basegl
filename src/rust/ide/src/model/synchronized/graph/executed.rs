//! A module with Executed Graph Controller.
//!
//! This controller provides operations on a specific graph with some execution context - these
//! operations usually involves retrieving values on nodes: that's are i.e. operations on
//! visualisations, retrieving types on ports, etc.
use crate::prelude::*;

use crate::model::execution_context::ComputedValueInfoRegistry;
use crate::model::execution_context::Visualization;
use crate::model::execution_context::VisualizationId;
use crate::model::execution_context::VisualizationUpdateData;
use crate::model::synchronized::ExecutionContext;

use enso_protocol::language_server::MethodPointer;
use flo_stream::MessagePublisher;



// ==============
// === Errors ===
// ==============

#[allow(missing_docs)]
#[fail(display = "The node {} has not been evaluated yet.", _0)]
#[derive(Debug,Fail,Clone,Copy)]
pub struct NotEvaluatedYet(double_representation::node::Id);

#[allow(missing_docs)]
#[fail(display = "The node {} does not resolve to a method call.", _0)]
#[derive(Debug,Fail,Clone,Copy)]
pub struct NoResolvedMethod(double_representation::node::Id);



// ====================
// === Notification ===
// ====================

/// Notification about change in the executed graph.
///
/// It may pertain either the state of the graph itself or the notifications from the execution.
#[derive(Clone,Debug,PartialEq)]
pub enum Notification {
    /// The notification passed from the graph controller.
    Graph(crate::controller::graph::Notification),
    /// The notification from the execution context about the computed value information
    /// being updated.
    ComputedValueInfo(crate::model::execution_context::ComputedValueExpressions),
    /// Notification emitted when the node has been entered.
    EnteredNode(double_representation::node::Id),
    /// Notification emitted when the node was step out.
    SteppedOutOfNode(double_representation::node::Id),
}



// ==============
// === Model ===
// ==============

/// Model being the executed graph, i.e. the graph and associated execution context.
#[derive(Debug)]
pub struct Model {
    #[allow(missing_docs)]
    pub logger:Logger,
    /// A handle to basic graph operations.
    graph:RefCell<controller::Graph>,
    /// Execution Context handle, its call stack top contains `graph`'s definition.
    execution_ctx:Rc<ExecutionContext>,
    /// The handle to project controller is necessary, as entering nodes might need to switch
    /// modules, and only the project can provide their controllers.
    project:controller::Project,
    /// The publisher allowing sending notification to subscribed entities. Note that its outputs is
    /// merged with publishers from the stored graph and execution controllers.
    notifier:RefCell<crate::notification::Publisher<Notification>>,
}

impl Model {
    /// Create handle for given graph and execution context.
    ///
    /// This takes a (shared) ownership of execution context which will be shared between all copies
    /// of this handle. It is held through `Rc` because the registry in the project controller needs
    /// to store a weak handle to the execution context as well (to be able to properly route some
    /// notifications, like visualization updates).
    ///
    /// However, in a typical setup, this controller handle (and its copies) shall be the only
    /// strong references to the execution context and it is expected that it will be dropped after
    /// the last copy of this controller is dropped.
    /// Then the context when being dropped shall remove itself from the Language Server.
    pub fn new
    ( graph:controller::Graph
    , project:&controller::Project
    , execution_ctx:Rc<ExecutionContext>
    ) -> Self {
        let logger   = Logger::sub(&graph.logger,"Executed");
        let graph    = RefCell::new(graph);
        let project  = project.clone_ref();
        let notifier = default();
        Model {logger,graph,execution_ctx,project,notifier}
    }

    /// See `attach_visualization` in `ExecutionContext`.
    pub async fn attach_visualization
    (&self, visualization:Visualization)
    -> FallibleResult<impl Stream<Item=VisualizationUpdateData>> {
        self.execution_ctx.attach_visualization(visualization).await
    }

    /// See `detach_visualization` in `ExecutionContext`.
    pub async fn detach_visualization(&self, id:VisualizationId) -> FallibleResult<Visualization> {
        self.execution_ctx.detach_visualization(id).await
    }

    /// See `expression_info_registry` in `ExecutionContext`.
    pub fn computed_value_info_registry(&self) -> &ComputedValueInfoRegistry {
        self.execution_ctx.computed_value_info_registry()
    }

    /// Subscribe to updates about changes in this executed graph.
    ///
    /// The stream of notification contains both notifications from the graph and from the execution
    /// context.
    pub fn subscribe(&self) -> impl Stream<Item=Notification> {
        let registry     = self.execution_ctx.computed_value_info_registry();
        let value_stream = registry.subscribe().map(Notification::ComputedValueInfo).boxed_local();
        let graph_stream = self.graph().subscribe().map(Notification::Graph).boxed_local();
        let self_stream = self.notifier.borrow_mut().subscribe().boxed_local();
        futures::stream::select_all(vec![value_stream,graph_stream,self_stream])
    }

    /// Create a graph controller for the given method.
    ///
    /// Fails if the module is inaccessible or if it does not contain given method.
    pub async fn graph_for_method
    (&self, method:&MethodPointer) -> FallibleResult<controller::Graph> {
        let module_path = model::module::Path::from_file_path(method.file.clone())?;
        let module      = self.project.module_controller(module_path).await?;
        debug!(self.logger,"Looking up method definition {method:?} in the module.");
        let module_ast = module.model.model.ast();
        let definition = double_representation::module::lookup_method(&module_ast,method)?;
        module.graph_controller(definition)
    }

    /// Step into node by given ID.
    ///
    /// This will cause pushing a new stack frame to the execution context and changing the graph
    /// controller to point to a new definition.
    ///
    /// Fails if there's no information about target method pointer (e.g. because node value hasn't
    /// been yet computed by the engine) or if method graph cannot be created (see
    /// `graph_for_method` documentation).
    pub async fn step_into_node(&self, node:double_representation::node::Id) -> FallibleResult<()> {
        debug!(self.logger, "Entering node {node}");
        let registry   = self.execution_ctx.computed_value_info_registry();
        let node_info  = registry.get(&node).ok_or_else(|| NotEvaluatedYet(node))?;
        let method_ptr = node_info.method_call.as_ref().ok_or_else(|| NoResolvedMethod(node))?;
        let graph      = self.graph_for_method(method_ptr).await?;
        let call       = model::execution_context::LocalCall {
            call : node,
            definition : method_ptr.clone()
        };
        self.execution_ctx.push(call).await?;
        self.graph.replace(graph);
        self.notifier.borrow_mut().publish(Notification::EnteredNode(node)).await;
        Ok(())
    }

    /// Step out of the current node. Reverse of `step_into_node`.
    ///
    /// Fails if this execution context is already at the stack's root or if the parent graph
    /// cannot be retrieved.
    pub async fn step_out_of_node(&self) -> FallibleResult<()> {
        let frame  = self.execution_ctx.pop().await?;
        let method = self.execution_ctx.current_method();
        let graph  = self.graph_for_method(&method).await?;
        self.graph.replace(graph);
        self.notifier.borrow_mut().publish(Notification::SteppedOutOfNode(frame.call)).await;
        Ok(())
    }

    /// Get the controller for the currently active graph.
    ///
    /// Note that the controller returned by this method may change as the nodes are stepped into.
    pub fn graph(&self) -> controller::Graph {
        self.graph.borrow().clone_ref()
    }
}



// ============
// === Test ===
// ============

#[cfg(test)]
mod tests {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;

    use enso_protocol::language_server;
    use utils::test::traits::*;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    /// Test that checks that value computed notification is properly relayed by the executed graph.
    #[wasm_bindgen_test]
    fn dispatching_value_computed_notification() {
        // Setup the controller.
        let mut fixture    = TestWithLocalPoolExecutor::set_up();
        let mut ls         = language_server::MockClient::default();
        let execution_data = model::synchronized::execution_context::tests::MockData::new();
        let execution      = execution_data.context_provider(&mut ls);
        let graph_data     = controller::graph::test_utils::MockData::new_inline("1 + 2");
        let connection     = language_server::Connection::new_mock_rc(ls);
        let (_,graph)      = graph_data.create_controllers_with_ls(connection.clone_ref());
        let execution      = Rc::new(execution(connection.clone_ref()));
        let project        = controller::project::test::setup_mock_project(|_| {}, |_| {});
        let executed_graph = Model::new(graph, &project, execution.clone_ref());

        // Generate notification.
        let notification = execution_data.mock_values_computed_update();
        let update       = &notification.updates[0];

        // Notification not yet send.
        let registry          = executed_graph.computed_value_info_registry();
        let mut notifications = executed_graph.subscribe().boxed_local();
        notifications.expect_pending();
        assert!(registry.get(&update.id).is_none());

        // Sending notification.
        execution.handle_expression_values_computed(notification.clone()).unwrap();
        fixture.run_until_stalled();

        // Observing that notification was relayed.
        let observed_notification = notifications.expect_next();
        let typename_in_registry  = registry.get(&update.id).unwrap().typename.clone();
        let expected_typename     = update.typename.clone().map(ImString::new);
        assert_eq!(observed_notification,Notification::ComputedValueInfo(vec![update.id]));
        assert_eq!(typename_in_registry,expected_typename);
        notifications.expect_pending();
    }
}
