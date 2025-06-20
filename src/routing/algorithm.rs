use std::collections::HashSet;

// --- 型定義 ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EndpointId(String);

struct BundleDescriptor {
    already_sent: HashSet<EndpointId>,
}

impl BundleDescriptor {
    fn get_already_sent(&self) -> &HashSet<EndpointId> {
        &self.already_sent
    }
}

trait ConvergenceSender {
    fn get_peer_endpoint_id(&self) -> EndpointId;
}

struct TcpSender {
    peer_id: EndpointId,
}

impl ConvergenceSender for TcpSender {
    fn get_peer_endpoint_id(&self) -> EndpointId {
        self.peer_id.clone()
    }
}

// --- Routing Algorithm Trait ---

trait RoutingAlgorithm {
    fn notify_new_bundle(&mut self, descriptor: &BundleDescriptor);
    fn select_peers_for_forwarding<'a>(
        &self,
        descriptor: &BundleDescriptor,
        all_senders: &'a [Box<dyn ConvergenceSender>],
    ) -> Vec<&'a Box<dyn ConvergenceSender>>;
}

// --- Epidemic Routing ---

struct EpidemicRouting;

impl RoutingAlgorithm for EpidemicRouting {
    fn notify_new_bundle(&mut self, _descriptor: &BundleDescriptor) {
        // Do nothing
    }

    fn select_peers_for_forwarding<'a>(
        &self,
        descriptor: &BundleDescriptor,
        all_senders: &'a [Box<dyn ConvergenceSender>],
    ) -> Vec<&'a Box<dyn ConvergenceSender>> {
        let already_sent = descriptor.get_already_sent();

        let mut seen_eids = HashSet::new();
        let mut result = Vec::new();

        for sender in all_senders {
            let eid = sender.get_peer_endpoint_id();
            if !already_sent.contains(&eid) && seen_eids.insert(eid.clone()) {
                result.push(sender);
            }
        }

        result
    }
}
