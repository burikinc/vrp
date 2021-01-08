use crate::construction::constraints::{ConstraintPipeline, TransportConstraintModule};
use crate::models::common::{Dimensions, IdDimension, TimeInterval, TimeSpan, TimeWindow};
use crate::models::problem::*;
use crate::models::Problem;
use std::sync::Arc;

struct ExampleTransportCost {}

impl TransportCost for ExampleTransportCost {
    fn duration(&self, _: i32, _: usize, _: usize, _: f64) -> f64 {
        42.
    }

    fn distance(&self, _: i32, _: usize, _: usize, _: f64) -> f64 {
        42.
    }
}

/// Creates an example jobs used in documentation tests.
fn create_example_jobs(fleet: &Fleet, transport: &Arc<dyn TransportCost + Sync + Send>) -> Arc<Jobs> {
    Arc::new(Jobs::new(
        fleet,
        vec![Job::Single(Arc::new(Single {
            places: vec![Place {
                location: Some(1),
                duration: 0.0,
                times: vec![TimeSpan::Window(TimeWindow::new(0., 100.))],
            }],
            dimens: Default::default(),
        }))],
        &transport,
    ))
}

/// Creates an example fleet used in documentation tests.
fn create_example_fleet() -> Arc<Fleet> {
    let drivers = vec![Arc::new(Driver {
        costs: Costs { fixed: 0., per_distance: 0., per_driving_time: 0., per_waiting_time: 0., per_service_time: 0. },
        dimens: Default::default(),
        details: vec![],
    })];
    let mut vehicle_dimens = Dimensions::new();
    vehicle_dimens.set_id("v1");
    let vehicles = vec![Arc::new(Vehicle {
        profile: 0,
        costs: Costs { fixed: 0., per_distance: 1., per_driving_time: 0., per_waiting_time: 0., per_service_time: 0. },
        dimens: vehicle_dimens,
        details: vec![VehicleDetail {
            start: Some(VehiclePlace { location: 0, time: TimeInterval::default() }),
            end: None,
        }],
    })];

    Arc::new(Fleet::new(drivers, vehicles, Box::new(|_| Box::new(|_| 0))))
}

/// Creates an example problem used in documentation tests.
pub fn create_example_problem() -> Arc<Problem> {
    let activity: Arc<dyn ActivityCost + Sync + Send> = Arc::new(SimpleActivityCost::default());
    let transport: Arc<dyn TransportCost + Sync + Send> = Arc::new(ExampleTransportCost {});
    let fleet = create_example_fleet();
    let jobs = create_example_jobs(&fleet, &transport);
    let mut constraint = ConstraintPipeline::default();
    constraint.add_module(Box::new(TransportConstraintModule::new(
        activity.clone(),
        transport.clone(),
        Arc::new(|_| (None, None)),
        1,
        2,
        3,
    )));

    Arc::new(Problem {
        fleet,
        jobs,
        locks: vec![],
        constraint: Arc::new(constraint),
        activity,
        transport,
        objective: Arc::new(ObjectiveCost::default()),
        extras: Arc::new(Default::default()),
    })
}
