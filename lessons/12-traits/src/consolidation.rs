//! 课后巩固：图形面积报表。
//!
//! 报表要同时处理矩形和圆形：它们的数据字段不同，却能计算面积、按比例
//! 缩放，并用同一种格式写进报告。这个场景和主练习的数据、边界都不同，
//! 用来检验你是否能把 trait 用在“行为相同、数据不同”的另一类对象上。
//!
//! 请只替换各个 `todo!()`，不要修改类型定义、方法签名和文件底部的测试。

use std::fmt::{self, Display};

/// 图形操作失败的原因。
#[derive(Debug, PartialEq)]
pub enum ShapeError {
    /// 缩放因子必须大于 `0`；`0` 和负数都不接受。
    NonPositiveFactor,
}

/// 图形共同具备的行为。
///
/// 除了面积和缩放，它还提供一个默认的 `describe`：名称加冒号，再加保留
/// 两位小数的面积。实现可以覆盖它。
pub trait Shape {
    /// 图形名称。
    fn name(&self) -> &str;

    /// 返回面积。
    fn area(&self) -> f64;

    /// 按 `factor` 等比缩放。
    ///
    /// `factor` 必须大于 `0`；否则返回 `Err(ShapeError::NonPositiveFactor)`
    /// 并且不修改图形。
    fn scale(&mut self, factor: f64) -> Result<(), ShapeError>;

    /// 报告中的说明文字。
    ///
    /// 默认行为是 `名称: 面积`，面积保留两位小数，例如矩形 `2 × 3`
    /// 得到 `矩形: 6.00`。
    fn describe(&self) -> String {
        format!("{}: {:.2}", self.name(), self.area())
    }
}

/// 矩形。
#[derive(Debug)]
pub struct Rectangle {
    /// 宽。本练习假定构造时传入正数。
    pub width: f64,
    /// 高。本练习假定构造时传入正数。
    pub height: f64,
}

impl Shape for Rectangle {
    fn name(&self) -> &str {
        "矩形"
    }

    /// 返回 `width * height`。
    fn area(&self) -> f64 {
        let _ = &self;
        todo!()
    }

    /// 宽和高同时乘以 `factor`。
    fn scale(&mut self, factor: f64) -> Result<(), ShapeError> {
        let _ = (&self, factor);
        todo!()
    }
}

/// 圆形。
#[derive(Debug)]
pub struct Circle {
    /// 半径。本练习假定构造时传入正数。
    pub radius: f64,
}

impl Shape for Circle {
    fn name(&self) -> &str {
        "圆形"
    }

    /// 返回 `π * radius * radius`，其中 `π` 取标准库提供的圆周率常量
    /// （见 README 第十二节）。
    fn area(&self) -> f64 {
        let _ = &self;
        todo!()
    }

    /// 半径乘以 `factor`。
    fn scale(&mut self, factor: f64) -> Result<(), ShapeError> {
        let _ = (&self, factor);
        todo!()
    }

    /// 覆盖默认说明：圆形的关键信息是半径，而不是直接写面积。
    ///
    /// 固定格式为 `圆形（半径 X.XX）`，半径保留两位小数。
    fn describe(&self) -> String {
        todo!()
    }
}

/// 计算所有图形面积之和。空切片返回 `0.0`。
pub fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    let _ = &shapes;
    todo!()
}

/// 返回面积最大的图形的名称。
///
/// - 空切片返回 `None`；
/// - 面积并列最大时返回最先出现的那个；
/// - 返回名称的副本，不借用图形。
pub fn largest_name(shapes: &[Box<dyn Shape>]) -> Option<String> {
    let _ = &shapes;
    todo!()
}

/// 按比例缩放全部图形。
///
/// - `factor` 小于等于 `0` 时返回 `Err(ShapeError::NonPositiveFactor)`，
///   并且不修改任何图形；
/// - 否则逐个缩放并返回 `Ok(())`，顺序保持不变。
pub fn scale_all(shapes: &mut [Box<dyn Shape>], factor: f64) -> Result<(), ShapeError> {
    let _ = (&shapes, factor);
    todo!()
}

/// 一份逐行排列的面积报告。
#[derive(Debug, PartialEq, Eq)]
pub struct Report(pub Vec<String>);

impl Display for Report {
    /// 每行一个元素，行与行之间用一个换行符分隔，末尾不加多余的换行。
    ///
    /// 例如元素为 `["矩形: 6.00", "圆形（半径 2.00）"]` 时输出
    /// `矩形: 6.00\n圆形（半径 2.00）`；空报告输出空字符串。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = (&self, &f);
        todo!()
    }
}

