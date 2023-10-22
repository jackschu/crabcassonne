use core::panic;
use itertools::Itertools;
use rayon::prelude::*;
use std::todo;

use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    arena::Match,
    board::BoardData,
    referee::{Player, RefereeState},
    tile::TileData,
    tilebag::{self, TileBag},
};

use super::bot::{Bot, MoveRequest};

pub struct MCTSBot {
    pub own_player: Player,
    rng: StdRng,
    depth: u32,
}

impl MCTSBot {
    pub fn new(player: Player, depth: u32) -> Self {
        MCTSBot {
            rng: StdRng::seed_from_u64(rand::random()),
            own_player: player,
            depth,
        }
    }
}

impl Bot for MCTSBot {
    fn get_name(&self) -> String {
        format!("MCTS bot {}", self.depth)
    }

    fn get_own_player(&self) -> &Player {
        &self.own_player
    }

    fn get_move(&mut self, state: &RefereeState) -> MoveRequest {
        let mut arena = ArenaTree::new(state);
        for _i in 0..self.depth {
            //            println!("arena iter {_i}");
            //            arena.debug_data();
            arena.mcts_iter();
            // if arena.rollouts > self.depth {
            //     break;
            // }
        }
        println!("{} used rollouts {}", self.get_name(), arena.rollouts);

        arena.recommend()
    }
}

enum Edge {
    Placement(MoveRequest),
    Draw(Vec<TileData>),
}

struct NodeData {
    visited: u64,
    player: Player,
    reward: f64,
    entry: Edge,
}

struct ArenaTree<'a> {
    pub rollouts: u32,
    arena: Vec<Node>,
    rng: StdRng,
    state: &'a RefereeState,
}

struct Node {
    idx: usize,
    data: NodeData,
    parent: Option<usize>,
    children: Vec<usize>,
}

impl<'a> ArenaTree<'a> {
    pub fn debug_data(&self) {
        println!(
            "own player {} child ct {}",
            self.arena[0].data.player,
            self.arena.len()
        );
    }

    pub fn new(state: &'a RefereeState) -> Self {
        Self {
            rollouts: 0,
            state,
            rng: StdRng::seed_from_u64(rand::random()),
            arena: vec![Node {
                idx: 0,
                data: NodeData {
                    visited: 0,
                    player: state.get_player().clone(),
                    reward: 0.0,
                    entry: Edge::Draw(vec![state.tilebag.peek().unwrap().clone()]),
                },
                parent: None,
                children: vec![],
            }],
        }
    }
    pub fn recommend(&self) -> MoveRequest {
        let max_visited = self.arena[0]
            .children
            .iter()
            .max_by(|l, r| {
                let lv = self.arena[*l.to_owned()].data.visited;
                let rv = self.arena[*r.to_owned()].data.visited;
                lv.partial_cmp(&rv).unwrap()
            })
            .unwrap();
        if let Edge::Placement(out) = &self.arena[*max_visited].data.entry {
            out.clone()
        } else {
            panic!()
        }
    }

    pub fn mcts_iter(&mut self) {
        let expansion_idx = self.selection();
        // let mut path = self.path_to_root(expansion_idx);
        // path.reverse();
        // let mut depth = 0;
        // for elem in path {
        //     let node = &self.arena[elem];
        //     let entry = match node.data.entry {
        //         Edge::Draw(_) => "Drew",
        //         Edge::Placement(_) => "Placed",
        //     };
        //     println!(
        //         "path to expansion {depth}, reward {} visited {}, player {}, action {entry}",
        //         node.data.reward, node.data.visited, node.data.player,
        //     );
        //     depth += 1;
        // }

        let new_nodes = self.expansion(expansion_idx);
        let results: Vec<(usize, f64)> = new_nodes
            .into_par_iter()
            .map(|idx| (idx, self.simulation(idx)))
            .collect();

        self.rollouts += results.len() as u32;
        for result in results {
            self.back_prop(result.0, result.1);
        }
    }

    fn state_at(&self, node_idx: usize) -> RefereeState {
        let mut path = self.path_to_root(node_idx);
        path.reverse();
        let mut out = self.state.clone();
        for idx in path {
            let node = &self.arena[idx];
            match &node.data.entry {
                Edge::Draw(rig) => out.tilebag.rig(rig.clone()),
                Edge::Placement(request) => out.process_move(request.clone()).unwrap(),
            }
        }
        return out;
    }
    fn path_to_root(&self, node_idx: usize) -> Vec<usize> {
        let mut out = vec![node_idx];
        let mut cur_idx = node_idx;
        while let Some(next_idx) = self.arena[cur_idx].parent {
            cur_idx = next_idx;
            out.push(cur_idx);
        }
        return out;
    }

