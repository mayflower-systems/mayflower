#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;

pub use mayf_fixed_number::{DirectedRounding, FixedNumber};

/// Fixed parameters shared by every node in a forest.
///
/// Do not change them while the forest is in use: stored point-time was
/// computed under them. [`validate_config`] states the bounds they must satisfy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config<N: FixedNumber> {
    /// Deepest permitted depth. Top members have depth 0, so `max_depth = 2`
    /// permits three levels.
    pub max_depth: u8,
    /// Override rate `q`: the fraction of each source's deposit credited to
    /// *each* upline member. It must be a value on `N`'s grid; round an outside
    /// ratio up to reach one.
    pub override_rate: N,
}

/// Why an action rejected its input. Actions never mutate their inputs, so the
/// caller's stored state is unchanged after an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// An override rate or sharing fraction is below 0 or above 1.
    InvalidRatio,
    /// `override_rate >= 1`, or `max_depth * override_rate >= 1`, on `N`'s grid.
    InvalidConfig,
    /// A depth is greater than `max_depth`.
    InvalidDepth,
    /// A node fails a check it can show on its own. See [`validate_node`].
    InvalidNode,
    /// A path is empty or longer than `max_depth + 1`, or a parent and child
    /// disagree. The child's deposits exceed the parent's totals, or the child
    /// has settled later or seen a higher index than the parent holds.
    InvalidPath,
    /// [`share_points`] received a negative amount.
    InvalidAmount,
    /// `time` is earlier than the node's `last_time`.
    TimeWentBackwards,
    /// The parent index is lower than the one the node last saw.
    IndexWentBackwards,
    /// A value does not fit `u64` or `N`, or a `FixedNumber` operation failed.
    Overflow,
    /// `N` is too narrow to hold point-time. See [`validate_point_time_range`].
    RangeTooSmall,
}

/// Result type of this crate's actions.
pub type Result<T> = core::result::Result<T, Error>;

impl From<mayf_fixed_number::MathError> for Error {
    fn from(_: mayf_fixed_number::MathError) -> Self {
        Self::Overflow
    }
}

/// Check that `0 <= override_rate < 1` and `max_depth * override_rate < 1` on
/// `N`'s grid.
///
/// The second bound keeps a source's kept share, `1 - depth * override_rate`,
/// positive at every depth. Every action that takes a [`Config`] runs this
/// check. Call it directly to check an override rate right after converting
/// it.
pub fn validate_config<N: FixedNumber>(config: Config<N>) -> Result<()> {
    validate_proportion(config.override_rate)?;
    let depth = N::checked_from_i128(i128::from(config.max_depth))?;
    if config.override_rate >= N::ONE
        || config
            .override_rate
            .checked_mul_round(depth, DirectedRounding::Ceil)?
            >= N::ONE
    {
        return Err(Error::InvalidConfig);
    }
    Ok(())
}

/// Core points of a node at `depth`:
///
/// ```text
/// (1 - depth * override_rate) * deposit + override_rate * downline_deposits
/// ```
///
/// `deposit` is the node's own deposit. `downline_deposits` is the total over
/// its whole downline. The node keeps its deposit minus one override per
/// upline member, and gains one override from each downline member's deposit.
///
/// Each override is a grid value times an integer, which is exact. The kept
/// part is found by subtraction. So core points across a forest sum exactly to
/// its deposits.
///
/// Errors: [`Error::InvalidDepth`] if `depth > max_depth`;
/// [`Error::InvalidNode`] if a node at `max_depth` has downline deposits;
/// [`Error::Overflow`] if a sum or product does not fit.
pub fn effective_points<N: FixedNumber>(
    config: Config<N>,
    depth: u8,
    deposit: u64,
    downline_deposits: u64,
) -> Result<N> {
    validate_config(config)?;
    if depth > config.max_depth {
        return Err(Error::InvalidDepth);
    }
    deposit
        .checked_add(downline_deposits)
        .ok_or(Error::Overflow)?;
    if depth == config.max_depth && downline_deposits != 0 {
        return Err(Error::InvalidNode);
    }
    let depth = N::checked_from_i128(i128::from(depth))?;
    let own = N::checked_from_i128(i128::from(deposit))?;
    let downline = N::checked_from_i128(i128::from(downline_deposits))?;
    let per_upline = config
        .override_rate
        .checked_mul_round(own, DirectedRounding::Ceil)?;
    let retained = own.checked_sub(per_upline.checked_mul_round(depth, DirectedRounding::Ceil)?)?;
    let incoming = config
        .override_rate
        .checked_mul_round(downline, DirectedRounding::Ceil)?;
    Ok(retained.checked_add(incoming)?)
}

