use crate::helpers::*;
use crate::json::problem::*;
use crate::json::solution::*;

#[test]
fn can_use_break_between_two_jobs_in_relation() {
    let relation_type = RelationType::Flexible;

    let problem = Problem {
        id: "my_problem".to_string(),
        plan: Plan {
            jobs: vec![create_delivery_job("job1", vec![1., 0.]), create_delivery_job("job2", vec![2., 0.])],
            relations: Some(vec![Relation {
                type_field: relation_type,
                jobs: vec!["job1".to_string(), "break".to_string(), "job2".to_string()],
                vehicle_id: "my_vehicle_1".to_string(),
            }]),
        },
        fleet: Fleet {
            types: vec![VehicleType {
                id: "my_vehicle".to_string(),
                profile: "car".to_string(),
                costs: create_default_vehicle_costs(),
                places: create_default_vehicle_places(),
                capacity: vec![10],
                amount: 1,
                skills: None,
                limits: None,
                vehicle_break: Some(VehicleBreak {
                    times: vec![vec![format_time(0), format_time(1000)]],
                    duration: 2.0,
                    location: Some(vec![3., 0.]),
                }),
            }],
        },
    };
    let matrix = Matrix {
        num_origins: 4,
        num_destinations: 4,
        travel_times: vec![0, 1, 1, 2, 1, 0, 2, 1, 1, 2, 0, 3, 2, 1, 3, 0],
        distances: vec![0, 1, 1, 2, 1, 0, 2, 1, 1, 2, 0, 3, 2, 1, 3, 0],
        error_codes: Option::None,
    };

    let solution = solve_with_metaheuristic(problem, vec![matrix]);

    assert_eq!(
        solution,
        Solution {
            problem_id: "my_problem".to_string(),
            statistic: Statistic {
                cost: 26.,
                distance: 6,
                duration: 10,
                times: Timing { driving: 6, serving: 2, waiting: 0, break_time: 2 },
            },
            tours: vec![Tour {
                vehicle_id: "my_vehicle_1".to_string(),
                type_id: "my_vehicle".to_string(),
                stops: vec![
                    create_stop_with_activity(
                        "departure",
                        "departure",
                        (0., 0.),
                        2,
                        ("1970-01-01T00:00:00Z", "1970-01-01T00:00:00Z"),
                    ),
                    create_stop_with_activity(
                        "job1",
                        "delivery",
                        (1., 0.),
                        1,
                        ("1970-01-01T00:00:01Z", "1970-01-01T00:00:02Z"),
                    ),
                    create_stop_with_activity(
                        "break",
                        "break",
                        (3., 0.),
                        1,
                        ("1970-01-01T00:00:03Z", "1970-01-01T00:00:05Z"),
                    ),
                    create_stop_with_activity(
                        "job2",
                        "delivery",
                        (2., 0.),
                        0,
                        ("1970-01-01T00:00:06Z", "1970-01-01T00:00:07Z"),
                    ),
                    create_stop_with_activity(
                        "arrival",
                        "arrival",
                        (0., 0.),
                        0,
                        ("1970-01-01T00:00:09Z", "1970-01-01T00:00:09Z"),
                    )
                ],
                statistic: Statistic {
                    cost: 26.,
                    distance: 6,
                    duration: 10,
                    times: Timing { driving: 6, serving: 2, waiting: 0, break_time: 2 },
                },
            }],
            unassigned: vec![],
            extras: Extras { performance: vec![] },
        }
    );
}
