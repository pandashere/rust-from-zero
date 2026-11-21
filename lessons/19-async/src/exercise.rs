//! 主练习：有界并发通知调度器。
//!
//! 这个文件里混有两类代码：
//!
//! - 已经写好的部分：数据类型、`FakeService`（假服务）和结果模型；
//! - 需要你完成的部分：`with_timeout`、`send_with_retries`、`run_batch`。
//!
//! 学习者只替换函数体里的 `todo!()`，不要修改签名，也不要改动
//! `tests/job_tests.rs`。每个函数上方的注释就是完整的行为约定。
//!
//! 背景故事：一个程序要向外发送一批通知。真实网络会慢、会抖动、会永久
//! 拒绝，也可能让请求一直挂住。本练习用 `FakeService` 模拟这些情况，
//! 让测试不访问网络、不真正睡眠，同时保留并发、超时、重试和取消的关键行为。

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;

use crate::runtime::yield_now;

/// 一条待发送的通知作业。
///
/// 作业由调度器拥有；发送时只把引用交给服务，因此作业文本不会被复制或修改。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// 作业编号，用来排序结果和查找脚本。调用方应保证一轮作业内编号不重复。
    pub id: usize,
    /// 收件人标识，例如邮箱或用户名。
    pub recipient: String,
    /// 要发送的正文。
    pub message: String,
}

impl Job {
    /// 创建一条作业，把 `recipient` 和 `message` 的内容存进对应字段。
    pub fn new(id: usize, recipient: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id,
            recipient: recipient.into(),
            message: message.into(),
        }
    }
}

/// `FakeService` 对一次尝试的预设结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// 成功。先让出 `delay` 次，再返回。
    Success { delay: usize },
    /// 暂时性失败，调用方可以重试。先让出 `delay` 次，再返回。
    Temporary { reason: String, delay: usize },
    /// 永久性失败，调用方不应重试。先让出 `delay` 次，再返回。
    Permanent { reason: String, delay: usize },
    /// 直接 panic，用来模拟服务实现里的 bug。第一次 poll 就发生。
    Panic,
}

impl Step {
    /// 构造一个延迟 `delay` 次让出后成功的步骤。
    pub fn success(delay: usize) -> Self {
        Self::Success { delay }
    }

    /// 构造一个延迟 `delay` 次让出后返回暂时性失败的步骤。
    pub fn temporary(reason: impl Into<String>, delay: usize) -> Self {
        Self::Temporary {
            reason: reason.into(),
            delay,
        }
    }

    /// 构造一个延迟 `delay` 次让出后返回永久性失败的步骤。
    pub fn permanent(reason: impl Into<String>, delay: usize) -> Self {
        Self::Permanent {
            reason: reason.into(),
            delay,
        }
    }
}

/// 服务返回的可重试/不可重试错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceFault {
    /// 暂时性错误：网络抖动、服务繁忙等，稍后重试可能成功。
    Temporary { reason: String },
    /// 永久性错误：地址不存在、参数非法等，重试不会改变结果。
    Permanent { reason: String },
}

/// 一个可编程的假通知服务。
///
/// 脚本以“作业编号 → 每次尝试的步骤列表”的形式给出。第 1 次调用使用列表的
/// 第 1 个元素，第 2 次调用使用第 2 个元素，依此类推；次数从 1 开始。
///
/// 服务同时记录三件可观察的事实：
///
/// - 每个作业被调用了几次（`calls_for`）；
/// - 当前有多少个 `send` future 已经开始但还没结束（`active_now`）；
/// - 历史最高同时进行数（`peak_active`）。
///
/// 这些计数都用 `Cell`/`RefCell` 保存，因为本课运行时是单线程的；真实服务
/// 如果要跨线程共享，需要换成线程安全的类型。
#[derive(Debug)]
pub struct FakeService {
    scripts: RefCell<HashMap<usize, Vec<Step>>>,
    calls: RefCell<HashMap<usize, usize>>,
    active: Cell<usize>,
    peak: Cell<usize>,
}

impl FakeService {
    /// 用一组 `(作业编号, 尝试脚本)` 创建服务。没有出现在列表里的作业编号，
    /// 每次尝试都会得到“脚本缺失”的永久性错误。
    pub fn new(scripts: Vec<(usize, Vec<Step>)>) -> Self {
        Self {
            scripts: RefCell::new(scripts.into_iter().collect()),
            calls: RefCell::new(HashMap::new()),
            active: Cell::new(0),
            peak: Cell::new(0),
        }
    }