/// One member's state in one lane.
///
/// A node does not store its depth. Its depth is its position in the
/// top-to-source path the caller supplies. Settle nodes with
/// [`checkpoint_path`], and change them with [`set_deposit`] or
/// [`set_sharing`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node<N: FixedNumber> {
    /// The member's own deposit in this lane.
    pub deposit: u64,
    /// Deposits of the whole downline, not just direct children. Overrides are
    /// computed from this.
    pub downline_deposits: u64,
    /// Deposits of direct children only. Sharing divides by this.
    pub child_deposits: u64,
    /// Fraction of available point-time shared with direct children, from 0
    /// to 1. Starts at 0.
    pub sharing: N,
    /// Clock tick of the last settlement.
    pub last_time: u64,
    /// The parent's `shared_point_time_index` at the last settlement. At the
    /// next settlement, the node receives `deposit` times the parent's index
    /// growth since then. Always zero at depth 0.
    pub last_parent_point_time_index: N,
    /// Cumulative point-time this node has shared per unit of
    /// `child_deposits`. Each child is owed its own deposit times this index's
    /// growth since the child last settled. Never decreases.
    pub shared_point_time_index: N,
    /// Cumulative point-time this node has kept, up to `last_time` only. It is
    /// core points times elapsed ticks, plus receipts from the parent, minus
    /// what the node shared. Never decreases.
    pub point_time: N,
}

/// An empty node that starts at tick 0, with a zero parent index.
impl<N: FixedNumber> Default for Node<N> {
    fn default() -> Self {
        initialize_node(0, N::ZERO)
    }
}

/// Start an empty node for a new member.
///
/// Pass the current time and the parent's `shared_point_time_index` after the
/// parent has settled at that time. Pass zero at depth 0. The node then has no
/// claim on point-time from before it joined. Give it a deposit with
/// [`set_deposit`].
pub const fn initialize_node<N: FixedNumber>(time: u64, parent_point_time_index: N) -> Node<N> {
    Node {
        deposit: 0,
        downline_deposits: 0,
        child_deposits: 0,
        sharing: N::ZERO,
        last_time: time,
        last_parent_point_time_index: parent_point_time_index,
        shared_point_time_index: N::ZERO,
        point_time: N::ZERO,
    }
}

/// Check that `N` can hold the largest point-time any node can reach.
///
/// A tree's deposits fit in `u64`, and so does the clock. So a node's
/// point-time, its sharing index, and every intermediate sum stay at or below
/// `u64::MAX * u64::MAX`, about 2^128, in any state built by this crate's
/// actions. A type that holds that value cannot overflow while settling such
/// state. `U192F64` qualifies; `U64F64` and other types with 64 integer bits do
/// not.
///
/// The point-time actions ([`checkpoint`], [`checkpoint_path`],
/// [`set_deposit`], and [`set_sharing`]) run this check before they settle
/// anything. A narrow type therefore fails on first use. It cannot fail later,
/// after point-time has grown, and lock every path through a node, withdrawals
/// included. The pure calculators [`effective_points`] and [`share_points`]
/// have no such limit.
///
/// Errors: [`Error::RangeTooSmall`].
pub fn validate_point_time_range<N: FixedNumber>() -> Result<()> {
    let largest = N::checked_from_u64(u64::MAX).map_err(|_| Error::RangeTooSmall)?;
    largest
        .checked_mul_round(largest, DirectedRounding::Floor)
        .map(|_| ())
        .map_err(|_| Error::RangeTooSmall)
}

