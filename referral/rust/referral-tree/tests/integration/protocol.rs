//! Point-time behavior on a deliberately coarse 0.1-unit grid.
use crate::support;
use referral_tree::*;
use support::{WideTenths as Tenths, tenths};
type TestNode = Node<Tenths>;

const CONFIG: Config<Tenths> = Config {
    max_depth: 2,
    override_rate: tenths(1),
};

fn node(deposit: u64, downline: u64, children: u64) -> TestNode {
    Node {
        deposit,
        downline_deposits: downline,
        child_deposits: children,
        ..Node::default()
    }
}
fn settle(path: &[TestNode], time: u64) -> Vec<TestNode> {
    checkpoint_path(
        path,
        CheckpointPathArgs {
            config: CONFIG,
            time,
        },
    )
    .unwrap()
    .next_path
}
fn totals(path: &[TestNode]) -> Vec<i64> {
    path.iter()
        .map(|n| i64::try_from(n.point_time.checked_to_scaled_u128().unwrap()).unwrap())
        .collect()
}

#[test]
fn published_examples_and_zero_deposit_affiliate() {
    let example = [node(100, 200, 100), node(100, 100, 100), node(100, 0, 0)];
    assert_eq!(totals(&settle(&example, 1)), [1200, 1000, 800]);
    let affiliate = [node(100, 100, 0), node(0, 100, 100), node(100, 0, 0)];
    assert_eq!(totals(&settle(&affiliate, 1)), [1100, 100, 800]);
}

#[test]
fn recursive_sharing_is_deposit_weighted_and_conserves_point_time() {
    let mut path = [node(100, 200, 100), node(100, 100, 100), node(100, 0, 0)];
    path[0].sharing = tenths(5);
    path[1].sharing = tenths(5);
    assert_eq!(totals(&settle(&path, 1)), [600, 800, 1600]);
    assert_eq!(totals(&settle(&path, 1)).iter().sum::<i64>(), 3000);
    assert_eq!(path[0].point_time, Tenths::ZERO);
}

#[test]
fn original_source_attribution_proves_conservation_for_many_forests() {
    // Allocate sources independently, without using cached downline weights.
    let mut rng = Rng(83947);
    for max_depth in 0..=5 {
        for override_raw in 0..=3 {
            if i64::from(max_depth) * override_raw >= 10 {
                continue;
            }
            let config = Config {
                max_depth,
                override_rate: tenths(override_raw),
            };
            for _ in 0..100 {
                let mut parents: Vec<Option<usize>> = Vec::new();
                let mut depths = Vec::new();
                let mut own = Vec::new();
                for i in 0..20 {
                    let candidate = if i == 0 {
                        0
                    } else {
                        (rng.next() % i as u64) as usize
                    };
                    let parent =
                        if i == 0 || rng.next().is_multiple_of(4) || depths[candidate] == max_depth
                        {
                            None
                        } else {
                            Some(candidate)
                        };
                    depths.push(parent.map_or(0, |p| depths[p] + 1));
                    parents.push(parent);
                    own.push(rng.next() % 1000);
                }
                let mut expected = vec![0_i64; own.len()];
                let mut downline = vec![0_u64; own.len()];
                for source in 0..own.len() {
                    expected[source] +=
                        (10 - i64::from(depths[source]) * override_raw) * own[source] as i64;
                    let mut upline = parents[source];
                    while let Some(a) = upline {
                        expected[a] += override_raw * own[source] as i64;
                        downline[a] += own[source];
                        upline = parents[a];
                    }
                }
                for i in 0..own.len() {
                    assert_eq!(
                        effective_points(config, depths[i], own[i], downline[i]).unwrap(),
                        tenths(expected[i])
                    );
                    // Every subtree pays only its outside upline. Internal
                    // overrides cancel even for branching trees and forests.
                    let mut subtree_points = 0_i64;
                    for (member, points) in expected.iter().enumerate() {
                        let mut cursor = Some(member);
                        while let Some(current) = cursor {
                            if current == i {
                                subtree_points += points;
                                break;
                            }
                            cursor = parents[current];
                        }
                    }
                    assert_eq!(
                        subtree_points,
                        (10 - i64::from(depths[i]) * override_raw) * (own[i] + downline[i]) as i64
                    );
                }
                assert_eq!(
                    expected.iter().sum::<i64>(),
                    10 * own.iter().map(|p| *p as i64).sum::<i64>()
                );
            }
        }
    }
}