    /// 返回编号为 `id` 的作业一共被调用了几次；从未调用过时返回 0。
    pub fn calls_for(&self, id: usize) -> usize {
        self.calls.borrow().get(&id).copied().unwrap_or(0)
    }

    /// 返回此刻仍在进行中的 `send` 调用数量。
    pub fn active_now(&self) -> usize {
        self.active.get()
    }

    /// 返回从服务创建以来，同时进行中的 `send` 调用数量的最大值。
    pub fn peak_active(&self) -> usize {
        self.peak.get()
    }

    /// 执行第 `attempt` 次发送尝试（`attempt` 从 1 开始）。
    ///
    /// 行为由脚本决定：
    ///
    /// - 找到第 `attempt` 个步骤后，先让出 `delay` 次；
    /// - `Success` 返回 `Ok("已通知 {recipient}：{message}")`；
    /// - `Temporary`/`Permanent` 返回对应的 `ServiceFault`；
    /// - `Panic` 在第一次 poll 时 panic；
    /// - 脚本没有第 `attempt` 个步骤时，返回永久性错误，理由为
    ///   `作业 #<id> 没有第 <attempt> 次尝试的脚本`。
    ///
    /// 无论正常返回、panic 还是被超时取消（future 被丢弃），
    /// “进行中”计数都会在离开时减一。这个清理由 `ActiveGuard` 完成，
    /// 它演示了取消时资源如何释放。
    pub async fn send(&self, job: &Job, attempt: usize) -> Result<String, ServiceFault> {
        let active = self.active.get() + 1;
        self.active.set(active);
        self.peak.set(self.peak.get().max(active));
        let _guard = ActiveGuard { service: self };

        *self.calls.borrow_mut().entry(job.id).or_insert(0) += 1;

        let scripted = self
            .scripts
            .borrow()
            .get(&job.id)
            .and_then(|steps| attempt.checked_sub(1).and_then(|index| steps.get(index)))
            .cloned();
        let step = scripted.unwrap_or_else(|| Step::Permanent {
            reason: format!("作业 #{} 没有第 {attempt} 次尝试的脚本", job.id),
            delay: 0,
        });

        let delay = match &step {
            Step::Success { delay }
            | Step::Temporary { delay, .. }
            | Step::Permanent { delay, .. } => *delay,
            Step::Panic => 0,
        };
        for _ in 0..delay {
            yield_now().await;
        }

        match step {
            Step::Success { .. } => Ok(format!("已通知 {}：{}", job.recipient, job.message)),
            Step::Temporary { reason, .. } => Err(ServiceFault::Temporary { reason }),
            Step::Permanent { reason, .. } => Err(ServiceFault::Permanent { reason }),
            Step::Panic => panic!("作业 #{} 第 {attempt} 次尝试模拟 panic", job.id),
        }
    }
}

/// 在 `send` future 结束或被丢弃时减少“进行中”计数。
struct ActiveGuard<'a> {
    service: &'a FakeService,
}

impl Drop for ActiveGuard<'_> {
    fn drop(&mut self) {
        let current = self.service.active.get();
        self.service.active.set(current.saturating_sub(1));
    }
}

/// 重试策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    /// 最多尝试几次，包含第一次。`run_batch` 要求它至少为 1。
    pub max_attempts: usize,
    /// 单次尝试最多允许经历多少次 `Pending`；再多一次就判为超时。
    pub patience: usize,
}

/// 一批作业的调度配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchConfig {
    /// 同时进行中的作业数量上限，必须至少为 1。
    pub limit: usize,
    /// 每个作业使用的重试与超时策略。
    pub policy: RetryPolicy,
}

/// 单次尝试超过允许的 `Pending` 次数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimedOut;

/// 一个作业最终失败的原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobError {
    /// 每次尝试都返回暂时性错误，重试次数已经用完。
    TemporaryExhausted { last_reason: String },
    /// 服务返回永久性错误，立即停止重试。
    Permanent { reason: String },
    /// 某次尝试超过 `patience`，立即停止重试。
    TimedOut,
    /// 某次尝试 panic，立即停止重试。
    Panicked,
}

/// 一个作业的最终结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobOutcome {
    /// 成功。`attempts` 是实际尝试次数，从 1 开始。
    Success {
        id: usize,
        output: String,
        attempts: usize,
    },
    /// 失败。`attempts` 是已经发生的尝试次数。
    Failed {
        id: usize,
        error: JobError,
        attempts: usize,
    },
}

