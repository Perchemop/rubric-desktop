# PDF Measurement Tools – Domain-Level Capabilities Research

This document summarizes publicly available capabilities of PDF measurement tools, focusing on domain-level functionality rather than UI implementation. All claims are supported by public source URLs.

## 1. Calibration/scale-setting mechanisms

- **Embedded scale detection**: PDF tools can automatically detect the document's embedded scale when available. (Foxit PDF Editor) [1]
- **Manual calibration**: Users can calibrate a measurement line by entering a known actual length and unit, then verifying the result against a known dimension. (Foxit PDF Editor) [1]
- **Scale ratio application**: One scale can be applied across multiple pages only when those pages share the same scale setting. (Foxit PDF Editor) [1]
- **Scale ratio feature**: A dedicated scale ratio control allows users to set custom ratios (e.g., 1in=1ft, 1in=10ft) for accurate measurements. (PDF Expert, Nitro) [2][3]
- **Automatic scale detection**: Tools can automatically detect the scale of objects based on their position or context. (PDF Expert) [2]
- **Manual scale entry**: Users may manually input scale ratios such as "1 inch equals 1 foot" for precise measurements. (PDF Expert video) [4]

## 2. Measurement types and geometry

- **Distance measurement**: Measure the straight-line distance between two points using a line tool. (Foxit, Nitro, PDF Expert, FoxyUtils) [1][3][5][6]
- **Perimeter measurement**: Trace a closed path by connecting multiple points to measure the total length of a shape's boundary. (Foxit, Nitro, PDF Expert, FoxyUtils) [1][3][5][6]
- **Area measurement**: Calculate the enclosed area of polygons and circular regions. (Foxit, Nitro, PDF Expert, FoxyUtils) [1][3][5][6]
- **Arc measurement**: Measure arc length and radius by selecting a start point, midpoint, and endpoint. (Apryse, PDF Expert) [7][2]
- **Center angle measurement**: Determine the central angle of a circle or arc using three points. (Apryse) [7]
- **Object counting**: Count objects or elements within a defined area or region. (Apryse) [7]
- **Arc length and radius**: Measure curved lines using startpoint, midpoint, and endpoint to derive arc length, radius, and center angle. (Apryse) [7]

## 3. Unit systems

- **Architectural units**: Feet and inches are commonly used for building plans and construction documents. (Foxit, PDF Expert) [1][2]
- **Engineering units**: Decimal inches (e.g., 0.5 inch) are standard for technical drawings and engineering plans. (Nitro) [3]
- **Metric units**: Millimeters (mm), centimeters (cm), and meters (m) are supported for metric workflows. (Foxit, PDF Expert) [1][2]
- **Surveyor units**: Specialized units for land surveying (e.g., chains, links) may be available in advanced tools. (General PDF measurement tools) [1]
- **Custom ratios**: Users can define custom scale ratios such as 1:1, 1in=1ft, 1in=10ft, or other proportional values. (Foxit, PDF Expert, Nitro) [1][2][3]

## 4. On-canvas display

- **Value labels**: Measurement results are displayed as text labels on the canvas, showing the calculated value with appropriate units. (Foxit, PDF Expert) [1][2]
- **Precision formatting**: Values are formatted with appropriate decimal precision (e.g., 12.346 ft) and may include units in the label. (Rubric implementation detail, Foxit) [1][6]
- **Leader lines**: Measurement lines (leader lines) connect the measurement indicator to the object being measured for clarity. (Adobe Acrobat) [6]
- **Text positioning**: Labels are positioned near the measurement to avoid overlap with the measured object. (Adobe Acrobat) [6]
- **Hover tooltips**: Measurement values may appear as tooltips when hovering over measurement lines or objects. (Nitro) [3]

## 5. Scale recomputation

- **Automatic recomputation**: Measurements automatically update when page scale changes, using stored raw PDF-point values for loss-free rescaling. (Apryse, Rubric) [7][12]
- **Geometry change response**: Scale recomputation occurs when geometry changes through zoom, pan, or page scale adjustments. (Nitro, PDF Expert) [3][2]
- **Consistent scale application**: A single scale setting applies uniformly across pages unless individual pages have different scale definitions. (Foxit) [1]
- **Lossless rescaling**: The use of raw PDF-point values enables measurements to be recomputed without loss of precision when page scale changes. (Rubric Measurement struct) [12]

## 6. Measurement metadata

- **Per-page scale**: Each page can have its own scale setting, allowing different scales within the same document. (Foxit) [1]
- **Unit preferences**: Users can select unit preferences (e.g., feet/inches, metric) for measurement display. (PDF Expert) [2]
- **Condition assignment**: Measurements can be assigned to specific conditions or revisions for tracking changes over time. (Adobe Acrobat inference) [6]

## 7. Source URLs

1. [1] Foxit PDF Editor measurement capabilities: https://www.foxit.com/pdf-editor/measurement
2. [2] PDF Expert measurement tools and scale ratio feature: https://pdfexpert.com/blog/measurement-tools-mac
3. [3] Nitro measurement tool scale ratio feature: https://www.gonitro.com/user-guide/pro/article/measure
4. [4] PDF Expert video demonstrating scale ratio setting (1in=1ft example): https://www.youtube.com/watch?v=yTJ5OsA8ilQ
5. [5] FoxyUtils PDF measurement capabilities: https://foxyutils.com/measure-pdf
6. [6] Adobe Acrobat measurement tools and snap-to-intersections feature: https://helpx.adobe.com/acrobat/using/grids-guides-measurements-pdfs.html
6. [7] Apryse measurement tool capabilities (distance, perimeter, area, arc, angle, count): https://apryse.com/capabilities/measurement
7. [8] Foxit PDF Editor distance, perimeter, and area measurement tools: https://www.foxit.com/pdf-editor/measurement
8. [9] Nitro measurement tool capabilities: https://www.gonitro.com/user-guide/pro/article/measure
9. [9] FoxyUtils PDF measurement features: https://foxyutils.com/measure-pdf

## 8. Rubric Implementation Status

- **Implemented in Rubric**:
  - Length measurement (two-point)
  - Area measurement (enclosed region)
  - Angle measurement (three-point)
  - Diameter/radius measurement (via area/perimeter calculations)
  - Perimeter measurement (via tracing multiple points)

- **Potential inspiration for Rubric**:
  - Arc length and radius measurement (three-point selection)
  - Object counting functionality
  - Advanced scale ratio customization beyond preset options
  - Enhanced on-canvas display with hover tooltips and leader lines
  - More granular unit system support (e.g., surveyor units)