use std::{collections::HashSet, path::Path};

use tspf::TspBuilder;

use crate::{
    alg::{cand_gen_nn, solvers::solve_greedy, SolverKind},
    data::{DataStore, Metric, NodeIndex, NodeKind},
    tour::{TourOrder, TwoLevelList},
};

#[derive(Debug)]
// Note: single depot only.
pub struct Model<M> {
    complete: bool,
    groupsize: usize,
    store: DataStore<M>,
    depots: HashSet<usize>,
    tours: Vec<TwoLevelList>,
}

impl<M> Model<M> {
    // new(metric, conf);
    pub fn new(metric: Metric, groupsize: usize) -> Self {
        Self {
            complete: false,
            groupsize,
            store: DataStore::new(metric),
            depots: HashSet::new(),
            tours: Vec::with_capacity(0),
        }
    }

    pub fn with_capacity(
        metric: Metric,
        groupsize: usize,
        cap_depots: usize,
        cap_nodes: usize,
    ) -> Self {
        Self {
            complete: false,
            groupsize,
            store: DataStore::with_capacity(metric, cap_nodes),
            depots: HashSet::with_capacity(cap_depots),
            tours: Vec::with_capacity(0),
        }
    }

    #[inline]
    pub fn n_depots(&self) -> usize {
        self.depots.len()
    }

    #[inline]
    pub fn n_nodes(&self) -> usize {
        self.store.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    #[inline]
    pub fn complete(&mut self) {
        if !self.complete {
            self.complete = true;
            self.store.compute();
        }
    }

    pub fn add(&mut self, kind: NodeKind, pos: Vec<f64>, meta: M) -> Option<NodeIndex> {
        if self.complete {
            None
        } else {
            let node = self.store.add(kind, pos, meta);
            if let Some(x) = &node {
                if kind == NodeKind::Depot {
                    self.depots.insert(x.index());
                }
            }
            node
        }
    }

    // TODO: should return status and/or result.
    pub fn solve(&mut self, config: &RunConfig) -> Option<TourOrder> {
        self.complete();

        let mut tour = TwoLevelList::new(&self.store, self.groupsize);
        cand_gen_nn(&mut tour, config.cands);

        let result = match config.solver {
            SolverKind::Greedy(ref starters) => solve_greedy(&mut tour, starters),
        };

        result.unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RunConfig {
    cands: usize,
    solver: SolverKind,
}

#[derive(Debug, Default)]
pub struct RunConfigBuilder {
    cands: Option<usize>,
    solver: Option<SolverKind>,
}

impl RunConfigBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn cands(mut self, cands: usize) -> Self {
        self.cands = Some(cands);
        self
    }

    pub fn solver(mut self, solver: SolverKind) -> Self {
        self.solver = Some(solver);
        self
    }

    pub fn build(self) -> RunConfig {
        RunConfig {
            cands: self.cands.unwrap_or(10),
            solver: self.solver.unwrap_or_else(|| SolverKind::Greedy(vec![0])),
        }
    }
}

pub fn load_tsp<P>(path: P, groupsize: usize) -> Model<usize>
where
    P: AsRef<Path>,
{
    // TODO: catch err.
    let mut tsp = TspBuilder::parse_path(path).unwrap();

    let metric = match tsp.weight_kind() {
        tspf::WeightKind::Explicit => Metric::Explicit,
        tspf::WeightKind::Euc2d => Metric::Euc2d,
        tspf::WeightKind::Euc3d => Metric::Euc3d,
        tspf::WeightKind::Max2d => Metric::Max2d,
        tspf::WeightKind::Max3d => Metric::Max3d,
        tspf::WeightKind::Man2d => Metric::Man2d,
        tspf::WeightKind::Man3d => Metric::Man3d,
        tspf::WeightKind::Ceil2d => Metric::Ceil2d,
        tspf::WeightKind::Geo => Metric::Geo,
        tspf::WeightKind::Att => Metric::Att,
        tspf::WeightKind::Xray1 => Metric::Xray1,
        tspf::WeightKind::Xray2 => Metric::Xray2,
        tspf::WeightKind::Custom => Metric::Custom,
        tspf::WeightKind::Undefined => Metric::Undefined,
    };

    let n_nodes = tsp.dim();
    let n_depots = tsp.depots().len();

    let mut model = Model::<usize>::with_capacity(metric, groupsize, n_depots, n_nodes);
    {
        let nc = std::mem::take(tsp.node_coords_mut());
        for (_, pt) in nc {
            let (idx, v) = pt.into_value();

            let kind = if tsp.depots().contains(&idx) {
                NodeKind::Depot
            } else {
                NodeKind::Target
            };

            model.add(kind, v, idx);
        }
    }

    model.complete();

    model
}
