use lnd_rust::rpc::LightningAddress;

pub trait AbstractLightningNode {
    fn address(&self) -> LightningAddress;
}
