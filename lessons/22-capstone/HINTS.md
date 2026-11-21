# 第二十二课提示

提示按三层递进：

- **一级**：只给思考方向，适合刚开始动手。
- **二级**：把任务拆成小步骤，适合已经尝试但卡住。
- **三级**：接近完整的结构，用 `<<...>>` 标出仍需你自己完成的关键部分，不提供可直接粘贴的完整答案。

建议先看一级并动手试，仍然卡住再看下一级。函数文档和测试名才是需求来源，提示只帮你组织思路。

## 主练习

### 任务 1：`Record::try_new` 的规范化

**一级**

先决定检查顺序，再决定每一步是产生新文本还是只检查。分类和标签都需要先产生新文本再检查；备注不 trim，只检查禁用字符。想清楚“什么情况下应该返回错误”，而不是只想着怎么构造成功结果。

**二级**

1. 分类先 `trim`，再判断是否为空。
2. 三个字段的禁用字符集合并不相同：分类和备注禁用 `|` 与回车 `\r`、换行 `\n`、制表符 `\t`；标签还要额外禁用 `;`。
3. 标签处理是“逐个转换 → 跳过空项 → 检查 → 去重 → 最后统一排序”。先全部转换再排序，不要在处理过程中反复排序。
4. 去重可以在推入前判断，也可以推入后统一处理；但最终 `tags` 必须没有重复。
5. 备注保持原样，不能顺手 `trim`。
6. 最后组装 `Self`。注意 `category`、`tags`、`note` 的所有权都转移进结构体。

**三级**

```text
fn try_new(id, category, value, tags, note) -> Result<Self, ValidationError> {
    let category = <<去两端空白>>;
    if <<category 为空>> {
        return Err(<<对应的 ValidationError>>);
    }
    <<检查 category 的禁用字符，出错就返回>>;

    let mut normalized = Vec::new();
    for tag in tags {
        let tag = <<去空白并转小写>>;
        if <<tag 为空>> { continue; }
        <<检查 tag 的禁用字符>>;
        if <<normalized 还没有这个标签>> {
            normalized.push(tag);
        }
    }
    <<把 normalized 按字典序排序>>;

    <<检查 note 的禁用字符>>;

    Ok(Self { id, category, value, tags: normalized, note })
}

fn 检查(text, field, forbidden) -> Result<(), ValidationError> {
    <<找到第一个禁用字符 -> ForbiddenCharacter { field, character }>>
    <<没有找到 -> Ok(())>>
}
```

自己决定错误检查的先后：分类为空和分类含禁用字符同时成立时，测试只要求你选定一种并保持一致。

### 任务 2：`Record::parse_line` 与 `Record::to_line`

**一级**

`parse_line` 是“切分 → 检查形状 → 逐个解析 → 交给 `try_new`”。`to_line` 是它的反向操作。先让一个五个字段都非空的例子往返成功，再补空标签、空备注和各类错误。

**二级**

1. 用 `split('|')` 收集字段；在访问下标之前先检查字段数。
2. 第一段解析成 `u64`，第三段解析成 `i64`；失败时错误里要带原始文本。
3. 第四段为空表示没有标签，否则按 `;` 拆分。
4. 不要在这里重复写 trim、去重、排序；把它们交给 `try_new`，保证导入和存储共用一套规则。
5. `to_line` 用五个字段和四个分隔符拼成一行；`tags` 用 `;` 连接。注意结尾不要加 `\n`，换行由存储层负责。

**三级**

```text
pub fn parse_line(line: &str) -> Result<Self, ParseRecordError> {
    let fields: Vec<&str> = line.split('|').collect();

    if fields.len() != FIELD_COUNT {
        return Err(ParseRecordError::FieldCount { found: <<实际数量>> });
    }

    let id = <<解析 fields[0]；失败时保留原始文本>>;
    let value = <<解析 fields[2]；失败时保留原始文本>>;
    let tags = if <<fields[3] 为空>> {
        Vec::new()
    } else {
        <<按 ';' 拆分并转成 String>>
    };

    <<调用 try_new，并把 ValidationError 转成 ParseRecordError>>
}

pub fn to_line(&self) -> String {
    <<用 '|' 连接 id、category、value、tags.join(";")、note>>
}
```