impl JobOutcome {
    /// 返回结果对应的作业编号，无论成功还是失败。
    pub fn id(&self) -> usize {
        match self {
            Self::Success { id, .. } | Self::Failed { id, .. } => *id,
        }
    }
}

/// 一整批作业的报告。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchReport {
    /// 每个输入作业恰好一个结果，按作业编号从小到大排列。
    pub outcomes: Vec<JobOutcome>,
}

/// 整批作业无法开始的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchError {
    /// `limit` 为 0，无法确定并发上限。
    ZeroLimit,
    /// `max_attempts` 为 0，作业连第一次尝试都不会发生。
    ZeroAttempts,
}

/// 给一个 future 加上“最多经历 `patience` 次 `Pending`”的限制。
///
/// 语义：
///
/// - future 在允许范围内完成时，返回 `Ok(它的输出)`；
/// - future 每返回一次 `Pending`，已用配额加一；
/// - 当已经用掉 `patience` 次配额后，下一次 poll 仍然是 `Pending`，
///   就立即返回 `Err(TimedOut)`，并丢弃内部 future（即取消它）；
/// - `patience == 0` 表示不允许任何 `Pending`：第一次 poll 就必须完成。
///
/// 返回 `Err` 时，内部 future 的所有局部值都会正常析构，因此它持有的
/// guard、锁或连接可以被释放；但已经发生的副作用不会自动回滚。
///
/// 参数 `future` 可以是任意 future，不要求它实现 `Unpin`。
pub async fn with_timeout<F: Future>(future: F, patience: usize) -> Result<F::Output, TimedOut> {
    let _ = (&future, patience);
    todo!()
}

/// 对一条作业执行“至多 `policy.max_attempts` 次尝试”的发送流程。
///
/// 尝试编号从 1 开始。每次尝试的顺序是：
///
/// 1. 调用 `service.send(job, 当前尝试编号)`；
/// 2. 用 `policy.patience` 限制这次调用的等待；
/// 3. 把服务 panic 转成结构化失败，而不是让整个调度器崩溃。
///
/// 结果规则：
///
/// - 某次尝试成功：返回 `JobOutcome::Success`，`output` 是服务给出的文本，
///   `attempts` 是当前尝试编号；
/// - 服务返回 `ServiceFault::Temporary`：如果还有剩余次数就继续下一次；
///   如果这就是最后一次，返回 `TemporaryExhausted`，`last_reason` 保留
///   最后一次尝试给出的理由，`attempts` 等于 `max_attempts`；
/// - 服务返回 `ServiceFault::Permanent`：立即返回 `Permanent`，不重试；
/// - 某次尝试超时：立即返回 `TimedOut`，不重试；
/// - 某次尝试 panic：立即返回 `Panicked`，不重试。
///
/// 调用方必须保证 `policy.max_attempts >= 1`；`run_batch` 会在开始前验证，
/// 不会把 0 传进来。
pub async fn send_with_retries(
    service: &FakeService,
    job: &Job,
    policy: RetryPolicy,
) -> JobOutcome {
    let _ = (service, job, policy);
    todo!()
}

/// 调度一整批作业，并收集每个作业的最终结果。
///
/// 配置检查按以下顺序进行：
///
/// - `config.limit == 0` 返回 `Err(BatchError::ZeroLimit)`；
/// - 否则 `config.policy.max_attempts == 0` 返回 `Err(BatchError::ZeroAttempts)`。
///
/// 通过检查后：
///
/// - 空输入返回 `Ok`，`outcomes` 为空；
/// - 作业按输入顺序分成若干批，每批最多 `config.limit` 条；
/// - 同一批内的作业并发推进，因此同时进行中的 `send` 数量不会超过 `limit`；
/// - 一批全部结束后才开始下一批；
/// - 每个作业都使用同一份 `config.policy`；
/// - 单个作业失败不会丢弃同批其他作业的结果；
/// - 所有结果按作业编号从小到大排序，与完成顺序无关；
/// - 不留下无法观察的后台任务：本函数返回时，所有 future 都已完成或被取消。
///
/// 返回值中的 `outcomes` 长度等于输入作业数量，且每个作业恰好对应一个结果。
pub async fn run_batch(
    service: &FakeService,
    jobs: Vec<Job>,
    config: BatchConfig,
) -> Result<BatchReport, BatchError> {
    let _ = (service, &jobs, config);
    todo!()
}