#[test]
fn vertical_and_horizontal_splits_preserve_combined_core_weight() {
    for total in 0..=200 {
        let alice = effective_points(CONFIG, 0, 100, total).unwrap();
        let bob = effective_points(CONFIG, 1, total, 0).unwrap();
        for moved in 0..=total {
            let parent = effective_points(CONFIG, 1, total - moved, moved).unwrap();
            let child = effective_points(CONFIG, 2, moved, 0).unwrap();
            assert_eq!(parent.checked_add(child).unwrap(), bob);
            assert_eq!(effective_points(CONFIG, 0, 100, total).unwrap(), alice);
            let left = effective_points(CONFIG, 1, moved, 0).unwrap();
            let right = effective_points(CONFIG, 1, total - moved, 0).unwrap();
            assert_eq!(left.checked_add(right).unwrap(), bob);
        }
    }
}

#[test]
fn base_point_time_is_exact_and_independent_of_checkpoint_frequency() {
    let path = [node(0, 1, 1), node(1, 0, 0)];
    let once = settle(&path, 10);
    let mut frequent = path.to_vec();
    for time in 1..=10 {
        frequent = settle(&frequent, time);
    }
    assert_eq!(totals(&once), [10, 90]);
    assert_eq!(frequent, once);
    assert_eq!(settle(&frequent, 10), frequent);
}

#[test]
fn sharing_rounding_stays_with_parent_and_never_disappears() {
    let path = [
        Node {
            sharing: tenths(5),
            ..node(1, 7, 7)
        },
        node(7, 0, 0),
    ];
    let once = settle(&path, 2);
    let frequent = settle(&settle(&path, 1), 2);
    // 3.4 available: a single 50% split indexes 1.4; two splits index 0.7 each.
    assert_eq!(totals(&once), [20, 140]);
    assert_eq!(totals(&frequent).iter().sum::<i64>(), 160);
    // A deliberately smaller sharing fraction demonstrates the allowed bias:
    // one combined checkpoint creates an index; individual checkpoints do not.
    let path = [
        Node {
            sharing: tenths(1),
            ..path[0]
        },
        path[1],
    ];
    let once = settle(&path, 10);
    let mut frequent = path.to_vec();
    for time in 1..=10 {
        frequent = settle(&frequent, time);
    }
    assert!(frequent[0].point_time > once[0].point_time);
    assert_eq!(totals(&once).iter().sum::<i64>(), 800);
    assert_eq!(totals(&frequent).iter().sum::<i64>(), 800);
    assert_eq!(frequent[0].shared_point_time_index, Tenths::ZERO);
}

#[test]
fn policies_and_deposit_changes_apply_after_old_history_is_settled() {
    let mut path = [node(1, 10, 3), node(3, 7, 7), node(7, 0, 0)];
    path[0].sharing = tenths(5);
    path[1].sharing = tenths(5);
    let before = settle(&path, 3);
    let changed = set_deposit(
        &path,
        SetDepositArgs {
            config: CONFIG,
            time: 3,
            new_deposit: 8,
        },
    )
    .unwrap()
    .next_path;
    assert_eq!(changed[0].deposit, 1);
    assert_eq!(changed[0].downline_deposits, 11);
    assert_eq!(changed[0].child_deposits, 3);
    assert_eq!(changed[1].downline_deposits, 8);
    assert_eq!(changed[1].child_deposits, 8);
    assert_eq!(changed[2].deposit, 8);
    for i in 0..3 {
        assert_eq!(changed[i].point_time, before[i].point_time);
        assert_eq!(
            changed[i].shared_point_time_index,
            before[i].shared_point_time_index
        );
    }
    let unchanged = set_deposit(
        &changed,
        SetDepositArgs {
            config: CONFIG,
            time: 3,
            new_deposit: 8,
        },
    )
    .unwrap()
    .next_path;
    assert_eq!(unchanged, changed);
    let back = set_deposit(
        &changed,
        SetDepositArgs {
            config: CONFIG,
            time: 3,
            new_deposit: 7,
        },
    )
    .unwrap()
    .next_path;
    assert_eq!(
        back, before,
        "absolute deposit round trips do not harvest rounding"
    );
    let policy = set_sharing(
        &changed[..2],
        SetSharingArgs {
            config: CONFIG,
            time: 3,
            sharing: Tenths::ZERO,
        },
    )
    .unwrap()
    .next_path;
    assert_eq!(policy[1].point_time, before[1].point_time);
    assert_eq!(
        policy[1].shared_point_time_index,
        before[1].shared_point_time_index
    );
}

