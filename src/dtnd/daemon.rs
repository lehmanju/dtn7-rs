use std::collections::HashMap;

use super::{httpd, janitor};
use crate::cla::{self, ConvergenceLayerAgent};
use crate::core::application_agent::SimpleApplicationAgent;
use crate::core::store::{BundleStoresEnum, InMemoryBundleStore};
use crate::core::{store, DtnStatistics};
use crate::dtnconfig::DtnConfig;
use crate::ipnd::neighbour_discovery;
use crate::peers_add;
use crate::{cla_add, routing, DtnCore, DtnPeer};
use bp7::Bundle;
use futures::channel::mpsc::Sender;
use log::{error, info};

/*
use crate::core::core::DtnCore;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub enum DtnCmd {
    DtnCore(Sender<DtnCmdResult>),
}

#[derive(Debug)]
pub enum DtnCmdResult {
    DtnCore(Sender<DtnCore>, DtnCore),
}

pub fn access_core<F>(tx: Sender<DtnCmd>, mut f: F)
where
    F: FnMut(&mut DtnCore),
{
    let (tx2, rx2) = mpsc::channel();
    tx.send(DtnCmd::DtnCore(tx2)).unwrap();
    let DtnCmdResult::DtnCore(tx3, mut core) = rx2.recv().expect("Couldn't access dtn core!");
    f(&mut core);
    tx3.send(core).expect("IPC Error");
}

fn spawn_core_daemon(rx: Receiver<DtnCmd>) {
    for received in rx {
        //println!("Got: {:?}", received);
        match received {
            DtnCmd::DtnCore(tx) => {
                let (tx2, rx2) = mpsc::channel();
                tx.send(DtnCmdResult::DtnCore(tx2, core))
                    .expect("IPC Error");
                core = rx2.recv().unwrap();
            }
        };
    }
}*/

pub struct DtnDaemon {
    config: DtnConfig,
    dtn_core: DtnCore,
    peers: HashMap<String, DtnPeer>,
    stats: DtnStatistics,
    sender_task: Option<Sender<Bundle>>,
    store: BundleStoresEnum,
}

impl DtnDaemon {
    pub fn new(config: DtnConfig) -> Self {
        info!("Local Node ID: {}", config.host_eid);
        info!("Work Dir: {:?}", config.workdir);
        info!("DB Backend: {}", config.db);
        info!(
            "Announcement Interval: {}",
            humantime::format_duration(config.announcement_interval)
        );
        info!(
            "Janitor Interval: {}",
            humantime::format_duration(config.janitor_interval)
        );

        info!(
            "Peer Timeout: {}",
            humantime::format_duration(config.peer_timeout)
        );
        info!("Web Port: {}", config.webport);
        info!("IPv4: {}", config.v4);
        info!("IPv6: {}", config.v6);
        info!("RoutingAgent: {}", config.routing);

        for cla in &config.clas {
            info!("Adding CLA: {}", cla);
            cla_add(cla::new(cla));
        }

        for s in config.statics {
            let port_str = s.cla_list[0]
                .1
                .map(|v| format!("{}", v))
                .unwrap_or("".into());
            info!(
                "Adding static peer: {}://{}:{}/{}",
                s.cla_list[0].0,
                s.addr,
                port_str,
                s.eid.node().unwrap()
            );
            peers_add(s.clone());
        }

        let local_host_id = config.host_eid;
        let dtn_core = DtnCore::new();
        dtn_core
            .register_application_agent(SimpleApplicationAgent::with(local_host_id.clone()).into());
        for e in &config.endpoints {
            let eid = local_host_id
                .new_endpoint(e)
                .expect("Error constructing new endpoint");
            dtn_core.register_application_agent(SimpleApplicationAgent::with(eid).into());
        }

        let store = store::new(&config.db);

        dtn_core.routing_agent = routing::new(&config.routing);
        Self {
            config,
            dtn_core,
            peers: HashMap::new(),
            stats: DtnStatistics::new(),
            sender_task: None,
            store,
        }
    }
    pub async fn spawn_daemon(&mut self) -> anyhow::Result<()> {
        info!("Starting convergency layers");
        for cl in &mut self.dtn_core.cl_list {
            info!("Setup {}", cl);
            cl.setup();
        }
        if self.config.janitor_interval.as_micros() != 0 {
            janitor::spawn_janitor();
        }
        if self.config.announcement_interval.as_micros() != 0 {
            if let Err(errmsg) = neighbour_discovery::spawn_neighbour_discovery().await {
                error!("Error spawning service discovery: {:?}", errmsg);
            }
        }
        // to task
        httpd::spawn_httpd().await?;
        // mpsc queue poll and message processing
        Ok(())
    }
}
