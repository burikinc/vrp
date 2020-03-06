# Error Index

## E1xxx: Validation errors

Errors with E1xxx mask are used by validation engine which checks logical correctness of the VRP definition.


### E1000

This error is returned when `plan.jobs` has jobs with the same ids.

```json
{
  "plan": {
    "jobs": [
      {
        "id": "job1",
        /** omitted **/
      },
      {
        /** Error: this id is already used by another job **/
        "id": "job1",
        /** omitted **/
      }
      /** omitted **/
    ]
  }
}
```


### E1001

This error code is returned when job has both pickups and deliveries, but the sum of pickups demand does not match to
the sum of deliveries demand.

```json
{
    "id": "job",
    "pickups": [
      {
        "places": [/** omitted **/],
        "demand": [1],
      },
      {
       "places": [/** omitted **/],
       "demand": [1],
      },
    ],
    "deliveries": [
      {
       "places": [/** omitted **/],
        "demand": [
          /** Error: should be 2 as the sum of pickups is 2 **/
          1
        ],
      }
    ]
}
```

### E1002

This error is returned when there is a job which has invalid time windows, e.g.:

```json
/** Error: end time is one hour earlier than start time**/
"times": [
    [
      "2020-07-04T12:00:00Z",
      "2020-07-04T11:00:00Z"
    ]
]
```

Each time window must satisfy the following criteria:

* array of two strings each of these specifies date in RFC3339 format. The first is considered as start,
the second - as end
* start date is earlier than end date
* if multiple time windows are specified, they must not intersect, e.g.:

```json
/** Error: second time window intersects with first one: [13:00, 14:00] **/
"times": [
    [
      "2020-07-04T10:00:00Z",
      "2020-07-04T14:00:00Z"
    ],
    [
      "2020-07-04T13:00:00Z",
      "2020-07-04T17:00:00Z"
    ]
]
```

### E1003

This error is returned when `fleet.vehicles` has vehicle types with the same `type_id`.

```json
{
  "fleet": {
    "vehicles": [
      {
        "type_id": "vehicle_1",
        /** omitted **/
      },
      {
        /** Error: this id is already used by another vehicle type **/
        "type_id": "vehicle_1",
        /** omitted **/
      }
      /** omitted **/
    ]
  }
}
```

### E1004

This error is returned when `fleet.vehicles` has vehicle types with the same `vehicle_ids`.

```json
{
  "fleet": {
    "vehicles": [
      {
        "type_id": "vehicle_1",
        "vehicle_ids": [
          "vehicle_1_a",
          "vehicle_1_b",
          /** Error: vehicle_1_b is used second time **/
          "vehicle_1_b"
        ],
        /** omitted **/
      },
      {
        "type_id": "vehicle_2",
        "vehicle_ids": [
          /** Error: vehicle_1_a is used second time **/
          "vehicle_1_a",
          "vehicle_2_b"
        ],
        /** omitted **/
      }
      /** omitted **/
    ]
  }
}
```

Please note that vehicle id should be unique across all vehicle types.


### E1005

This error is returned when vehicle has shift times violating one of time windows rules defined for jobs in E1002. 