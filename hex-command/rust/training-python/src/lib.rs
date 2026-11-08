//! Python owns tensors; dedicated Rust worker threads own game lifetimes.
use hex_command_core::{data::*, simulation::*, training::*};
use numpy::ndarray::Array2;
use numpy::{IntoPyArray, PyArray1, PyArray2, PyReadonlyArray2};
use pyo3::{exceptions::PyValueError, prelude::*};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
fn err(e: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(e.to_string())
}
fn parse(config: &str) -> Result<MatchConfig, String> {
    let c: MatchConfig = serde_json::from_str(config).map_err(|e| e.to_string())?;
    let (a, b) = match &c {
        MatchConfig::Arena(c) => (c.player_blocks, c.enemy_blocks),
        MatchConfig::Campaign(c) => (c.player_blocks, c.enemy_blocks),
    };
    if [a, b]
        .into_iter()
        .flatten()
        .any(|b| b.planner == hex_command_core::ai::Planner::Learned)
    {
        return Err("supply baseline blocks; the bridge selects the learned planner itself".into());
    }
    Ok(c)
}
fn load_model(s: &str) -> Result<Model, String> {
    let m: Model = serde_json::from_str(s).map_err(|e| e.to_string())?;
    m.validate()?;
    Ok(m)
}
fn learned(config: &mut MatchConfig) {
    match config {
        MatchConfig::Arena(c) => {
            c.player_blocks = Some(learned_blocks(c.player_blocks.unwrap_or(c.player.blocks())))
        }
        MatchConfig::Campaign(c) => {
            c.player_blocks = Some(learned_blocks(c.player_blocks.unwrap_or(c.player.blocks())))
        }
    }
}
fn summary(r: &MatchResult) -> serde_json::Value {
    match r {
        MatchResult::Arena(r) => {
            serde_json::json!({"winner":r.winner,"reason":r.reason,"rounds":r.rounds,"success":r.winner==Some(Side::Player),"truncated":r.winner.is_none()})
        }
        MatchResult::Campaign(r) => {
            serde_json::json!({"success":r.completed,"missions":r.missions.iter().map(|m|serde_json::json!({"id":m.id,"round":m.round,"winner":m.winner})).collect::<Vec<_>>(),"truncated":r.missions.last().is_some_and(|m|m.winner.is_none())})
        }
    }
}
fn matrix(py: Python<'_>, rows: Vec<[f32; FEATURES]>) -> Py<PyArray2<f32>> {
    let n = rows.len();
    let flat = rows.into_iter().flatten().collect();
    Array2::from_shape_vec((n, FEATURES), flat)
        .unwrap()
        .into_pyarray(py)
        .unbind()
}
type DataResult = (
    Py<PyArray2<f32>>,
    Py<PyArray1<i64>>,
    Py<PyArray1<i64>>,
    usize,
);
#[pyfunction]
#[pyo3(signature=(configs, max_samples=100000))]
fn collect(py: Python<'_>, configs: Vec<String>, max_samples: usize) -> PyResult<DataResult> {
    if max_samples == 0 {
        return Err(err("max_samples must be positive"));
    }
    let data = py
        .detach(move || -> Result<Dataset, String> {
            let rules = Rules::load().map_err(|e| e.to_string())?;
            let mut definitions = CampaignDefinitions::new(&rules);
            let (result, data) = scoped(
                Some(Dataset {
                    limit: max_samples,
                    ..Default::default()
                }),
                None,
                None,
                || {
                    for config in configs {
                        let config = parse(&config)?;
                        definitions
                            .prepare_match(&config)
                            .map_err(|e| e.to_string())?;
                        run(&rules, config, &definitions).map_err(|e| e.to_string())?;
                    }
                    Ok::<_, String>(())
                },
            );
            result?;
            Ok(data.unwrap())
        })
        .map_err(err)?;
    Ok((
        matrix(py, data.features),
        data.offsets.into_pyarray(py).unbind(),
        data.labels.into_pyarray(py).unbind(),
        data.missing,
    ))
}
#[pyfunction]
fn evaluate(py: Python<'_>, configs: Vec<String>, model: Option<String>) -> PyResult<Vec<String>> {
    let model = model.map(|s| load_model(&s)).transpose().map_err(err)?;
    py.detach(move || -> Result<Vec<String>, String> {
        let rules = Rules::load().map_err(|e| e.to_string())?;
        let mut definitions = CampaignDefinitions::new(&rules);
        let (r, _) = scoped(None, model.clone(), None, || {
            configs
                .into_iter()
                .map(|s| {
                    let mut c = parse(&s)?;
                    if model.is_some() {
                        learned(&mut c);
                    }
                    definitions.prepare_match(&c).map_err(|e| e.to_string())?;
                    run(&rules, c, &definitions)
                        .map(|r| summary(&r).to_string())
                        .map_err(|e| e.to_string())
                })
                .collect::<Result<Vec<_>, String>>()
        });
        r
    })
    .map_err(err)
}
#[pyfunction]
fn score<'py>(
    py: Python<'py>,
    model: String,
    features: PyReadonlyArray2<'py, f32>,
) -> PyResult<Bound<'py, PyArray1<f32>>> {
    let m = load_model(&model).map_err(err)?;
    let a = features.as_array();
    if a.ncols() != FEATURES {
        return Err(err("feature width mismatch"));
    }
    Ok(a.rows()
        .into_iter()
        .map(|r| m.score(r.as_slice().unwrap_or(&r.to_vec())))
        .collect::<Vec<_>>()
        .into_pyarray(py))
}
enum Event {
    Decision(Choices),
    Done(Result<String, String>),
}
struct Worker {
    tx: Option<Sender<usize>>,
    rx: std::sync::Mutex<Receiver<Event>>,
    join: Option<JoinHandle<()>>,
    event: Option<Event>,
}
impl Worker {
    fn new(config: String) -> Self {
        let (tx, actions) = mpsc::channel();
        let (events, rx) = mpsc::channel();
        let join = std::thread::spawn(move || {
            let result = (|| -> Result<String, String> {
                let rules = Rules::load().map_err(|e| e.to_string())?;
                let mut c = parse(&config)?;
                learned(&mut c);
                let mut definitions = CampaignDefinitions::new(&rules);
                definitions.prepare_match(&c).map_err(|e| e.to_string())?;
                let emit = events.clone();
                let controller: hex_command_core::training::Controller = Box::new(move |choices| {
                    if emit.send(Event::Decision(choices.clone())).is_err() {
                        return 0;
                    }
                    actions.recv().unwrap_or(0)
                });
                let (result, _) = scoped(None, None, Some(controller), || {
                    run(&rules, c, &definitions)
                });
                result
                    .map(|r| summary(&r).to_string())
                    .map_err(|e| e.to_string())
            })();
            let _ = events.send(Event::Done(result));
        });
        Self {
            tx: Some(tx),
            rx: std::sync::Mutex::new(rx),
            join: Some(join),
            event: None,
        }
    }
    fn receive(&mut self) -> Result<(), String> {
        self.event = Some(self.rx.lock().unwrap().recv().map_err(|e| e.to_string())?);
        if let Some(Event::Done(Err(e))) = &self.event {
            return Err(e.clone());
        }
        Ok(())
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}
/// Packed candidates and offsets. Finished slots have zero candidates until reset.
#[pyclass]
struct BatchEnv {
    workers: Vec<Worker>,
}
type StepResult = (
    Py<PyArray2<f32>>,
    Py<PyArray1<i64>>,
    Vec<f32>,
    Vec<bool>,
    Vec<bool>,
);
impl BatchEnv {
    fn observation(&self, py: Python<'_>) -> PyResult<StepResult> {
        let mut features = vec![];
        let mut offsets = vec![0i64];
        let mut rewards = vec![];
        let mut terminated = vec![];
        let mut truncated = vec![];
        for w in &self.workers {
            let mut reward = 0.;
            let mut term = false;
            let mut trunc = false;
            match w.event.as_ref().unwrap() {
                Event::Decision(c) => features.extend_from_slice(&c.features),
                Event::Done(Ok(s)) => {
                    let v: serde_json::Value = serde_json::from_str(s).unwrap();
                    trunc = v["truncated"] == true;
                    term = !trunc;
                    reward = if v["success"] == true {
                        1.
                    } else if trunc {
                        0.
                    } else {
                        -1.
                    };
                }
                Event::Done(Err(e)) => return Err(err(e)),
            }
            offsets.push(features.len() as i64);
            rewards.push(reward);
            terminated.push(term);
            truncated.push(trunc);
        }
        Ok((
            matrix(py, features),
            offsets.into_pyarray(py).unbind(),
            rewards,
            terminated,
            truncated,
        ))
    }
}
#[pymethods]
impl BatchEnv {
    #[new]
    fn new() -> Self {
        Self { workers: vec![] }
    }
    fn reset(&mut self, py: Python<'_>, configs: Vec<String>) -> PyResult<StepResult> {
        if configs.is_empty() || configs.len() > 64 {
            return Err(err("provide 1..64 configurations"));
        }
        for c in &configs {
            parse(c).map_err(err)?;
        }
        py.detach(|| -> Result<(), String> {
            self.workers.clear();
            self.workers = configs.into_iter().map(Worker::new).collect();
            for w in &mut self.workers {
                w.receive()?;
            }
            Ok(())
        })
        .map_err(err)?;
        self.observation(py)
    }
    fn step(&mut self, py: Python<'_>, actions: Vec<usize>) -> PyResult<StepResult> {
        if actions.len() != self.workers.len() || self.workers.is_empty() {
            return Err(err("one action per environment required"));
        }
        for (w, &a) in self.workers.iter().zip(&actions) {
            if let Some(Event::Decision(c)) = &w.event {
                if a >= c.commands.len() {
                    return Err(err("candidate index out of range"));
                }
            }
        }
        py.detach(|| -> Result<(), String> {
            // Dispatch the whole batch before waiting so native workers overlap.
            for (w, a) in self.workers.iter().zip(actions) {
                if matches!(w.event, Some(Event::Decision(_))) {
                    w.tx.as_ref().unwrap().send(a).map_err(|e| e.to_string())?;
                }
            }
            for w in &mut self.workers {
                if matches!(w.event, Some(Event::Decision(_))) {
                    w.receive()?;
                }
            }
            Ok(())
        })
        .map_err(err)?;
        self.observation(py)
    }
    fn close(&mut self, py: Python<'_>) {
        py.detach(|| self.workers.clear());
    }
}
#[pyfunction]
fn fingerprints(py: Python<'_>) -> PyResult<String> {
    py.detach(|| {
        let rules = Rules::load().map_err(err)?;
        Ok(hex_command_core::simulation::cli::fingerprints(&rules).to_string())
    })
}
#[pymodule]
fn hex_command_training(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("SCHEMA", SCHEMA)?;
    m.add_function(wrap_pyfunction!(fingerprints, m)?)?;
    m.add("FEATURE_NAMES", FEATURE_NAMES.to_vec())?;
    m.add_function(wrap_pyfunction!(collect, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate, m)?)?;
    m.add_function(wrap_pyfunction!(score, m)?)?;
    m.add_class::<BatchEnv>()?;
    Ok(())
}