#[test]
fn dormant_sibling_keeps_its_share_across_another_childs_deposit_change() {
    let parent = Node {
        sharing: Tenths::ONE,
        ..node(10, 10, 10)
    };
    let dormant = node(7, 0, 0);
    let changed = set_deposit(
        &[parent, node(3, 0, 0)],
        SetDepositArgs {
            config: CONFIG,
            time: 1,
            new_deposit: 4,
        },
    )
    .unwrap()
    .next_path;
    assert_eq!(changed[0].shared_point_time_index, tenths(11));
    let settled = settle(&[changed[0], dormant], 1);
    assert_eq!(settled[1].point_time, tenths(140));
    assert_eq!(settled[0].shared_point_time_index, tenths(11));
    assert_eq!(settled[0].point_time, Tenths::ZERO);
    assert_eq!(
        changed[1]
            .point_time
            .checked_add(settled[1].point_time)
            .unwrap(),
        tenths(200)
    );
}

#[test]
fn bob_onboarding_does_not_inherit_alices_past_point_time() {
    // Alice holds ten deposits and shares 100%. Nobody can receive shares yet.
    let alice = Node {
        sharing: Tenths::ONE,
        ..node(10, 0, 0)
    };
    let alice = settle(&[alice], 100)[0];
    assert_eq!(alice.point_time, tenths(10_000));
    let bob = initialize_node(100, alice.shared_point_time_index);
    let joined = set_deposit(
        &[alice, bob],
        SetDepositArgs {
            config: CONFIG,
            time: 100,
            new_deposit: 10,
        },
    )
    .unwrap()
    .next_path;
    assert_eq!(joined[1].point_time, Tenths::ZERO);
    let settled = settle(&joined, 110);
    assert_eq!(totals(&settled), [10_000, 2_000]);
    let emptied = set_deposit(
        &settled,
        SetDepositArgs {
            config: CONFIG,
            time: 110,
            new_deposit: 0,
        },
    )
    .unwrap()
    .next_path;
    assert_eq!(emptied[1].point_time, settled[1].point_time);
    let later = settle(&emptied, 120);
    assert_eq!(totals(&later), [11_000, 2_000]);
    let rejoined = set_deposit(
        &later,
        SetDepositArgs {
            config: CONFIG,
            time: 120,
            new_deposit: 10,
        },
    )
    .unwrap()
    .next_path;
    assert_eq!(rejoined[1].point_time, tenths(2_000));
}

#[test]
fn zero_deposit_affiliates_never_receive_a_parent_share() {
    let path = [
        Node {
            sharing: Tenths::ONE,
            ..node(10, 10, 0)
        },
        Node {
            sharing: Tenths::ONE,
            ..node(0, 10, 10)
        },
        node(10, 0, 0),
    ];
    let result = settle(&path, 1);
    assert_eq!(totals(&result), [110, 0, 90]);
    assert_eq!(result[0].shared_point_time_index, Tenths::ZERO);
}

