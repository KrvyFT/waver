//! [`crate::Graph`] → [`crate::Schedule`] compiler (validate, Kahn, delay insertion).

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    Graph, GraphError, Link, Node, NodeId, NodeKind, PortDirection, PortId, PortRef, Schedule,
};

/// Upper bound on automatic delay nodes inserted to break feedback loops.
const MAX_DELAY_INSERTS: u32 = 64;

/// Internal edge with an optional same-block dependency for topological sort.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Wire {
    from: PortRef,
    to: PortRef,
    /// When false, the link routes audio but does not force same-block ordering
    /// (used on the output side of a delay breaking a feedback path).
    same_block_dep: bool,
}

impl Wire {
    fn dependency(from: PortRef, to: PortRef) -> Self {
        Self {
            from,
            to,
            same_block_dep: true,
        }
    }

    fn routing(from: PortRef, to: PortRef) -> Self {
        Self {
            from,
            to,
            same_block_dep: false,
        }
    }

    fn to_link(self) -> Link {
        Link {
            from: self.from,
            to: self.to,
        }
    }
}

/// Validate topology, break cycles with delay nodes when needed, emit a schedule.
pub(crate) fn compile_graph(graph: &Graph) -> Result<Schedule, GraphError> {
    validate(graph)?;

    let mut nodes: Vec<Node> = graph.nodes().to_vec();
    let mut wires: Vec<Wire> = graph
        .edges()
        .iter()
        .map(|edge| Wire::dependency(edge.from, edge.to))
        .collect();
    let mut next_id = graph.next_node_id();
    let mut delay_inserts = 0u32;

    loop {
        match kahn(&nodes, &wires) {
            Ok(order) => return Ok(build_schedule(order, &nodes, &wires)),
            Err(remaining) => {
                if delay_inserts >= MAX_DELAY_INSERTS {
                    return Err(GraphError::Cycle);
                }
                let wire_idx = pick_cycle_wire(&remaining, &wires).ok_or(GraphError::Cycle)?;
                let delay_id = NodeId::new(next_id);
                next_id = next_id.saturating_add(1);
                delay_inserts = delay_inserts.saturating_add(1);
                nodes.push(Node {
                    id: delay_id,
                    kind: NodeKind::Delay,
                });
                split_wire_with_delay(&mut wires, wire_idx, delay_id);
            }
        }
    }
}

fn validate(graph: &Graph) -> Result<(), GraphError> {
    let kinds: HashMap<NodeId, NodeKind> = graph
        .nodes()
        .iter()
        .map(|node| (node.id, node.kind))
        .collect();

    for edge in graph.edges() {
        let from_kind = kinds.get(&edge.from.node).ok_or(GraphError::UnknownNode {
            node: edge.from.node,
        })?;
        let to_kind = kinds.get(&edge.to.node).ok_or(GraphError::UnknownNode {
            node: edge.to.node,
        })?;

        if edge.from.node == edge.to.node {
            return Err(GraphError::SelfLoop {
                node: edge.from.node,
            });
        }

        if !from_kind.is_output_port(edge.from.port) {
            return Err(GraphError::InvalidPort {
                node: edge.from.node,
                port: edge.from.port,
                direction: PortDirection::Output,
            });
        }

        if !to_kind.is_input_port(edge.to.port) {
            return Err(GraphError::InvalidPort {
                node: edge.to.node,
                port: edge.to.port,
                direction: PortDirection::Input,
            });
        }
    }

    Ok(())
}

/// Kahn topological sort using only same-block dependency wires.
fn kahn(nodes: &[Node], wires: &[Wire]) -> Result<Vec<NodeId>, Vec<NodeId>> {
    let ids: HashSet<NodeId> = nodes.iter().map(|node| node.id).collect();
    let mut in_degree: HashMap<NodeId, u32> = ids.iter().copied().map(|id| (id, 0)).collect();
    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for wire in wires {
        if !wire.same_block_dep {
            continue;
        }
        if !ids.contains(&wire.from.node) || !ids.contains(&wire.to.node) {
            continue;
        }
        *in_degree.entry(wire.to.node).or_insert(0) += 1;
        adjacency
            .entry(wire.from.node)
            .or_default()
            .push(wire.to.node);
    }

    for neighbors in adjacency.values_mut() {
        neighbors.sort_unstable_by_key(|id| id.raw());
    }

    let mut ready: Vec<NodeId> = in_degree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(&id, _)| id)
        .collect();
    ready.sort_unstable_by_key(|id| id.raw());

    let mut order = Vec::with_capacity(ids.len());
    let mut queue: VecDeque<NodeId> = ready.into();

    while let Some(id) = queue.pop_front() {
        order.push(id);
        if let Some(neighbors) = adjacency.get(&id) {
            for &next in neighbors {
                let degree = in_degree.get_mut(&next).expect("wire target in graph");
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(next);
                }
            }
        }
    }

    if order.len() == ids.len() {
        Ok(order)
    } else {
        let scheduled: HashSet<NodeId> = order.iter().copied().collect();
        let mut remaining: Vec<NodeId> = ids
            .into_iter()
            .filter(|id| !scheduled.contains(id))
            .collect();
        remaining.sort_unstable_by_key(|id| id.raw());
        Err(remaining)
    }
}