/// Check the invariants a node can show on its own.
///
/// The checks are: a valid depth and sharing fraction; non-negative indexes
/// and point-time; `child_deposits <= downline_deposits`; and a zero parent
/// index at depth 0. There are two depth checks. A node at `max_depth` has no
/// downline. A node one level above it has only direct children in its
/// downline, so its two totals match. Failures return [`Error::InvalidNode`] or the error of
/// [`effective_points`].
///
/// This cannot check the aggregates against the real tree. For example, it
/// cannot check that `downline_deposits` equals the sum of the actual downline
/// deposits. Only the caller's storage can.
pub fn validate_node<N: FixedNumber>(config: Config<N>, depth: u8, node: Node<N>) -> Result<()> {
    effective_points(config, depth, node.deposit, node.downline_deposits)?;
    validate_proportion(node.sharing)?;
    for value in [
        node.last_parent_point_time_index,
        node.shared_point_time_index,
        node.point_time,
    ] {
        if value < N::ZERO {
            return Err(Error::InvalidNode);
        }
    }
    if node.child_deposits > node.downline_deposits
        || (depth == 0 && node.last_parent_point_time_index != N::ZERO)
        // At the penultimate level, every downline member must be a direct child.
        || (depth.checked_add(1) == Some(config.max_depth)
            && node.child_deposits != node.downline_deposits)
    {
        return Err(Error::InvalidNode);
    }
    Ok(())
}

/// Result of [`share_points`]. `retained + child_deposits * index_increment`
/// equals the shared amount exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct SharingAllocation<N: FixedNumber> {
    /// The part the sharing node keeps, including any amount too small to
    /// index.
    pub retained: N,
    /// What each unit of child deposit receives. Add it to the sharing node's
    /// index.
    pub index_increment: N,
}

/// Split `available` between a node and its direct children.
///
/// The children's part is `available * sharing`, expressed as an index per
/// unit of `child_deposits`. One fused multiply-divide rounds that index down.
/// The node keeps `available - child_deposits * index_increment`, so the two
/// parts sum to `available` exactly. With no child deposits, zero sharing, or
/// nothing available, the node keeps everything.
///
/// This is a pure calculator with no clock or node state. A reward composition
/// uses it to split a reward amount by the same rule as point-time.
///
/// Errors: [`Error::InvalidRatio`] if `sharing` is outside 0 to 1;
/// [`Error::InvalidAmount`] if `available` is negative.
pub fn share_points<N: FixedNumber>(
    available: N,
    sharing: N,
    child_deposits: u64,
) -> Result<SharingAllocation<N>> {
    validate_proportion(sharing)?;
    if available < N::ZERO {
        return Err(Error::InvalidAmount);
    }
    if child_deposits == 0 || sharing == N::ZERO || available == N::ZERO {
        return Ok(SharingAllocation {
            retained: available,
            index_increment: N::ZERO,
        });
    }
    let children = N::checked_from_i128(i128::from(child_deposits))?;
    let index_increment =
        available.checked_mul_div_round(sharing, children, DirectedRounding::Floor)?;
    let shared = index_increment.checked_mul_round(children, DirectedRounding::Floor)?;
    Ok(SharingAllocation {
        retained: available.checked_sub(shared)?,
        index_increment,
    })
}

/// Arguments to [`checkpoint`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckpointArgs<N: FixedNumber> {
    /// The forest's fixed parameters.
    pub config: Config<N>,
    /// The node's depth. 0 for a top member.
    pub depth: u8,
    /// Clock tick to settle to. Must not be earlier than the node's
    /// `last_time`.
    pub time: u64,
    /// The parent's `shared_point_time_index` after the parent settled at
    /// `time`. Zero at depth 0.
    pub parent_point_time_index: N,
}

/// Result of [`checkpoint`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "persist the returned replacement state"]
pub struct CheckpointOutput<N: FixedNumber> {
    /// The settled node. Store it in place of the input node.
    pub next_node: Node<N>,
}

/// Arguments to [`checkpoint_path`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckpointPathArgs<N: FixedNumber> {
    /// The forest's fixed parameters.
    pub config: Config<N>,
    /// Clock tick to settle every node to.
    pub time: u64,
}

/// Result of [`checkpoint_path`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "persist the returned replacement state"]
pub struct CheckpointPathOutput<N: FixedNumber> {
    /// The settled path, top first. Store every node atomically.
    pub next_path: Vec<Node<N>>,
}