    fn back_prop(&mut self, start_idx: usize, reward: f64) {
        let path = self.path_to_root(start_idx);

        let root_player = self.arena[0].data.player.clone();
        for idx in path {
            let node = &mut self.arena[idx];
            if node.data.player != root_player {
                node.data.reward -= reward;
            } else {
                node.data.reward += reward;
            }
            node.data.visited += 1;
        }
    }

    fn insert_children(&mut self, children: Vec<NodeData>, parent_idx: usize) -> Vec<usize> {
        self.arena.reserve(children.len());

        let mut prev_len = self.arena.len();

        let mut out = vec![];
        for child in children {
            self.arena.push(Node {
                idx: prev_len,
                data: child,
                parent: Some(parent_idx),
                children: vec![],
            });
            out.push(prev_len);
            prev_len += 1;
        }

        let parent = &mut self.arena[parent_idx];
        parent.children.append(&mut out.clone());
        return out;
    }

    fn expansion(&mut self, idx: usize) -> Vec<usize> {
        let cur = &self.arena[idx];
        let state = self.state_at(idx);
        let edges: Vec<Edge> = match cur.data.entry {
            Edge::Draw(_) => {
                let moves = state.get_legal_moves();
                moves
                    .into_iter()
                    .map(|request| Edge::Placement(request))
                    .collect()
            }
            Edge::Placement(_) => {
                let remaining_tiles = state.tilebag.get_data().clone();

                let mut legals = vec![];
                let mut illegals = vec![];
                for tile in remaining_tiles {
                    if state.board.as_overlay().does_legal_move_exist(&tile) {
                        legals.push(tile);
                    } else {
                        illegals.push(tile);
                    }
                }

                let mut out = vec![];
                for i in 0..(illegals.len() + 1) {
                    illegals
                        .iter()
                        .combinations(i)
                        .cartesian_product(&legals)
                        .for_each(|(illegal, legal)| {
                            let mut data: Vec<TileData> = illegal.into_iter().cloned().collect();
                            data.push(legal.clone());
                            out.push(Edge::Draw(data));
                        })
                }
                out
            }
        };

        let player = state.get_player();

        let data: Vec<NodeData> = edges
            .into_iter()
            .map(|edge| NodeData {
                player: player.clone(),
                entry: edge,
                reward: 0.0,
                visited: 0,
            })
            .collect();
        self.insert_children(data, idx)
    }

    fn simulation(&self, idx: usize) -> f64 {
        let mut out: f64 = 0.0;
        let state = self.state_at(idx);
        let own_player = self.arena[0].data.player.clone();
        let result = Match::play_random_from_state(state).unwrap();
        for (player, points) in result.player_scores {
            if player == own_player {
                out += points as f64;
            } else {
                out -= points as f64;
            }
        }
        out
    }

    fn selection(&mut self) -> usize {
        let mut cur_idx = 0;
        let mut parent_visited = 0;
        loop {
            let cur = &self.arena[cur_idx];
            let next_idx = match cur.data.entry {
                Edge::Draw(_) => self.max_ucb_idx(cur_idx, parent_visited),
                Edge::Placement(_) => {
                    let n = cur.children.len();
                    if n == 0 {
                        None
                    } else {
                        let idx = self.rng.gen_range(0..n);
                        Some(cur.children[idx])
                    }
                }
            };
            parent_visited = self.arena[cur_idx].data.visited;
            if let Some(next_idx) = next_idx {
                cur_idx = next_idx;
            } else {
                return cur_idx;
            }
        }
    }

    fn max_ucb_idx(&self, start_idx: usize, parent_visited: u64) -> Option<usize> {
        self.arena[start_idx]
            .children
            .iter()
            .map(|c| (c, self.arena[*c].ucb(parent_visited)))
            .max_by(|l, r| l.1.partial_cmp(&r.1).unwrap())
            .map(|x| x.0)
            .copied()
    }
}

impl Node {
    fn ucb(&self, parent_visited: u64) -> f64 {
        let c = 2.0;

        let ev = self.data.average_reward();
        if parent_visited == 0 {
            return ev;
        }

        let own_visits = self.data.visited as f64;
        let parent_visits = parent_visited as f64;
        let explore = (parent_visits.ln() / own_visits).sqrt();
        return ev + c * explore;
    }
}
impl NodeData {
    pub fn average_reward(&self) -> f64 {
        if self.visited != 0 {
            (self.reward as f64) / (self.visited as f64)
        } else {
            0.0
        }
    }
}