检查一个容易忽略的例子：`1|book|0||`。它有两个空字段，仍然必须得到空 `tags` 和空 `note`，而不是字段数错误。

### 任务 3：`query::execute`

**一级**

把任务拆成三段独立逻辑：范围检查、筛选、排序与截断。先让默认查询（无筛选、按 id）通过，再逐项加条件。返回值是引用，不要为了省事克隆记录。

**二级**

1. 范围检查放在最前面，因为它是整个查询的前提。
2. 分类和标签的查询值只需要预处理一次。分类只 trim；标签 trim 后还要转小写，才能和规范化标签比较。
3. 每个筛选条件都写成“没有这个条件，或者条件成立”。五个条件用 `&&` 连接。
4. 排序先比较主键；`descending` 只反转主键；主键相等时再比较 `id`。
5. `limit` 一定在排序之后应用。
6. 返回 `Ok(selected)`，其中每个元素都借用 `records`。

**三级**

```text
pub fn execute<'a>(records: &'a [Record], query: &Query) -> Result<Vec<&'a Record>, QueryError> {
    if <<同时有 min 和 max 且 min > max>> {
        return Err(QueryError::InvalidRange { min, max });
    }

    let category = <<预处理 query.category>>;
    let tag = <<预处理 query.tag>>;

    let mut selected: Vec<&Record> = records.iter().filter(|record| {
        <<category 条件>>
            && <<tag 条件>>
            && <<min 条件>>
            && <<max 条件>>
            && <<contains 条件>>
    }).collect();

    selected.sort_by(|left, right| {
        let primary = <<按 query.sort 比较主键>>;
        let primary = <<如果 descending 就反转，否则保持>>;
        <<主键相等时按 id 从小到大>>
    });

    <<应用 limit>>

    Ok(selected)
}
```

`contains` 是大小写敏感的子串匹配；不需要自己写循环，`str::contains` 已经处理字符边界。`limit = Some(0)` 应得到空结果。

### 任务 4：统计

**一级**

`summarize` 是“遍历一次，持续更新四个字段”。`mean` 是“没有记录就没有平均值”。`group_by_category` 是“先按分类建表，再把表变成有序列表”。先想清楚空集合的每个字段应该是什么。

**二级**

1. `count`、`sum` 从 0 开始，`min`、`max` 从 `None` 开始。
2. 更新 `min`/`max` 时，第一次遇到记录要能放进 `Some`；已有值时再比较。可以用 `map_or`，也可以用 `match`。
3. `sum` 用 `i128::from(value)` 累加，避免两个大 `i64` 直接相加溢出。
4. 平均值要区分 `count == 0` 和 `count > 0`；不要用 `sum / count` 造成除零。
5. 分组容器选 `BTreeMap`。每遇到一条记录，先取得该分类的 `Summary`，再按和 `summarize` 相同的规则更新。
6. 最后把 map 转成 `Vec`。`BTreeMap` 的迭代顺序就是分类名升序。

**三级**

```text
pub fn summarize(records: &[Record]) -> Summary {
    let mut summary = Summary { count: 0, sum: 0, min: None, max: None };
    for record in records {
        summary.count += <<1>>;
        summary.sum += <<value 转成 i128>>;
        summary.min = <<用 record.value 更新 Option>>;
        summary.max = <<用 record.value 更新 Option>>;
    }
    summary
}

pub fn mean(&self) -> Option<f64> {
    if <<count 为 0>> {
        None
    } else {
        Some(<<sum / count，注意类型转换>>)
    }
}

pub fn group_by_category(records: &[Record]) -> Vec<(String, Summary)> {
    let mut groups: BTreeMap<String, Summary> = BTreeMap::new();
    for record in records {
        let summary = <<取得或创建该分类的空 Summary>>;
        <<按 count/sum/min/max 更新>>
    }
    <<把 map 转成 Vec>>
}
```

空集合的 `min`/`max` 必须是 `None`。把它们初始化成 `Some(0)` 会让空集合看起来像有一条值为 0 的记录。

### 任务 5：存储

**一级**

先做 `load`，再做 `save`。`load` 的关键是“文件不存在不是错误，坏行才要报错”。`save` 的关键是“先写别处，最后一步才替换目标”。想清楚每一步失败时原文件处于什么状态。