/// 为每个图形生成一行说明，装进 `Report`。空切片得到空报告。
pub fn build_report(shapes: &[Box<dyn Shape>]) -> Report {
    let _ = &shapes;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{
        Circle, Rectangle, Report, Shape, ShapeError, build_report, largest_name, scale_all,
        total_area,
    };
    use std::f64::consts::PI;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    fn rectangle(width: f64, height: f64) -> Box<dyn Shape> {
        Box::new(Rectangle { width, height })
    }

    fn circle(radius: f64) -> Box<dyn Shape> {
        Box::new(Circle { radius })
    }

    #[test]
    fn rectangle_area_is_width_times_height() {
        let shape = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        assert_close(shape.area(), 12.0);
    }

    #[test]
    fn circle_area_is_pi_times_radius_squared() {
        let shape = Circle { radius: 2.0 };
        assert_close(shape.area(), 4.0 * PI);
    }

    #[test]
    fn rectangle_uses_the_default_description() {
        let shape = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(shape.describe(), "矩形: 6.00");
    }

    #[test]
    fn circle_overrides_the_description() {
        let shape = Circle { radius: 2.0 };
        assert_eq!(shape.describe(), "圆形（半径 2.00）");
    }

    #[test]
    fn total_area_of_an_empty_slice_is_zero() {
        assert_close(total_area(&[]), 0.0);
    }

    #[test]
    fn total_area_adds_every_shape() {
        let shapes = vec![rectangle(2.0, 3.0), circle(1.0)];
        assert_close(total_area(&shapes), 6.0 + PI);
    }

    #[test]
    fn largest_name_of_an_empty_slice_is_none() {
        assert_eq!(largest_name(&[]), None);
    }

    #[test]
    fn largest_name_returns_the_biggest_shape() {
        let shapes = vec![rectangle(2.0, 3.0), circle(2.0), rectangle(1.0, 1.0)];
        assert_eq!(largest_name(&shapes), Some(String::from("圆形")));
    }

    #[test]
    fn largest_name_keeps_the_first_shape_on_a_tie() {
        let shapes = vec![rectangle(PI, 1.0), circle(1.0)];
        assert_eq!(largest_name(&shapes), Some(String::from("矩形")));
    }

    #[test]
    fn scaling_a_rectangle_multiplies_both_sides() {
        let mut shape = Rectangle {
            width: 2.0,
            height: 3.0,
        };
        assert_eq!(shape.scale(2.0), Ok(()));
        assert_close(shape.area(), 24.0);
    }

    #[test]
    fn scaling_a_circle_to_half_size_quarters_its_area() {
        let mut shape = Circle { radius: 2.0 };
        assert_eq!(shape.scale(0.5), Ok(()));
        assert_close(shape.area(), PI);
    }

    #[test]
    fn zero_or_negative_scale_is_rejected_without_changes() {
        let mut shape = Rectangle {
            width: 2.0,
            height: 3.0,
        };

        assert_eq!(shape.scale(0.0), Err(ShapeError::NonPositiveFactor));
        assert_eq!(shape.scale(-1.0), Err(ShapeError::NonPositiveFactor));
        assert_close(shape.area(), 6.0);
    }

    #[test]
    fn scale_all_rejects_an_invalid_factor_before_touching_any_shape() {
        let mut shapes = vec![rectangle(2.0, 3.0), circle(1.0)];

        assert_eq!(
            scale_all(&mut shapes, 0.0),
            Err(ShapeError::NonPositiveFactor)
        );
        assert_close(shapes[0].area(), 6.0);
        assert_close(shapes[1].area(), PI);
    }

    #[test]
    fn scale_all_updates_every_shape() {
        let mut shapes = vec![rectangle(2.0, 3.0), circle(1.0)];

        assert_eq!(scale_all(&mut shapes, 2.0), Ok(()));
        assert_close(shapes[0].area(), 24.0);
        assert_close(shapes[1].area(), 4.0 * PI);
    }

    #[test]
    fn build_report_uses_each_shape_description() {
        let shapes = vec![rectangle(2.0, 3.0), circle(2.0)];
        assert_eq!(
            build_report(&shapes),
            Report(vec![
                String::from("矩形: 6.00"),
                String::from("圆形（半径 2.00）"),
            ])
        );
    }

    #[test]
    fn report_displays_one_line_per_shape() {
        let shapes = vec![rectangle(2.0, 3.0), circle(2.0)];
        assert_eq!(
            format!("{}", build_report(&shapes)),
            "矩形: 6.00\n圆形（半径 2.00）"
        );
    }

    #[test]
    fn empty_report_displays_as_an_empty_string() {
        assert_eq!(format!("{}", build_report(&[])), "");
    }
}
