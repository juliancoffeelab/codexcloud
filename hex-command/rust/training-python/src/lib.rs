//! Python owns tensors; dedicated Rust worker threads own game lifetimes.
use hex_command_core::{data::*, simulation::*, training::*};
use numpy::ndarray::Array2;
use numpy::{IntoPyArray, PyArray1, PyArray2, PyReadonlyArray1, PyReadonlyArray2};
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
        .any(|b| b.planner == hex_command_core::ai::PlannerKind::Learned)
    {
        return Err("supply baseline blocks; the bridge selects the learned planner itself".into());
    }
    Ok(c)
}
fn load_rules() -> Result<Catalogue, ContentError> {
    match std::env::var_os("HEX_COMMAND_DATA_ROOT") {
        Some(root) => Catalogue::load_from(std::path::PathBuf::from(root)),
        None => Catalogue::load(),
    }
}
fn load_model(s: &str) -> Result<Model, String> {
    let mut m: Model = serde_json::from_str(s).map_err(|e| e.to_string())?;
    m.upgrade()?;
    Ok(m)
}
#[pyfunction]
fn upgrade_model(model: String) -> PyResult<String> {
    serde_json::to_string(&load_model(&model).map_err(err)?).map_err(err)
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
fn summary(r: &MatchResult, catalogue: &Catalogue) -> serde_json::Value {
    match r {
        MatchResult::Arena(r) => {
            serde_json::json!({"winner":r.winner,"reason":r.reason,"rounds":r.rounds,"success":r.winner==Some(BattleResult::PlayerVictory),"truncated":r.winner.is_none()})
        }
        MatchResult::Campaign(r) => {
            let definition = catalogue.campaign(&r.config.campaign);
            let failure = r.missions.last().and_then(|m| {
                if m.winner != Some(BattleResult::PlayerDefeat) {
                    return None;
                }
                let mission = definition
                    .chapters()
                    .iter()
                    .find(|v| v.map_id() == m.id)
                    .expect("reached mission");
                let evacuated = m
                    .r#final
                    .campaign
                    .as_ref()
                    .map(|c| c.evacuated.as_slice())
                    .unwrap_or_default();
                let alive = |u: &hex_command_core::data::PublicUnit| {
                    m.r#final.units.iter().any(|v| v.id == u.id)
                        || evacuated.iter().any(|v| v.unit.id == u.id)
                };
                // Inspect all reached rosters, including characters spawned by events.
                // This is terminal telemetry, never an input revealing hidden units.
                let seen = m.initial.units.iter().chain(
                    m.phases
                        .iter()
                        .flat_map(|p| p.before.iter().map(|u| u.as_ref())),
                );
                let lost: Vec<_> = seen
                    .filter(|u| {
                        !alive(u)
                            && u.character.as_ref().is_some_and(|id| {
                                mission.scripts().required_companions.contains(id)
                            })
                    })
                    .collect();
                if lost.iter().any(|u| u.role.is_some_and(Role::is_escort)) {
                    Some("escort_loss")
                } else if !lost.is_empty() {
                    Some("protected_loss")
                } else if !m
                    .r#final
                    .units
                    .iter()
                    .any(|u| u.faction == "player" && u.role == Some(Role::Leader))
                    && !evacuated
                        .iter()
                        .any(|v| v.unit.faction == "player" && v.unit.role == Some(Role::Leader))
                {
                    Some("leader_loss")
                } else {
                    Some("defeat")
                }
            });
            serde_json::json!({"success":r.completed,"failure":failure,"missions":r.missions.iter().map(|m|serde_json::json!({"id":m.id,"round":m.round,"winner":m.winner})).collect::<Vec<_>>(),"truncated":r.missions.last().is_some_and(|m|m.winner.is_none()),"outcome":r.missions.last().and_then(|m|m.r#final.campaign.as_ref()).and_then(|c|c.outcome.as_ref())})
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
fn collect_data(
    py: Python<'_>,
    configs: Vec<String>,
    max_samples: usize,
    side: Option<String>,
    spatial: bool,
    imitators: Option<Vec<String>>,
) -> PyResult<Dataset> {
    let side = match side.as_deref() {
        None => None,
        Some("player") => Some(TurnPhase::Player),
        Some("enemy") => Some(TurnPhase::Ai),
        _ => return Err(err("side must be player, enemy, or None")),
    };
    if max_samples == 0 {
        return Err(err("max_samples must be positive"));
    }
    if imitators.is_some() && side != Some(TurnPhase::Player) {
        return Err(err("pretrain_imitator collects the player side only"));
    }
    if imitators.as_ref().is_some_and(|x| x.len() != configs.len()) {
        return Err(err("one pretrain_imitator is required per config"));
    }
    py.detach(move || -> Result<Dataset, String> {
        let rules = load_rules().map_err(|e| e.to_string())?;
        let mut definitions = CampaignDefinitions::new(&rules);
        let mut data = Dataset {
            limit: max_samples,
            phase: side,
            spatial,
            ..Default::default()
        };
        for (i, config) in configs.into_iter().enumerate() {
            data.game = i;
            let mut config = parse(&config)?;
            let spec = imitators
                .as_ref()
                .map(|x| serde_json::from_str::<PretrainImitator>(&x[i]))
                .transpose()
                .map_err(|e| e.to_string())?;
            let controller = spec
                .as_ref()
                .map(|v| imitator::controller(v, &rules, &config))
                .transpose()?
                .flatten();
            if controller.is_some() {
                if !spatial {
                    return Err("targeted imitators require collect_spatial".into());
                }
                if side != Some(TurnPhase::Player) {
                    return Err("targeted imitators collect the player side only".into());
                }
                learned(&mut config);
            } else if let Some(value @ PretrainImitator::Blocks { .. }) = &spec {
                let blocks =
                    serde_json::from_value(serde_json::to_value(value).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                match &mut config {
                    MatchConfig::Arena(c) => c.player_blocks = Some(blocks),
                    MatchConfig::Campaign(c) => c.player_blocks = Some(blocks),
                }
            } else if let Some(PretrainImitator::Named(name)) = &spec {
                let planner = serde_json::from_value(serde_json::Value::String(name.clone()))
                    .map_err(|e| e.to_string())?;
                let mut blocks = match &config {
                    MatchConfig::Arena(c) => c.player_blocks.unwrap_or(c.player.blocks()),
                    MatchConfig::Campaign(c) => c.player_blocks.unwrap_or(c.player.blocks()),
                };
                blocks.planner = planner;
                match &mut config {
                    MatchConfig::Arena(c) => c.player_blocks = Some(blocks),
                    MatchConfig::Campaign(c) => c.player_blocks = Some(blocks),
                }
            }
            definitions
                .prepare_match(&config)
                .map_err(|e| e.to_string())?;
            let (result, next) = scoped(Some(data), None, controller, || {
                run(&rules, config, &definitions)
            });
            result.map_err(|e| e.to_string())?;
            data = next.unwrap();
        }
        Ok(data)
    })
    .map_err(err)
}
fn data_arrays(py: Python<'_>, data: Dataset) -> DataResult {
    (
        matrix(py, data.features),
        data.offsets.into_pyarray(py).unbind(),
        data.labels.into_pyarray(py).unbind(),
        data.missing,
    )
}
type SpatialResult = (
    Py<PyArray1<f32>>,
    Py<PyArray1<i64>>,
    Py<PyArray2<i64>>,
    Py<PyArray2<i64>>,
    Py<PyArray1<f32>>,
    Py<PyArray2<i64>>,
);
fn spatial_arrays(
    py: Python<'_>,
    boards: Vec<Option<&spatial::SpatialBoard>>,
    indices: Vec<[i64; 3]>,
) -> SpatialResult {
    let mut values = vec![];
    let mut offsets = vec![0i64];
    let mut shapes = vec![];
    let mut context_values = vec![];
    let mut contexts = vec![];
    for (i, board) in boards.iter().enumerate() {
        if let Some(b) = board {
            for c in &b.contexts {
                contexts.extend([i as i64, c.actor_site as i64, context_values.len() as i64]);
                context_values.extend_from_slice(&c.values);
            }
            values.extend_from_slice(&b.values);
            shapes.extend([b.height as i64, b.width as i64]);
        } else {
            shapes.extend([0, 0]);
        }
        offsets.push(values.len() as i64);
    }
    let count = indices.len();
    (
        values.into_pyarray(py).unbind(),
        offsets.into_pyarray(py).unbind(),
        Array2::from_shape_vec((boards.len(), 2), shapes)
            .unwrap()
            .into_pyarray(py)
            .unbind(),
        Array2::from_shape_vec((count, 3), indices.into_iter().flatten().collect())
            .unwrap()
            .into_pyarray(py)
            .unbind(),
        context_values.into_pyarray(py).unbind(),
        Array2::from_shape_vec((contexts.len() / 3, 3), contexts)
            .unwrap()
            .into_pyarray(py)
            .unbind(),
    )
}
#[pyfunction]
#[pyo3(signature=(configs,max_samples=100000,side=None,imitators=None))]
fn collect(
    py: Python<'_>,
    configs: Vec<String>,
    max_samples: usize,
    side: Option<String>,
    imitators: Option<Vec<String>>,
) -> PyResult<DataResult> {
    Ok(data_arrays(
        py,
        collect_data(py, configs, max_samples, side, false, imitators)?,
    ))
}
#[pyfunction]
#[pyo3(signature=(configs,max_samples=100000,side=None,imitators=None))]
fn collect_spatial(
    py: Python<'_>,
    configs: Vec<String>,
    max_samples: usize,
    side: Option<String>,
    imitators: Option<Vec<String>>,
) -> PyResult<(DataResult, SpatialResult)> {
    let mut data = collect_data(py, configs, max_samples, side, true, imitators)?;
    let spatial = spatial_arrays(
        py,
        data.boards.iter().map(Some).collect(),
        std::mem::take(&mut data.indices),
    );
    Ok((data_arrays(py, data), spatial))
}
/// Ordered labels with explicit match, observer and chapter/reset identity.
#[pyfunction]
#[pyo3(signature=(configs,max_samples=100000,side=None,imitators=None))]
fn collect_spatial_sequences(
    py: Python<'_>,
    configs: Vec<String>,
    max_samples: usize,
    side: Option<String>,
    imitators: Option<Vec<String>>,
) -> PyResult<(DataResult, SpatialResult, String)> {
    let mut data = collect_data(py, configs, max_samples, side, true, imitators)?;
    let contexts = serde_json::to_string(&data.decision_contexts).map_err(err)?;
    let spatial = spatial_arrays(
        py,
        data.boards.iter().map(Some).collect(),
        std::mem::take(&mut data.indices),
    );
    Ok((data_arrays(py, data), spatial, contexts))
}
#[pyfunction]
fn evaluate(py: Python<'_>, configs: Vec<String>, model: Option<String>) -> PyResult<Vec<String>> {
    let model = model.map(|s| load_model(&s)).transpose().map_err(err)?;
    py.detach(move || -> Result<Vec<String>, String> {
        let rules = load_rules().map_err(|e| e.to_string())?;
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
                        .map(|r| summary(&r, &rules).to_string())
                        .map_err(|e| e.to_string())
                })
                .collect::<Result<Vec<_>, String>>()
        });
        r
    })
    .map_err(err)
}
/// Profile intervals are native thread CPU and exclude fixture decoding/encoding.
#[pyfunction]
fn profile_snapshot() -> Option<String> {
    hex_command_core::ai::profiling::snapshot().map(|v| v.to_string())
}
#[pyfunction]
fn replay_phases(
    py: Python<'_>,
    requests: Vec<String>,
    model: Option<String>,
) -> PyResult<Vec<String>> {
    let model = model.map(|s| load_model(&s)).transpose().map_err(err)?;
    py.detach(move || -> Result<Vec<String>, String> {
        let rules = load_rules().map_err(|e| e.to_string())?;
        let (result, _) = scoped(None, model, None, || {
            requests
                .into_iter()
                .map(|request| {
                    hex_command_core::simulation::cli::saved_phase_json(&rules, &request)
                        .map(|v| v.to_string())
                        .map_err(|e| e.to_string())
                })
                .collect()
        });
        result
    })
    .map_err(err)
}
#[pyfunction]
fn score<'py>(
    py: Python<'py>,
    model: String,
    features: PyReadonlyArray2<'py, f32>,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let m = load_model(&model).map_err(err)?;
    if m.spatial.is_some() {
        return Err(err("use score_spatial for a CNN model"));
    }
    let a = features.as_array();
    if ![
        LEGACY_FEATURES,
        DUTY_FEATURES,
        DIPLOMACY_FEATURES,
        CAMPAIGN_FEATURES,
        VISIBILITY_FEATURES,
        IDLE_FEATURES,
        FEATURES,
    ]
    .contains(&a.ncols())
    {
        return Err(err("feature width mismatch"));
    }
    let features: Vec<[f32; FEATURES]> = a
        .rows()
        .into_iter()
        .map(|r| {
            let mut row = [0.; FEATURES];
            for (i, &v) in r.iter().enumerate() {
                row[i] = v;
            }
            row
        })
        .collect();
    let scores = Evaluator::new(m).map_err(err)?.scores(&features, None);
    Ok(scores.into_pyarray(py))
}
#[pyfunction]
#[pyo3(signature=(model,features,offsets,boards,board_offsets,shapes,indices,context_values=None,contexts=None))]
fn score_spatial<'py>(
    py: Python<'py>,
    model: String,
    features: PyReadonlyArray2<'py, f32>,
    offsets: PyReadonlyArray1<'py, i64>,
    boards: PyReadonlyArray1<'py, f32>,
    board_offsets: PyReadonlyArray1<'py, i64>,
    shapes: PyReadonlyArray2<'py, i64>,
    indices: PyReadonlyArray2<'py, i64>,
    context_values: Option<PyReadonlyArray1<'py, f32>>,
    contexts: Option<PyReadonlyArray2<'py, i64>>,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let model = load_model(&model).map_err(err)?;
    if model.spatial.is_none() {
        return Err(err("spatial model required"));
    }
    let x = features.as_array();
    let sh = shapes.as_array();
    let ix = indices.as_array();
    if ![
        LEGACY_FEATURES,
        DUTY_FEATURES,
        DIPLOMACY_FEATURES,
        CAMPAIGN_FEATURES,
        VISIBILITY_FEATURES,
        IDLE_FEATURES,
        FEATURES,
    ]
    .contains(&x.ncols())
        || ix.nrows() != x.nrows()
        || ix.ncols() != 3
        || sh.ncols() != 2
    {
        return Err(err("spatial array dimensions mismatch"));
    }
    let x: Vec<[f32; FEATURES]> = x
        .rows()
        .into_iter()
        .map(|row| {
            let mut values = [0.; FEATURES];
            for (i, &v) in row.iter().enumerate() {
                values[i] = v;
            }
            values
        })
        .collect();
    let ix: Vec<[i64; 3]> = ix
        .rows()
        .into_iter()
        .map(|row| row.to_vec().try_into().unwrap())
        .collect();
    let shapes: Vec<[i64; 2]> = sh
        .rows()
        .into_iter()
        .map(|row| row.to_vec().try_into().unwrap())
        .collect();
    let offsets = offsets.as_array().to_vec();
    let board_offsets = board_offsets.as_array().to_vec();
    let values = boards.as_array().to_vec();
    if context_values.is_some() != contexts.is_some() {
        return Err(err("both context arrays required"));
    }
    let cv = context_values
        .map(|v| v.as_array().to_vec())
        .unwrap_or_default();
    let mut context_rows = Vec::new();
    if let Some(contexts) = contexts {
        let a = contexts.as_array();
        if a.ncols() != 3 {
            return Err(err("invalid actor context metadata"));
        }
        let mut expected = 0;
        for row in a.rows() {
            if row[0] < 0 || row[0] as usize >= shapes.len() || row[1] < 0 || row[2] != expected {
                return Err(err("invalid actor context offset"));
            }
            let shape = shapes[row[0] as usize];
            if shape[0] <= 0 || shape[1] <= 0 || shape[0] > 256 || shape[1] > 256 {
                return Err(err("invalid actor context shape"));
            }
            expected += shape[0] * shape[1] * spatial::ACTOR_CHANNELS.len() as i64;
            context_rows.push([row[0], row[1], row[2]]);
        }
        if expected as usize != cv.len() {
            return Err(err("invalid actor context buffer"));
        }
    }
    for (boundaries, length) in [(&offsets, x.len()), (&board_offsets, values.len())] {
        if boundaries.len() != shapes.len() + 1
            || boundaries.first() != Some(&0)
            || boundaries.last() != Some(&(length as i64))
            || boundaries.windows(2).any(|v| v[0] < 0 || v[0] > v[1])
        {
            return Err(err("invalid spatial offsets"));
        }
    }
    let scores = py
        .detach(move || -> Result<Vec<f64>, String> {
            let mut evaluator = Evaluator::new(model)?;
            let mut result = vec![];
            for (i, shape) in shapes.iter().enumerate() {
                if *shape == [0, 0]
                    && offsets[i] == offsets[i + 1]
                    && board_offsets[i] == board_offsets[i + 1]
                {
                    continue;
                }
                if shape[0] <= 0 || shape[1] <= 0 || shape[0] > 256 || shape[1] > 256 {
                    return Err("invalid spatial shape".into());
                }
                let mut board = spatial::SpatialBoard {
                    height: shape[0] as usize,
                    width: shape[1] as usize,
                    contexts: vec![],
                    values: values[board_offsets[i] as usize..board_offsets[i + 1] as usize]
                        .to_vec(),
                };
                let cells = board.height * board.width;
                for row in context_rows.iter().filter(|r| r[0] == i as i64) {
                    let start = row[2] as usize;
                    board.contexts.push(spatial::ActorContext {
                        actor_site: row[1] as usize,
                        values: cv[start..start + cells * spatial::ACTOR_CHANNELS.len()].to_vec(),
                    });
                }
                if [
                    spatial::LEGACY_CHANNELS,
                    spatial::DUTY_CHANNELS,
                    spatial::CAMPAIGN_CHANNELS,
                    spatial::VISIBILITY_CHANNELS,
                    spatial::HOLD_CHANNELS,
                ]
                .iter()
                .any(|&channels| board.values.len() == cells * channels)
                {
                    board.values.resize(cells * spatial::CHANNELS, 0.);
                }
                board.validate()?;
                let a = offsets[i] as usize;
                let b = offsets[i + 1] as usize;
                board.validate_indices(&ix[a..b])?;
                result.extend(evaluator.scores(&x[a..b], Some((&board, &ix[a..b]))));
            }
            Ok(result)
        })
        .map_err(err)?;
    Ok(scores.into_pyarray(py))
}
#[pyfunction]
fn gru_step<'py>(
    py: Python<'py>,
    model: String,
    summaries: PyReadonlyArray2<'py, f32>,
    memory: PyReadonlyArray2<'py, f32>,
) -> PyResult<Bound<'py, PyArray2<f32>>> {
    let model = load_model(&model).map_err(err)?;
    let gru = model
        .recurrent
        .as_ref()
        .ok_or_else(|| err("recurrent model required"))?;
    let x = summaries.as_array();
    let h = memory.as_array();
    if x.ncols() != 16
        || h.ncols() != 64
        || x.nrows() != h.nrows()
        || x.iter().chain(h.iter()).any(|v| !v.is_finite())
    {
        return Err(err("invalid recurrent input dimensions or values"));
    }
    let rows = x.nrows();
    let output: Vec<f32> = x
        .rows()
        .into_iter()
        .zip(h.rows())
        .flat_map(|(x, h)| {
            gru.step(
                &x.to_vec().try_into().unwrap(),
                &h.to_vec().try_into().unwrap(),
            )
        })
        .collect();
    Ok(Array2::from_shape_vec((rows, 64), output)
        .unwrap()
        .into_pyarray(py))
}
enum RolloutEvent {
    Decision(Choices),
    Done(Result<String, String>),
}
struct Worker {
    tx: Option<Sender<usize>>,
    rx: std::sync::Mutex<Receiver<(RolloutEvent, usize)>>,
    join: Option<JoinHandle<()>>,
    event: Option<RolloutEvent>,
    chapters_cleared: usize,
}
impl Worker {
    fn new(config: String, spatial: bool) -> Self {
        let (tx, actions) = mpsc::channel();
        let (events, rx) = mpsc::channel();
        let join = std::thread::spawn(move || {
            let trace = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
            let chapters_cleared = std::rc::Rc::new(std::cell::Cell::new(0));
            let result = (|| -> Result<String, String> {
                let rules = load_rules().map_err(|e| e.to_string())?;
                let mut c = parse(&config)?;
                learned(&mut c);
                let mut definitions = CampaignDefinitions::new(&rules);
                definitions.prepare_match(&c).map_err(|e| e.to_string())?;
                let emit = events.clone();
                let chosen = trace.clone();
                let progress = chapters_cleared.clone();
                let controller: hex_command_core::training::DecisionController =
                    Box::new(move |choices| {
                        if emit
                            .send((RolloutEvent::Decision(choices.clone()), progress.get()))
                            .is_err()
                        {
                            return None;
                        }
                        let action = actions.recv().ok()?;
                        chosen.borrow_mut().push(action);
                        Some(action)
                    });
                let (result, _) = scoped(
                    spatial.then(|| Dataset {
                        spatial: true,
                        ..Default::default()
                    }),
                    None,
                    Some(controller),
                    || {
                        matches::run_with_chapter_clears(&rules, c, &definitions, || {
                            chapters_cleared.set(chapters_cleared.get() + 1);
                        })
                    },
                );
                result
                    .map(|r| summary(&r, &rules).to_string())
                    .map_err(|e| e.to_string())
            })();
            let result = result.map_err(|error| {
                serde_json::json!({"error":error,"config":serde_json::from_str::<serde_json::Value>(&config).ok(),"actions":*trace.borrow()}).to_string()
            });
            let _ = events.send((RolloutEvent::Done(result), chapters_cleared.get()));
        });
        Self {
            tx: Some(tx),
            rx: std::sync::Mutex::new(rx),
            join: Some(join),
            event: None,
            chapters_cleared: 0,
        }
    }
    fn receive(&mut self) -> Result<(), String> {
        let (event, chapters_cleared) =
            self.rx.lock().unwrap().recv().map_err(|e| e.to_string())?;
        self.event = Some(event);
        self.chapters_cleared = chapters_cleared;
        if let Some(RolloutEvent::Done(Err(e))) = &self.event {
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
    spatial: bool,
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
                RolloutEvent::Decision(c) => features.extend_from_slice(&c.features),
                RolloutEvent::Done(Ok(s)) => {
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
                RolloutEvent::Done(Err(e)) => return Err(err(e)),
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
    #[pyo3(signature=(spatial=false))]
    fn new(spatial: bool) -> Self {
        Self {
            workers: vec![],
            spatial,
        }
    }
    /// Final summaries only; live slots expose no privileged state.
    fn results(&self) -> Vec<Option<String>> {
        self.workers
            .iter()
            .map(|w| match &w.event {
                Some(RolloutEvent::Done(Ok(s))) => Some(s.clone()),
                _ => None,
            })
            .collect()
    }
    /// Cumulative intermediate chapter clears per slot, including after termination.
    /// Observer, map, round, committed decision counter, chapter generation.
    fn decision_contexts(&self) -> Vec<Option<(String, String, u32, u64, u64)>> {
        self.workers
            .iter()
            .map(|w| match &w.event {
                Some(RolloutEvent::Decision(c)) => Some((
                    c.context.observer.clone(),
                    c.context.map.clone(),
                    c.context.round,
                    c.context.decision,
                    c.context.generation,
                )),
                _ => None,
            })
            .collect()
    }
    fn chapters_cleared(&self) -> Vec<usize> {
        self.workers.iter().map(|w| w.chapters_cleared).collect()
    }
    fn spatial_observation(&self, py: Python<'_>) -> PyResult<SpatialResult> {
        if !self.spatial {
            return Err(err("construct BatchEnv(spatial=True)"));
        }
        let mut boards = vec![];
        let mut indices = vec![];
        for worker in &self.workers {
            if let Some(RolloutEvent::Decision(c)) = &worker.event {
                let s = c
                    .spatial
                    .as_ref()
                    .ok_or_else(|| err("missing spatial observation"))?;
                boards.push(Some(&s.board));
                indices.extend_from_slice(&s.indices);
            } else {
                boards.push(None);
            }
        }
        Ok(spatial_arrays(py, boards, indices))
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
            self.workers = configs
                .into_iter()
                .map(|c| Worker::new(c, self.spatial))
                .collect();
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
            if let Some(RolloutEvent::Decision(c)) = &w.event {
                if a >= c.commands.len() {
                    return Err(err("candidate index out of range"));
                }
            }
        }
        py.detach(|| -> Result<(), String> {
            // Dispatch the whole batch before waiting so native workers overlap.
            for (w, a) in self.workers.iter().zip(actions) {
                if matches!(w.event, Some(RolloutEvent::Decision(_))) {
                    w.tx.as_ref().unwrap().send(a).map_err(|e| e.to_string())?;
                }
            }
            for w in &mut self.workers {
                if matches!(w.event, Some(RolloutEvent::Decision(_))) {
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
fn training_workload(py: Python<'_>) -> PyResult<String> {
    py.detach(|| {
        let rules = load_rules().map_err(err)?;
        serde_json::to_string(&rules.meta().simulation.training).map_err(err)
    })
}
#[pyfunction]
fn fingerprints(py: Python<'_>) -> PyResult<String> {
    py.detach(|| {
        let rules = load_rules().map_err(err)?;
        Ok(hex_command_core::simulation::cli::fingerprints(&rules).to_string())
    })
}
#[pymodule]
fn hex_command_training(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gru_step, m)?)?;
    m.add("SCHEMA", SCHEMA)?;
    m.add("SPATIAL_SCHEMA", spatial::SPATIAL_SCHEMA)?;
    m.add("SPATIAL_CHANNEL_NAMES", spatial::channel_names())?;
    m.add(
        "HOLD_SPATIAL_CHANNEL_NAMES",
        spatial::channel_names()[..spatial::HOLD_CHANNELS].to_vec(),
    )?;
    m.add(
        "VISIBILITY_SPATIAL_CHANNEL_NAMES",
        spatial::channel_names()[..spatial::VISIBILITY_CHANNELS].to_vec(),
    )?;
    m.add("SPATIAL_KERNEL", spatial::KERNEL.to_vec())?;
    m.add_function(wrap_pyfunction!(fingerprints, m)?)?;
    m.add_function(wrap_pyfunction!(training_workload, m)?)?;
    m.add("FEATURE_NAMES", FEATURE_NAMES.to_vec())?;
    m.add(
        "IDLE_FEATURE_NAMES",
        FEATURE_NAMES[..IDLE_FEATURES].to_vec(),
    )?;
    m.add(
        "VISIBILITY_FEATURE_NAMES",
        FEATURE_NAMES[..VISIBILITY_FEATURES].to_vec(),
    )?;
    m.add(
        "CAMPAIGN_FEATURE_NAMES",
        FEATURE_NAMES[..CAMPAIGN_FEATURES].to_vec(),
    )?;
    m.add(
        "CAMPAIGN_SPATIAL_CHANNEL_NAMES",
        spatial::channel_names()[..spatial::CAMPAIGN_CHANNELS].to_vec(),
    )?;
    m.add("ACTOR_SPATIAL_CHANNELS", spatial::ACTOR_CHANNELS.to_vec())?;
    m.add(
        "DIPLOMACY_FEATURE_NAMES",
        FEATURE_NAMES[..DIPLOMACY_FEATURES].to_vec(),
    )?;
    m.add(
        "LEGACY_FEATURE_NAMES",
        FEATURE_NAMES[..LEGACY_FEATURES].to_vec(),
    )?;
    m.add(
        "LEGACY_SPATIAL_CHANNEL_NAMES",
        spatial::channel_names()[..spatial::LEGACY_CHANNELS].to_vec(),
    )?;
    m.add(
        "DUTY_FEATURE_NAMES",
        FEATURE_NAMES[..DUTY_FEATURES].to_vec(),
    )?;
    m.add(
        "DUTY_SPATIAL_CHANNEL_NAMES",
        spatial::channel_names()[..spatial::DUTY_CHANNELS].to_vec(),
    )?;
    m.add_function(wrap_pyfunction!(upgrade_model, m)?)?;
    m.add_function(wrap_pyfunction!(collect, m)?)?;
    m.add_function(wrap_pyfunction!(collect_spatial, m)?)?;
    m.add_function(wrap_pyfunction!(collect_spatial_sequences, m)?)?;
    m.add_function(wrap_pyfunction!(score_spatial, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate, m)?)?;
    m.add_function(wrap_pyfunction!(score, m)?)?;
    m.add_function(wrap_pyfunction!(profile_snapshot, m)?)?;
    m.add_function(wrap_pyfunction!(replay_phases, m)?)?;
    m.add_class::<BatchEnv>()?;
    Ok(())
}
