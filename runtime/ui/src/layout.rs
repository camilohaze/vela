//! Layout system types and constraints

/// Box constraints for layout calculations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxConstraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl BoxConstraints {
    /// Create new box constraints
    pub fn new(
        min_width: f32,
        max_width: f32,
        min_height: f32,
        max_height: f32,
    ) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    /// Create tight constraints (fixed size)
    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    /// Create loose constraints (flexible within bounds)
    pub fn loose(size: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: size.width,
            min_height: 0.0,
            max_height: size.height,
        }
    }

    /// Create unconstrained constraints
    pub fn unconstrained() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    /// Check if constraints are valid
    pub fn is_valid(&self) -> bool {
        self.min_width <= self.max_width
            && self.min_height <= self.max_height
            && self.min_width >= 0.0
            && self.min_height >= 0.0
    }

    /// Constrain a size to these constraints
    pub fn constrain(&self, size: Size) -> Size {
        Size {
            width: size.width.clamp(self.min_width, self.max_width),
            height: size.height.clamp(self.min_height, self.max_height),
        }
    }
}

/// Size representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Create new size
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Zero size
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Infinite size
    pub fn infinite() -> Self {
        Self::new(f32::INFINITY, f32::INFINITY)
    }
}

/// Position/offset representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    /// Create new offset
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero offset
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
}

/// Edge insets for padding/margin
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeInsets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl EdgeInsets {
    /// All sides equal
    pub fn all(value: f32) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    /// Symmetric insets
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }

    /// Only horizontal insets
    pub fn horizontal(value: f32) -> Self {
        Self {
            left: value,
            top: 0.0,
            right: value,
            bottom: 0.0,
        }
    }

    /// Only vertical insets
    pub fn vertical(value: f32) -> Self {
        Self {
            left: 0.0,
            top: value,
            right: 0.0,
            bottom: value,
        }
    }

    /// Individual insets
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Total horizontal inset
    pub fn horizontal_total(&self) -> f32 {
        self.left + self.right
    }

    /// Total vertical inset
    pub fn vertical_total(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Alignment for positioning
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alignment {
    pub x: f32, // -1.0 = left, 0.0 = center, 1.0 = right
    pub y: f32, // -1.0 = top, 0.0 = center, 1.0 = bottom
}

impl Alignment {
    /// Top-left alignment
    pub fn top_left() -> Self {
        Self { x: -1.0, y: -1.0 }
    }

    /// Top-center alignment
    pub fn top_center() -> Self {
        Self { x: 0.0, y: -1.0 }
    }

    /// Top-right alignment
    pub fn top_right() -> Self {
        Self { x: 1.0, y: -1.0 }
    }

    /// Center-left alignment
    pub fn center_left() -> Self {
        Self { x: -1.0, y: 0.0 }
    }

    /// Center alignment
    pub fn center() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Center-right alignment
    pub fn center_right() -> Self {
        Self { x: 1.0, y: 0.0 }
    }

    /// Bottom-left alignment
    pub fn bottom_left() -> Self {
        Self { x: -1.0, y: 1.0 }
    }

    /// Bottom-center alignment
    pub fn bottom_center() -> Self {
        Self { x: 0.0, y: 1.0 }
    }

    /// Bottom-right alignment
    pub fn bottom_right() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    /// Convert alignment to offset within a size
    pub fn along_size(&self, size: Size) -> Offset {
        Offset {
            x: (size.width / 2.0) * (self.x + 1.0),
            y: (size.height / 2.0) * (self.y + 1.0),
        }
    }
}

/// Main axis alignment for flex layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainAxisAlignment {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross axis alignment for flex layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossAxisAlignment {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

/// Main axis size for flex layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainAxisSize {
    Min,
    Max,
}

/// Position for stack children
#[derive(Debug, Clone)]
pub struct Position {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Position {
    /// Create new position
    pub fn new(left: f32, top: f32) -> Self {
        Self {
            left: Some(left),
            top: Some(top),
            right: None,
            bottom: None,
            width: None,
            height: None,
        }
    }

    /// Create position with width and height
    pub fn sized(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left: Some(left),
            top: Some(top),
            right: None,
            bottom: None,
            width: Some(width),
            height: Some(height),
        }
    }
}

/// Stack fit options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackFit {
    Loose,
    Expand,
    Passthrough,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_constraints() {
        let constraints = BoxConstraints::new(10.0, 100.0, 20.0, 200.0);
        assert!(constraints.is_valid());

        let size = Size::new(50.0, 50.0);
        let constrained = constraints.constrain(size);
        assert_eq!(constrained, size);

        let oversized = Size::new(150.0, 250.0);
        let constrained = constraints.constrain(oversized);
        assert_eq!(constrained, Size::new(100.0, 200.0));
    }

    #[test]
    fn test_edge_insets() {
        let insets = EdgeInsets::all(10.0);
        assert_eq!(insets.horizontal_total(), 20.0);
        assert_eq!(insets.vertical_total(), 20.0);

        let symmetric = EdgeInsets::symmetric(5.0, 15.0);
        assert_eq!(symmetric.left, 5.0);
        assert_eq!(symmetric.top, 15.0);
    }

    #[test]
    fn test_alignment() {
        let center = Alignment::center();
        let size = Size::new(200.0, 100.0);
        let offset = center.along_size(size);
        assert_eq!(offset.x, 100.0); // Half width
        assert_eq!(offset.y, 50.0);  // Half height

        let top_left = Alignment::top_left();
        let offset = top_left.along_size(size);
        assert_eq!(offset.x, 0.0);
        assert_eq!(offset.y, 0.0);
    }

    #[test]
    fn test_position() {
        let pos = Position::new(10.0, 20.0);
        assert_eq!(pos.left, Some(10.0));
        assert_eq!(pos.top, Some(20.0));
        assert_eq!(pos.width, None);

        let sized = Position::sized(10.0, 20.0, 100.0, 50.0);
        assert_eq!(sized.width, Some(100.0));
        assert_eq!(sized.height, Some(50.0));
    }
}