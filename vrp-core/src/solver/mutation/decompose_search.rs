#[cfg(test)]
#[path = "../../../tests/unit/solver/mutation/decompose_search_test.rs"]
mod decompose_search_test;

use crate::construction::heuristics::{get_medoid, InsertionContext, SolutionContext};
use crate::solver::mutation::Mutation;
use crate::solver::population::{Greedy, Individual, Population};
use crate::solver::RefinementContext;
use crate::utils::{compare_floats, parallel_into_collect, Random};
use hashbrown::HashSet;
use std::cmp::Ordering;
use std::iter::{empty, once};
use std::sync::{Arc, RwLock};

/// A mutation which decomposes original solution into multiple partial solutions,
/// preforms search independently, and then merges partial solution back into one solution.
pub struct DecomposeSearch {
    inner_mutation: Arc<dyn Mutation + Send + Sync>,
    // TODO different repeat count depending on generation in refinement ctx
    repeat_count: usize,
}

impl DecomposeSearch {
    /// Create a new instance of `DecomposeSearch`.
    pub fn new(inner_mutation: Arc<dyn Mutation + Send + Sync>, repeat_count: usize) -> Self {
        Self { inner_mutation, repeat_count }
    }
}

impl Mutation for DecomposeSearch {
    fn mutate_one(&self, refinement_ctx: &RefinementContext, insertion_ctx: &InsertionContext) -> InsertionContext {
        refinement_ctx
            .population
            .ranked()
            .next()
            .and_then(|(individual, _)| {
                decompose_individual(&refinement_ctx, individual).map(|result| (individual.random.clone(), result))
            })
            .map(|(random, decomposed_contexts)| self.refine_decomposed(refinement_ctx, random, decomposed_contexts))
            .unwrap_or_else(|| self.inner_mutation.mutate_one(refinement_ctx, insertion_ctx))
    }

    fn mutate_all(
        &self,
        refinement_ctx: &RefinementContext,
        individuals: Vec<&InsertionContext>,
    ) -> Vec<InsertionContext> {
        individuals.into_iter().map(|individual| self.mutate_one(refinement_ctx, individual)).collect()
    }
}

const GREEDY_ERROR: &str = "greedy population has no individuals";

impl DecomposeSearch {
    fn refine_decomposed(
        &self,
        refinement_ctx: &RefinementContext,
        random: Arc<dyn Random + Send + Sync>,
        decomposed_contexts: Vec<RefinementContext>,
    ) -> Individual {
        // do actual refinement independently for each decomposed context
        let decomposed_populations = parallel_into_collect(decomposed_contexts, |mut decomposed_ctx| {
            (0..self.repeat_count).for_each(|_| {
                let insertion_ctx = decomposed_ctx.population.select().next().expect(GREEDY_ERROR);
                let insertion_ctx = self.inner_mutation.mutate_one(&decomposed_ctx, insertion_ctx);
                decomposed_ctx.population.add(insertion_ctx);
            });
            decomposed_ctx.population
        });

        // merge evolution results into one individual
        let mut individual = decomposed_populations.into_iter().fold(
            Individual::new(refinement_ctx.problem.clone(), random),
            |mut individual, decomposed_population| {
                let (decomposed_individual, _) = decomposed_population.ranked().next().expect(GREEDY_ERROR);

                let acc_solution = &mut individual.solution;
                let dec_solution = &decomposed_individual.solution;

                // NOTE theoretically, we can avoid deep copy here, but this would require extension in Population trait
                acc_solution.routes.extend(dec_solution.routes.iter().map(|route_ctx| route_ctx.deep_copy()));
                acc_solution.ignored.extend(dec_solution.ignored.iter().cloned());
                acc_solution.required.extend(dec_solution.required.iter().cloned());
                acc_solution.locked.extend(dec_solution.locked.iter().cloned());
                acc_solution.unassigned.extend(dec_solution.unassigned.iter().map(|(k, v)| (k.clone(), v.clone())));

                dec_solution.routes.iter().for_each(|route_ctx| {
                    acc_solution.registry.use_route(route_ctx);
                });

                individual
            },
        );

        refinement_ctx.problem.constraint.accept_solution_state(&mut individual.solution);

        individual
    }
}

fn create_population(individual: Individual) -> Box<dyn Population + Send + Sync> {
    Box::new(Greedy::new(individual.problem.clone(), Some(individual)))
}

