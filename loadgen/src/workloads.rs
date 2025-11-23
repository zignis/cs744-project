use crate::client::HttpClient;
use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub enum WorkloadType {
    PutAll,
    GetAll,
    GetPopular,
    Mixed,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub payload_size: usize,
    pub hotset: usize,
    pub mixed_get_pct: u8,
    pub mixed_put_pct: u8,
    pub mixed_delete_pct: u8,
    pub mixed_hot_get_pct: u8,
}

#[derive(Debug, Clone)]
pub enum ReqMethod {
    POST,
    GET,
    DELETE,
}

#[derive(Debug, Clone)]
pub struct Req {
    pub method: ReqMethod,
    pub key: String,
    pub value: Option<String>,
}

pub struct WorkloadGenerator {
    kind: WorkloadType,
    cfg: Config,
    rng: StdRng,
    counter: u64,
    hotset_keys: Vec<String>,
}

fn generate_hotset_keys(hotset: usize) -> Vec<String> {
    let mut hotset_keys = Vec::new();

    for i in 0..hotset {
        hotset_keys.push(format!("_hot_{:06}", i));
    }

    hotset_keys
}

impl WorkloadGenerator {
    pub fn new(kind: WorkloadType, cfg: Config, instance_id: u64) -> Self {
        let rng = StdRng::seed_from_u64({
            let now = OffsetDateTime::now_utc().unix_timestamp_nanos();
            (now as u64) ^ instance_id
        });

        // initialize hotset
        let hotset_keys = generate_hotset_keys(cfg.hotset);

        WorkloadGenerator {
            kind,
            cfg,
            rng,
            counter: instance_id * 1_000_000, // unique key offset per thread
            hotset_keys,
        }
    }

    #[inline(always)]
    fn next_unique_key(&mut self) -> String {
        let id = self.counter;
        self.counter += 1;
        format!("key_{:016x}", id)
    }

    #[inline(always)]
    fn random_value(&mut self) -> String {
        let mut buf = vec![0u8; self.cfg.payload_size];
        self.rng.fill(&mut buf[..]);
        hex::encode(buf)
    }

    pub fn next_request(&mut self) -> Req {
        match self.kind {
            WorkloadType::PutAll => Req {
                method: ReqMethod::POST,
                key: self.next_unique_key(),
                value: Some(self.random_value()),
            },

            WorkloadType::GetAll => Req {
                method: ReqMethod::GET,
                key: self.next_unique_key(),
                value: None,
            },

            WorkloadType::GetPopular => {
                let idx = self.rng.random_range(0..self.hotset_keys.len());
                Req {
                    method: ReqMethod::GET,
                    key: self.hotset_keys[idx].clone(),
                    value: None,
                }
            }

            WorkloadType::Mixed => {
                let roll = self.rng.random_range(0..100);

                // PUT
                if roll < self.cfg.mixed_put_pct {
                    return Req {
                        method: ReqMethod::POST,
                        key: self.next_unique_key(),
                        value: Some(self.random_value()),
                    };
                }

                // DELETE
                if roll < self.cfg.mixed_put_pct + self.cfg.mixed_delete_pct {
                    return Req {
                        method: ReqMethod::DELETE,
                        key: self.next_unique_key(),
                        value: None,
                    };
                }

                // GET
                if self.rng.random_range(0..100) < self.cfg.mixed_hot_get_pct {
                    // hot GET
                    let idx = self.rng.random_range(0..self.hotset_keys.len());
                    Req {
                        method: ReqMethod::GET,
                        key: self.hotset_keys[idx].clone(),
                        value: None,
                    }
                } else {
                    // unique GET
                    Req {
                        method: ReqMethod::GET,
                        key: self.next_unique_key(),
                        value: None,
                    }
                }
            }
        }
    }
}

pub async fn preload_hotset(
    client: &HttpClient,
    hotset: usize,
    payload_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let hotset_keys = generate_hotset_keys(hotset);

    // database preload
    for key in hotset_keys.iter() {
        let res = client
            .send_request(Req {
                method: ReqMethod::POST,
                key: key.clone(),
                value: random_value(payload_size).into(),
            })
            .await?;

        if !res.is_success() {
            eprintln!("failed to preload key {key}");
        }
    }

    // cache warmup
    for key in hotset_keys.iter() {
        match client
            .send_request(Req {
                method: ReqMethod::GET,
                key: key.clone(),
                value: None,
            })
            .await
        {
            Ok(_) => {}
            Err(e) => eprintln!("warmup GET error for {key}: {e}"),
        }
    }

    Ok(())
}

fn random_value(payload_size: usize) -> String {
    let mut buf = vec![0u8; payload_size];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut buf);
    hex::encode(buf)
}