**二级**

`load`：

1. 打开文件。区分“不存在”和“其他 IO 错误”。
2. 逐行读取并保留真实行号；空行跳过，但行号不能重置。
3. 每个非空行先解析，失败时把路径和行号一起包进存储错误。
4. id 唯一性在读完所有行之前就要检查；记录每个 id 第一次出现的行号。
5. 返回按文件顺序排列的记录。

`save`：

1. 用现成的 `temporary_path` 得到临时路径。
2. 创建临时文件，逐条写 `to_line()` 加换行。
3. 先把用户态缓冲刷给操作系统，再请求同步到存储设备。
4. 关闭临时文件后 rename。
5. 整个写入过程要能统一处理失败：删除临时文件，返回带路径的写错误。
6. 不要在失败路径上动原目标文件。

**三级**

```text
pub fn load(path: &Path) -> Result<Vec<Record>, StoreError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if <<是 NotFound>> => return Ok(Vec::new()),
        Err(source) => return Err(<<带 path 的 Read>>),
    };

    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut seen = <<从 id 到首次行号的映射>>;

    for (index, line) in reader.lines().enumerate() {
        let line_number = <<index + 1>>;
        let line = <<处理读取这一行的 IO 错误>>;
        if <<这一行只含空白>> { continue; }

        let record = <<解析；失败时带 path 和 line_number>>;

        if let Some(first_line) = <<已经见过这个 id>> {
            return Err(<<DuplicateId，包含两条行号>>);
        }
        <<记录 id 到行号>>;
        records.push(record);
    }

    Ok(records)
}

pub fn save(path: &Path, records: &[Record]) -> Result<(), StoreError> {
    let temporary = temporary_path(path);
    let result = <<一个返回 io::Result<()> 的闭包，按顺序完成：创建、写入、flush、sync_all、关闭、rename>>;

    match result {
        Ok(()) => Ok(()),
        Err(source) => {
            <<删除临时文件>>;
            Err(<<带 path 的 Write>>)
        }
    }
}
```

需要自己补 `use`。如果编译器提示找不到 `lines`，通常是没有导入 `BufRead`；如果提示找不到 `flush`，通常是没有导入 `Write`。

### 任务 6：`output::render_records`

**一级**

先把所有行变成同样形状的数据，再统一计算列宽，最后逐行拼接。不要在生成每行时临时决定宽度，否则后面的行无法影响前面已经输出的内容。

**二级**

1. 第一行是表头，后面每条记录一行。
2. 标签为空显示 `-`，否则用逗号连接；备注为空也显示 `-`。
3. 先收集所有行，再计算前四列的最大字符数。初始化宽度时要把表头也算进去。
4. 前四列补齐并加两个空格；第五列不补齐，避免行尾空格。
5. 每行以 `\n` 结尾；空记录列表也要输出表头行。
6. 宽度用字符数，不用字节数。

**三级**

```text
pub fn render_records(records: &[&Record]) -> String {
    let mut rows: Vec<[String; 5]> = Vec::new();
    rows.push(<<表头五个字符串>>);

    for record in records {
        let tags = <<空 -> "-"，否则 join(",")>>;
        let note = <<空 -> "-"，否则原文本>>;
        rows.push([<<id, category, value, tags, note 转成 String>>]);
    }

    let mut widths = <<前四列的初始宽度>>;
    for row in &rows {
        for <<前四列>> {
            widths[i] = <<取当前宽度和该单元格字符数的较大值>>;
        }
    }

    let mut output = String::new();
    for row in &rows {
        for <<前四列>> {
            <<补齐到 widths[i]>>;
            <<追加两个空格>>;
        }
        <<追加第五列>>;
        output.push('\n');
    }
    output
}
```

补齐逻辑可以复用已有的 `pad` 辅助函数，也可以自己写；关键是按 `chars().count()` 计算空格数，而不是按 `len()`。

## 课后巩固：时段记录

### 任务 7：`parse_sessions`

**一级**

和 `parse_line` 类似，但这份文本有很多行，而且错误必须带行号。先决定“空行跳过但行号照算”，再逐个字段验证，最后处理重复键。

**二级**