fn create_multiple_individuals(individual: &Individual) -> Option<Vec<Individual>> {
    // TODO limit by max amount of jobs (cannot be less than 2)
    const MAX_ROUTES_PER_INDIVIDUAL: usize = 3;

    let solution = &individual.solution;
    let profile = solution.routes.first().map(|route_ctx| route_ctx.route.actor.vehicle.profile)?;
    let transport = individual.problem.transport.as_ref();

    let indexed_medoids = solution
        .routes
        .iter()
        .enumerate()
        .map(|(idx, route_ctx)| (idx, get_medoid(route_ctx, transport)))
        .collect::<Vec<_>>();

    // estimate distances between all routes using their medoids
    let route_groups_distances = indexed_medoids
        .iter()
        .map(|(outer_idx, outer_medoid)| {
            let mut route_distances = indexed_medoids
                .iter()
                .filter(move |(inner_idx, _)| *outer_idx != *inner_idx)
                .map(move |(inner_idx, inner_medoid)| {
                    let distance = match (outer_medoid, inner_medoid) {
                        (Some(outer_medoid), Some(inner_medoid)) => {
                            let distance =
                                transport.distance(profile, *outer_medoid, *inner_medoid, Default::default());
                            if distance < 0. {
                                None
                            } else {
                                Some(distance)
                            }
                        }
                        _ => None,
                    };
                    (inner_idx, distance)
                })
                .collect::<Vec<_>>();

            route_distances.sort_by(|(_, a_distance), (_, b_distance)| match (a_distance, b_distance) {
                (Some(a_distance), Some(b_distance)) => compare_floats(*a_distance, *b_distance),
                (Some(_), None) => Ordering::Less,
                _ => Ordering::Greater,
            });

            route_distances
        })
        .collect::<Vec<_>>();

    // identify route groups and create individuals from them
    let used_indices = RwLock::new(HashSet::new());
    let individuals = route_groups_distances
        .iter()
        .enumerate()
        .filter(|(outer_idx, _)| !used_indices.read().unwrap().contains(outer_idx))
        .map(|(outer_idx, route_group_distance)| {
            let route_group = route_group_distance
                .iter()
                .cloned()
                .filter(|(inner_idx, _)| !used_indices.read().unwrap().contains(*inner_idx))
                .take((MAX_ROUTES_PER_INDIVIDUAL - 1).max(1))
                .map(|(inner_idx, _)| *inner_idx)
                .chain(once(outer_idx))
                .collect::<HashSet<_>>();

            route_group.iter().for_each(|idx| {
                debug_assert!(used_indices.write().unwrap().insert(*idx));
            });

            create_partial_individual(individual, route_group.iter().cloned())
        })
        .chain(create_empty_individuals(individual))
        .collect();

    Some(individuals)
}

fn create_partial_individual(individual: &Individual, route_indices: impl Iterator<Item = usize>) -> Individual {
    let routes = route_indices.map(|idx| individual.solution.routes[idx].deep_copy()).collect::<Vec<_>>();
    let actors = routes.iter().map(|route_ctx| route_ctx.route.actor.clone()).collect::<HashSet<_>>();
    let registry = individual.solution.registry.deep_slice(|actor| actors.contains(actor));
    let jobs = routes.iter().flat_map(|route_ctx| route_ctx.route.tour.jobs()).collect::<HashSet<_>>();
    let locked = individual.solution.locked.iter().filter(|job| jobs.contains(job)).cloned().collect();

    // TODO it would be nice to fill ignored jobs with actor specific jobs
    Individual {
        problem: individual.problem.clone(),
        solution: SolutionContext {
            required: Default::default(),
            ignored: Default::default(),
            unassigned: Default::default(),
            locked,
            routes,
            registry,
            state: Default::default(),
        },
        random: individual.random.clone(),
    }
}

fn create_empty_individuals(individual: &Individual) -> Box<dyn Iterator<Item = Individual>> {
    // TODO split into more individuals if too many required jobs are present
    //      this might increase overall refinement speed

    if individual.solution.required.is_empty() && individual.solution.unassigned.is_empty() {
        return Box::new(empty());
    } else {
        Box::new(once(Individual {
            problem: individual.problem.clone(),
            solution: SolutionContext {
                required: individual.solution.required.clone(),
                ignored: individual.solution.ignored.clone(),
                unassigned: individual.solution.unassigned.clone(),
                locked: individual.solution.locked.clone(),
                routes: Default::default(),
                registry: individual.solution.registry.deep_copy(),
                state: Default::default(),
            },
            random: individual.random.clone(),
        }))
    }
}

fn decompose_individual(refinement_ctx: &RefinementContext, individual: &Individual) -> Option<Vec<RefinementContext>> {
    create_multiple_individuals(individual)
        .map(|individuals| {
            individuals
                .into_iter()
                .map(|individual| RefinementContext {
                    problem: refinement_ctx.problem.clone(),
                    population: create_population(individual),
                    state: Default::default(),
                    quota: refinement_ctx.quota.clone(),
                    statistics: Default::default(),
                })
                .collect::<Vec<_>>()
        })
        .and_then(|contexts| if contexts.len() > 1 { Some(contexts) } else { None })
}
