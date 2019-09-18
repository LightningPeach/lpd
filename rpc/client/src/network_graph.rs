use interface::routing::ChannelGraph;

pub fn dot_format(graph: ChannelGraph) -> String {
    use interface::routing::{LightningNode, ChannelEdge};

    let nodes_number = graph.nodes.len();
    let edges_number = graph.edges.len();

    let node_dot = |node: &LightningNode| -> String {
        let addresses = node.addresses.iter()
            .fold(String::new(), |a, address| format!("{};{} {}", a, address.network, address.addr));
        format!(
            "{} [alias=\"{}\",last_update={},color=\"{}\",addresses=\"{}\"];\n",
            node.pub_key,
            node.alias,
            node.last_update,
            node.color,
            addresses.trim_start_matches(';'),
        )
    };

    let edge_dot = |edge: &ChannelEdge| -> String {
        use interface::{common::MilliSatoshi, routing::RoutingPolicy};

        let policy = |policy: Option<&RoutingPolicy>| -> String {
            if let Some(policy) = policy {
                format!(
                    "{{time_lock_delta: {}, min_htlc: {}, fee_base: {}, fee_rate: {}, disabled: {}}}",
                    policy.time_lock_delta,
                    policy.min_htlc,
                    policy.fee_base_msat,
                    policy.fee_rate_milli.as_ref().map(MilliSatoshi::get_value).unwrap_or(0),
                    policy.disabled,
                )
            } else {
                "None".to_owned()
            }
        };
        format!(
            "{} -- {} [last_update={},capacity={},channel_id={:x},chan_point=\"{}\",node1_policy=\"{}\",node2_policy=\"{}\"];\n",
            edge.node1_pub,
            edge.node2_pub,
            edge.last_update,
            edge.capacity,
            edge.channel_id,
            edge.chan_point,
            policy(edge.node1_policy.as_ref()),
            policy(edge.node2_policy.as_ref()),
        )
    };

    let nodes = graph.nodes.iter().fold(String::new(), |a, item| a + node_dot(item).as_str());
    let edges = graph.edges.iter().fold(String::new(), |a, item| a + edge_dot(item).as_str());

    format!(
        "graph network_map {{\n\
            \tsize = \"{},{}\";\n\
            \t{}\
            \t{}\
            }}\
            ",
        nodes_number,
        edges_number,
        nodes,
        edges,
    )
}