1. 逐行遍历，行号从 1 开始；空行跳过，但后续行号不回退。
2. 切分后先检查字段数，再访问字段。
3. 日期只 trim，不转小写；项目 trim 后转小写。
4. 分钟先解析成 `u32`，再检查范围。解析失败和超范围是两种不同错误。
5. 重复键是 `(date, project)` 的组合，不是单独的日期或项目。
6. 每个错误都包在外层 `SessionLineError` 里，行号放在外层。
7. 返回按输入顺序排列的 `Vec<Session>`。

**三级**

```text
pub fn parse_sessions(text: &str) -> Result<Vec<Session>, SessionLineError> {
    let mut sessions = Vec::new();
    let mut seen = <<(date, project) -> 首次行号>>;

    for (index, raw) in text.lines().enumerate() {
        let line = <<index + 1>>;
        if <<空行>> { continue; }

        let fields: Vec<&str> = raw.split('|').collect();
        if <<字段数不是 3>> {
            return Err(SessionLineError { line, error: <<FieldCount>> });
        }

        let date = <<trim>>;
        if <<为空>> { return Err(<<带行号的 EmptyDate>>); }

        let project = <<trim + 转小写>>;
        if <<为空>> { return Err(<<带行号的 EmptyProject>>); }

        let minutes = <<解析 u32；失败返回 InvalidMinutes>>;
        if <<不在 1..=1440>> {
            return Err(<<带行号的 MinutesOutOfRange>>);
        }

        if let Some(first_line) = <<seen 中已有 (date, project)>> {
            return Err(<<DuplicateProjectDay，包含 first_line>>);
        }
        <<记录首次行号>>;
        sessions.push(Session { date, project, minutes });
    }

    Ok(sessions)
}
```

不要把行号丢掉后只返回 `SessionError`。测试会比较外层错误的 `line` 字段。

### 任务 8：汇总与最忙的一天

**一级**

两个函数都是“先按某个键累加，再选结果”。第一个按键是项目名，第二个按键是日期。需要稳定顺序时用 `BTreeMap`，不要用 `HashMap`。

**二级**

`total_minutes_by_project`：

1. 用 `BTreeMap<String, u64>`。
2. 每条把分钟数转成 `u64` 后累加到对应项目。
3. 最后转成 `Vec`；顺序自然按项目名升序。

`busiest_day`：

1. 用 `BTreeMap<&str, u64>`，键借用输入里的日期文本。
2. 累加每天的总分钟。
3. 遍历有序的 map，维护当前最佳结果；只有严格大于才替换，这样平局保留先遇到的更早日期。
4. 空输入返回 `None`。

**三级**

```text
pub fn total_minutes_by_project(sessions: &[Session]) -> Vec<(String, u64)> {
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    for session in sessions {
        <<把 u64::from(session.minutes) 加到 session.project 对应的项>>
    }
    <<totals 转成 Vec>>
}

pub fn busiest_day(sessions: &[Session]) -> Option<(&str, u64)> {
    let mut days: BTreeMap<&str, u64> = BTreeMap::new();
    for session in sessions {
        <<把分钟数加到 session.date 对应的项>>
    }

    let mut best: Option<(&str, u64)> = None;
    for (date, total) in days {
        if <<best 还没有值，或 total 严格更大>> {
            best = Some((date, total));
        }
    }
    best
}
```

`busiest_day` 返回的 `&str` 借用 `sessions` 里的日期。不要返回局部 `String` 的引用；那会触发类似 README 中 E0515 的编译错误。

## 自查清单

完成主练习后，逐项确认：

- [ ] `cargo test -p l22_capstone --no-run` 通过（只编译，不运行）。
- [ ] `cargo test -p l22_capstone` 全部通过（包含巩固任务）。
- [ ] `cargo clippy -p l22_capstone --all-targets -- -D warnings` 无警告。
- [ ] `cargo fmt -p l22_capstone -- --check` 通过。
- [ ] 没有为了消警告而全局 `#![allow(dead_code)]`。
- [ ] 没有把测试字面量写死成唯一逻辑。
- [ ] `import` 失败后原 store 内容没有改变。
- [ ] 同样的 store 连续运行两次 `list`，输出完全一致。

如果某项失败，先读测试名和错误行号，再回到 README 对应小节。不要先改测试。