/// Arguments to [`set_deposit`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetDepositArgs<N: FixedNumber> {
    /// The forest's fixed parameters.
    pub config: Config<N>,
    /// Clock tick at which the new deposit takes effect.
    pub time: u64,
    /// The source's new deposit as an absolute amount, not a change.
    pub new_deposit: u64,
}

/// Result of [`set_deposit`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "persist the returned replacement state"]
pub struct SetDepositOutput<N: FixedNumber> {
    /// The settled path with the new deposit and updated totals, top first.
    /// Store every node atomically.
    pub next_path: Vec<Node<N>>,
}

/// Arguments to [`set_sharing`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetSharingArgs<N: FixedNumber> {
    /// The forest's fixed parameters.
    pub config: Config<N>,
    /// Clock tick at which the new fraction takes effect.
    pub time: u64,
    /// The source's new sharing fraction, from 0 to 1.
    pub sharing: N,
}

/// Result of [`set_sharing`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "persist the returned replacement state"]
pub struct SetSharingOutput<N: FixedNumber> {
    /// The settled path with the new fraction on the source, top first. Store
    /// every node atomically.
    pub next_path: Vec<Node<N>>,
}

/// Settle one node up to `time`, after its parent has settled at the same
/// time.
///
/// The node gains its core points times the elapsed ticks, plus its deposit
/// times the growth of the parent's index. It then shares part of that total
/// with its children through its own index, as [`share_points`] describes.
/// Multiplying fixed-point values by integer ticks or deposits is exact.
///
/// Prefer [`checkpoint_path`], which settles parents first. Call this directly
/// only if the caller already guarantees that order.
///
/// Errors: [`Error::RangeTooSmall`], [`Error::TimeWentBackwards`],
/// [`Error::IndexWentBackwards`], and the errors of [`validate_node`].
pub fn checkpoint<N: FixedNumber>(
    node: Node<N>,
    args: CheckpointArgs<N>,
) -> Result<CheckpointOutput<N>> {
    validate_point_time_range::<N>()?;
    settle(node, args)
}

fn settle<N: FixedNumber>(node: Node<N>, args: CheckpointArgs<N>) -> Result<CheckpointOutput<N>> {
    let CheckpointArgs {
        config,
        depth,
        time,
        parent_point_time_index,
    } = args;
    validate_node(config, depth, node)?;
    if depth == 0 && parent_point_time_index != N::ZERO {
        return Err(Error::InvalidNode);
    }
    let elapsed = time
        .checked_sub(node.last_time)
        .ok_or(Error::TimeWentBackwards)?;
    if parent_point_time_index < node.last_parent_point_time_index {
        return Err(Error::IndexWentBackwards);
    }
    let parent_delta = parent_point_time_index.checked_sub(node.last_parent_point_time_index)?;
    let points = effective_points(config, depth, node.deposit, node.downline_deposits)?;
    let ticks = N::checked_from_i128(i128::from(elapsed))?;
    let own = N::checked_from_i128(i128::from(node.deposit))?;
    let base = points.checked_mul_round(ticks, DirectedRounding::Floor)?;
    let incoming = parent_delta.checked_mul_round(own, DirectedRounding::Floor)?;
    let allocation = share_points(
        base.checked_add(incoming)?,
        node.sharing,
        node.child_deposits,
    )?;
    Ok(CheckpointOutput {
        next_node: Node {
            last_time: time,
            last_parent_point_time_index: parent_point_time_index,
            shared_point_time_index: node
                .shared_point_time_index
                .checked_add(allocation.index_increment)?,
            point_time: node.point_time.checked_add(allocation.retained)?,
            ..node
        },
    })
}