#[test]
fn newcomer_baselines_existing_parent_index_before_earning() {
    let parent = Node {
        sharing: Tenths::ONE,
        ..node(10, 10, 10)
    };
    let existing = settle(&[parent, node(10, 0, 0)], 10);
    assert_eq!(existing[0].shared_point_time_index, tenths(110));
    assert_eq!(existing[1].point_time, tenths(2_000));

    let newcomer = initialize_node(10, existing[0].shared_point_time_index);
    let joined = set_deposit(
        &[existing[0], newcomer],
        SetDepositArgs {
            config: CONFIG,
            time: 10,
            new_deposit: 10,
        },
    )
    .unwrap()
    .next_path;
    assert_eq!(joined[1].point_time, Tenths::ZERO);
    assert_eq!(joined[1].last_parent_point_time_index, tenths(110));

    let new_history = settle(&joined, 20);
    let old_history = settle(&[new_history[0], existing[1]], 20);
    assert_eq!(new_history[1].point_time, tenths(1_500));
    assert_eq!(old_history[1].point_time, tenths(3_500));
    assert_eq!(new_history[0].point_time, Tenths::ZERO);
}

#[test]
fn child_overflow_discards_completed_upline_work_for_every_path_action() {
    let path = [
        node(1, 1, 1),
        Node {
            point_time: Tenths::MAX,
            ..node(1, 0, 0)
        },
    ];
    let original = path;
    // The parent successfully accrues before the child's retained total
    // overflows. No action may expose that partial replacement path.
    assert_eq!(settle(&path[..1], 1)[0].point_time, tenths(11));
    assert_eq!(
        checkpoint_path(
            &path,
            CheckpointPathArgs {
                config: CONFIG,
                time: 1,
            },
        ),
        Err(Error::Overflow)
    );
    assert_eq!(
        set_deposit(
            &path,
            SetDepositArgs {
                config: CONFIG,
                time: 1,
                new_deposit: 0,
            },
        ),
        Err(Error::Overflow)
    );
    assert_eq!(
        set_sharing(
            &path,
            SetSharingArgs {
                config: CONFIG,
                time: 1,
                sharing: Tenths::ONE,
            },
        ),
        Err(Error::Overflow)
    );
    assert_eq!(path, original);
}

#[test]
fn penultimate_level_cannot_hide_downline_below_the_depth_limit() {
    for max_depth in [1, 2, u8::MAX] {
        let config = Config {
            max_depth,
            override_rate: Tenths::ZERO,
        };
        let depth = max_depth - 1;
        assert_eq!(validate_node(config, depth, node(0, 100, 100)), Ok(()));
        let invalid = node(0, 100, 0);
        assert_eq!(
            validate_node(config, depth, invalid),
            Err(Error::InvalidNode)
        );
        assert_eq!(
            checkpoint(
                invalid,
                CheckpointArgs {
                    config,
                    depth,
                    time: 0,
                    parent_point_time_index: Tenths::ZERO,
                },
            ),
            Err(Error::InvalidNode)
        );
    }
    let path = [node(0, 100, 0), node(0, 100, 0)];
    let original = path;
    assert_eq!(
        checkpoint_path(
            &path,
            CheckpointPathArgs {
                config: CONFIG,
                time: 0,
            },
        ),
        Err(Error::InvalidNode)
    );
    assert_eq!(
        set_deposit(
            &path,
            SetDepositArgs {
                config: CONFIG,
                time: 0,
                new_deposit: 1,
            },
        ),
        Err(Error::InvalidNode)
    );
    assert_eq!(
        set_sharing(
            &path,
            SetSharingArgs {
                config: CONFIG,
                time: 0,
                sharing: Tenths::ONE,
            },
        ),
        Err(Error::InvalidNode)
    );
    assert_eq!(path, original);
}

#[test]
fn maximum_configured_depth_supports_all_256_participating_levels() {
    let config = Config {
        max_depth: u8::MAX,
        override_rate: Tenths::ZERO,
    };
    let path = vec![Node::default(); 256];
    let activated = set_deposit(
        &path,
        SetDepositArgs {
            config,
            time: 0,
            new_deposit: 1,
        },
    )
    .unwrap()
    .next_path;
    assert_eq!(activated.len(), 256);
    for (depth, node) in activated.iter().enumerate() {
        assert_eq!(node.deposit, u64::from(depth == 255));
        assert_eq!(node.downline_deposits, u64::from(depth < 255));
        assert_eq!(node.child_deposits, u64::from(depth == 254));
    }
    let settled = checkpoint_path(&activated, CheckpointPathArgs { config, time: 10 })
        .unwrap()
        .next_path;
    assert_eq!(settled[255].point_time, tenths(100));
    assert_eq!(totals(&settled).iter().sum::<i64>(), 100);
    assert!(settled.iter().all(|node| node.last_time == 10));
}

