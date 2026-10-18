# Data Model

## Entities
- **Project**: id, name, documents[], active_document, tool_sets[], settings, created, modified
- **Document**: id, title, source_path?, pages[], markups[], created, modified
- **Page**: index, width, height (pt), rotation (0/90/180/270), scale (Scale)
- **Scale**: unit (MeasureUnit), units_per_point (f64)
- **Markup**: id, page (usize), kind (MarkupKind), geometry (MarkupGeometry), style (MarkupStyle), author, subject, status, layer?, locked, comments[], measurement?, created, modified
- **MarkupGeometry**: Bounds(Rect), Segment{a,b}, Path{points,closed}, Ink{strokes: Vec<Vec<Pt>>}, Text{at,wrap,text}, Callout{rect,tail,text}, Angle{a,vertex,b}, Count{at,number}
- **MarkupStyle**: stroke (Color), fill (Color), fill_mode (FillMode), width (f32), dash (DashPattern), font_size (f32)
- **Color**: r,g,b,a (u8)
- **FillMode**: None/Solid/Highlight
- **DashPattern**: Solid/Dashed/Dotted
- **ReviewStatus**: None/Accepted/Rejected/Completed/Cancelled/Alert
- **Comment**: id, author, text, created (DateTime<Utc>)
- **Measurement**: kind (MeasureKind), value (f64 in scale units), value_points (f64)
- **ToolSet**: id, name, tools (MarkupKind[])
- **Settings**: author, auto_save, auto_save_interval_secs, ui_scale, show_grid, show_measurements
- **Tool** (UI): Select + all markup kinds
- **History**: snapshots[ProjectSnapshot], cursor (undo/redo)
- **ProjectSnapshot**: project clone

## Primitives
- **Pt**: x,y (f32)
- **Rect**: x,y,w,h (f32); helpers: right(), bottom(), center(), contains(), expanded(m), union(), intersects(), translated(dx,dy), from_corners, from_bounds

## Units
**MeasureUnit**: Point, Inch, Foot, Yard, Mile, Millimeter, Centimeter, Meter, Kilometer. points_per_unit() gives conversion.

**MeasureKind**: Length, Perimeter, Area, Angle, Diameter, Radius, Count. is_area() for unit suffix.

## Markup kinds (22)
Review: Highlight, Strikeout, Underline, Squiggly, Text, Callout, Note, Stamp.  
Shapes: Rectangle, Ellipse, Cloud, Polygon.  
Paths: Polyline, Line, Arrow, Ink.  
Measure: MeasureLength, MeasurePerimeter, MeasureArea, MeasureAngle, MeasureDiameter, MeasureRadius, MeasureCount.

## Invariants
- Markup.page < document.pages.len()
- All points are finite f32.
- Geometry contains valid numeric ranges (width/height >=0 where applicable).
- Measurement recomputed after geometry change; area uses polygon area; radius = length/2.
- History bounded to 128 snapshots; pushing truncates redo branch.

## Serialisation
serde + serde_json. DateTime<Utc> serialised as RFC3339. Enums serde_rename "snake_case" where stable.

## Equality
Derived PartialEq for testing; clones are intentional for history snapshots.