/// Settle every node on a path to `time`, top first.
///
/// `path[0]` must be a top member at depth 0, and each `path[i]` must be the
/// parent of `path[i + 1]`. The last node is the source. The function checks
/// that adjacent nodes have consistent totals. It cannot prove the upline, so
/// the caller must load the path from authenticated storage.
///
/// Siblings off the path need no update, because their rights wait in their
/// parents' indexes. Work and output size are O(path length).
///
/// Errors: [`Error::RangeTooSmall`], [`Error::InvalidPath`], and the errors of
/// [`checkpoint`].
pub fn checkpoint_path<N: FixedNumber>(
    path: &[Node<N>],
    args: CheckpointPathArgs<N>,
) -> Result<CheckpointPathOutput<N>> {
    let CheckpointPathArgs { config, time } = args;
    validate_point_time_range::<N>()?;
    validate_path(config, path)?;
    let mut next = Vec::with_capacity(path.len());
    let mut parent_point_time_index = N::ZERO;
    for (depth, node) in path.iter().enumerate() {
        let output = settle(
            *node,
            CheckpointArgs {
                config,
                depth: depth as u8,
                time,
                parent_point_time_index,
            },
        )?;
        parent_point_time_index = output.next_node.shared_point_time_index;
        next.push(output.next_node);
    }
    Ok(CheckpointPathOutput { next_path: next })
}

/// Settle the path to `time`, then set the source's deposit to `new_deposit`.
///
/// The source is the last node of `path`. Its change applies to the
/// `downline_deposits` of every upline member and to its parent's
/// `child_deposits`. The old deposit applies up to `time` and the new one
/// after. Setting the same deposit only settles. Setting zero keeps the node's
/// point-time and index history.
pub fn set_deposit<N: FixedNumber>(
    path: &[Node<N>],
    args: SetDepositArgs<N>,
) -> Result<SetDepositOutput<N>> {
    let SetDepositArgs {
        config,
        time,
        new_deposit,
    } = args;
    let mut next = checkpoint_path(path, CheckpointPathArgs { config, time })?.next_path;
    let source = next.len() - 1;
    let old_deposit = next[source].deposit;
    next[source].deposit = new_deposit;
    for upline in &mut next[..source] {
        upline.downline_deposits =
            replace_contribution(upline.downline_deposits, old_deposit, new_deposit)?;
    }
    if source != 0 {
        let parent = &mut next[source - 1];
        parent.child_deposits =
            replace_contribution(parent.child_deposits, old_deposit, new_deposit)?;
    }
    validate_path(config, &next)?;
    Ok(SetDepositOutput { next_path: next })
}

/// Settle the path to `time`, then set the source's sharing fraction.
///
/// The old fraction applies up to `time` and the new one after. Point-time the
/// source already kept, or already indexed for its children, does not change.
pub fn set_sharing<N: FixedNumber>(
    path: &[Node<N>],
    args: SetSharingArgs<N>,
) -> Result<SetSharingOutput<N>> {
    let SetSharingArgs {
        config,
        time,
        sharing,
    } = args;
    validate_proportion(sharing)?;
    let mut next = checkpoint_path(path, CheckpointPathArgs { config, time })?.next_path;
    next.last_mut().ok_or(Error::InvalidPath)?.sharing = sharing;
    Ok(SetSharingOutput { next_path: next })
}

fn validate_path<N: FixedNumber>(config: Config<N>, path: &[Node<N>]) -> Result<()> {
    validate_config(config)?;
    if path.is_empty() || path.len() > usize::from(config.max_depth) + 1 {
        return Err(Error::InvalidPath);
    }
    for (depth, node) in path.iter().enumerate() {
        validate_node(config, depth as u8, *node)?;
    }
    for pair in path.windows(2) {
        let [parent, child] = [pair[0], pair[1]];
        let child_subtree = child
            .deposit
            .checked_add(child.downline_deposits)
            .ok_or(Error::Overflow)?;
        let other_downline = parent
            .downline_deposits
            .checked_sub(child_subtree)
            .ok_or(Error::InvalidPath)?;
        let other_children = parent
            .child_deposits
            .checked_sub(child.deposit)
            .ok_or(Error::InvalidPath)?;
        if other_children > other_downline
            || child.last_parent_point_time_index > parent.shared_point_time_index
            || child.last_time > parent.last_time
        {
            return Err(Error::InvalidPath);
        }
    }
    Ok(())
}

fn replace_contribution(total: u64, old: u64, new: u64) -> Result<u64> {
    total
        .checked_sub(old)
        .ok_or(Error::InvalidPath)?
        .checked_add(new)
        .ok_or(Error::Overflow)
}

fn validate_proportion<N: FixedNumber>(value: N) -> Result<()> {
    if value < N::ZERO || value > N::ONE {
        return Err(Error::InvalidRatio);
    }
    Ok(())
}