#[test]
fn validation_runs_on_no_op_paths_and_rejects_time_and_index_regressions() {
    let original = Node {
        deposit: 7,
        downline_deposits: 11,
        child_deposits: 11,
        sharing: tenths(3),
        last_time: 100,
        last_parent_point_time_index: tenths(15),
        shared_point_time_index: tenths(23),
        point_time: tenths(123),
    };
    let args = CheckpointArgs {
        config: CONFIG,
        depth: 1,
        time: 100,
        parent_point_time_index: tenths(15),
    };
    assert_eq!(checkpoint(original, args).unwrap().next_node, original);
    assert_eq!(
        checkpoint(
            Node {
                sharing: tenths(11),
                ..original
            },
            args
        ),
        Err(Error::InvalidRatio)
    );
    assert_eq!(
        checkpoint(original, CheckpointArgs { time: 99, ..args }),
        Err(Error::TimeWentBackwards)
    );
    assert_eq!(
        checkpoint(
            original,
            CheckpointArgs {
                parent_point_time_index: tenths(14),
                ..args
            }
        ),
        Err(Error::IndexWentBackwards)
    );
    assert_eq!(
        checkpoint(original, CheckpointArgs { depth: 3, ..args }),
        Err(Error::InvalidDepth)
    );
    assert_eq!(
        checkpoint(original, CheckpointArgs { depth: 0, ..args }),
        Err(Error::InvalidNode)
    );
    assert_eq!(
        checkpoint(
            node(0, 1, 1),
            CheckpointArgs {
                depth: 2,
                time: 0,
                parent_point_time_index: Tenths::ZERO,
                ..args
            }
        ),
        Err(Error::InvalidNode)
    );
}

#[test]
fn malformed_paths_and_overflow_fail_without_mutating_inputs() {
    let args = CheckpointPathArgs {
        config: CONFIG,
        time: 10,
    };
    assert_eq!(
        checkpoint_path::<Tenths>(&[], args),
        Err(Error::InvalidPath)
    );
    assert_eq!(
        checkpoint_path(&[node(0, 2, 2), node(3, 0, 0)], args),
        Err(Error::InvalidPath)
    );
    assert_eq!(
        checkpoint_path(
            &[
                node(0, 3, 3),
                Node {
                    last_time: 1,
                    ..node(3, 0, 0)
                }
            ],
            args
        ),
        Err(Error::InvalidPath)
    );
    assert_eq!(
        checkpoint_path(
            &[
                node(0, 3, 3),
                Node {
                    last_parent_point_time_index: tenths(1),
                    ..node(3, 0, 0)
                }
            ],
            args
        ),
        Err(Error::InvalidPath)
    );
    assert_eq!(
        checkpoint_path(&[node(0, 3, 4)], args),
        Err(Error::InvalidNode)
    );
    assert_eq!(
        checkpoint_path(&[node(0, 5, 5), node(2, 3, 3)], args),
        Err(Error::InvalidPath)
    );
    assert_eq!(
        checkpoint_path(&[node(0, 0, 0); 4], args),
        Err(Error::InvalidPath)
    );
    let path = [node(10, 10, 10), node(10, 0, 0)];
    assert_eq!(
        set_deposit(
            &path,
            SetDepositArgs {
                config: CONFIG,
                time: 0,
                new_deposit: u64::MAX
            }
        ),
        Err(Error::Overflow)
    );
    assert_eq!(path[0].downline_deposits, 10);
    assert_eq!(
        validate_config(Config {
            max_depth: 10,
            ..CONFIG
        }),
        Err(Error::InvalidConfig)
    );
    assert_eq!(
        validate_config(Config {
            max_depth: 0,
            override_rate: Tenths::ONE
        }),
        Err(Error::InvalidConfig)
    );
    assert_eq!(
        validate_config(Config {
            max_depth: u8::MAX,
            override_rate: Tenths::ZERO
        }),
        Ok(())
    );
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}