fn pick_cycle_wire(remaining: &[NodeId], wires: &[Wire]) -> Option<usize> {
    let set: HashSet<NodeId> = remaining.iter().copied().collect();
    wires.iter().position(|wire| {
        wire.same_block_dep && set.contains(&wire.from.node) && set.contains(&wire.to.node)
    })
}

fn split_wire_with_delay(wires: &mut Vec<Wire>, idx: usize, delay_id: NodeId) {
    let wire = wires.remove(idx);
    wires.push(Wire::dependency(
        wire.from,
        PortRef {
            node: delay_id,
            port: PortId::new(0),
        },
    ));
    wires.push(Wire::routing(
        PortRef {
            node: delay_id,
            port: PortId::new(0),
        },
        wire.to,
    ));
}

fn build_schedule(order: Vec<NodeId>, nodes: &[Node], wires: &[Wire]) -> Schedule {
    let kinds: HashMap<NodeId, NodeKind> = nodes.iter().map(|node| (node.id, node.kind)).collect();
    let links: Vec<Link> = wires.iter().copied().map(Wire::to_link).collect();
    Schedule::new(order, kinds, links)
}

#[cfg(test)]
mod tests {
    use super::compile_graph;
    use crate::{Graph, GraphError, NodeId, NodeKind, PortId, PortRef};

    fn port(node: NodeId, index: u32) -> PortRef {
        PortRef {
            node,
            port: PortId::new(index),
        }
    }

    #[test]
    fn linear_chain_orders_sources_before_sinks() {
        let mut graph = Graph::new();
        let vco = graph.insert(NodeKind::Vco);
        let vcf = graph.insert(NodeKind::Vcf);
        let out = graph.insert(NodeKind::Output);
        graph.connect(port(vco, 0), port(vcf, 0));
        graph.connect(port(vcf, 0), port(out, 0));

        let schedule = compile_graph(&graph).expect("linear graph");
        let order = schedule.order();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], vco);
        assert_eq!(order[1], vcf);
        assert_eq!(order[2], out);
        assert_eq!(schedule.links().len(), 2);
    }

    #[test]
    fn fan_in_records_multiple_sources() {
        let mut graph = Graph::new();
        let a = graph.insert(NodeKind::Vco);
        let b = graph.insert(NodeKind::Lfo);
        let mix = graph.insert(NodeKind::Mixer);
        graph.connect(port(a, 0), port(mix, 0));
        graph.connect(port(b, 0), port(mix, 1));

        let schedule = compile_graph(&graph).expect("fan-in graph");
        assert_eq!(schedule.sources_to(port(mix, 0)), vec![port(a, 0)]);
        assert_eq!(schedule.sources_to(port(mix, 1)), vec![port(b, 0)]);
    }

    #[test]
    fn feedback_loop_gets_delay_node() {
        let mut graph = Graph::new();
        let vco = graph.insert(NodeKind::Vco);
        let vcf = graph.insert(NodeKind::Vcf);
        let vca = graph.insert(NodeKind::Vca);
        graph.connect(port(vco, 0), port(vcf, 0));
        graph.connect(port(vcf, 0), port(vca, 0));
        graph.connect(port(vca, 0), port(vcf, 1));

        let schedule = compile_graph(&graph).expect("cycle broken by delay");
        assert!(schedule.order().iter().any(|id| schedule.kind_of(*id) == Some(NodeKind::Delay)));
        assert!(schedule.order().len() >= 4);
    }

    #[test]
    fn invalid_port_is_rejected() {
        let mut graph = Graph::new();
        let vco = graph.insert(NodeKind::Vco);
        let out = graph.insert(NodeKind::Output);
        graph.connect(port(vco, 0), port(out, 99));

        let err = compile_graph(&graph).expect_err("bad port");
        assert!(matches!(err, GraphError::InvalidPort { .. }));
    }

    #[test]
    fn self_loop_is_rejected() {
        let mut graph = Graph::new();
        let mix = graph.insert(NodeKind::Mixer);
        graph.connect(port(mix, 0), port(mix, 1));

        let err = compile_graph(&graph).expect_err("self loop");
        assert!(matches!(err, GraphError::SelfLoop { .. }));
    }
}
