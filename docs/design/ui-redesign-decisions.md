# UI Redesign Decisions

**Date:** 2026-11-21

## Grill-Me Output

### Primary Goal
Reduce visual clutter (most important)

### Essential Panels
Canvas only - everything else on demand

### Target Screen Size
Responsive - adapt to all

### Panel Integration
Tool Chest: Tab inside Tools panel

### Tool Selection
Icon strip (top 5) + dropdown for rest

### Panel Visibility
View menu toggles (current)

### Inspector Tabs
Single context-sensitive panel (Bluebeam pattern)

### Bottom Panels
Unified tab strip (Bluebeam pattern): Markups List | Quantities | Layers | Thumbnails

## Bluebeam Research Summary

### Panel Architecture
- **Left panel:** Can hold Pages, Tools, Tool Chest as tabs
- **Right panel:** Single Properties panel (context-sensitive sections)
- **Bottom panel:** Markups List (default tab) with overlay capability
- **Panel access bar:** Icons auto-hide, edge-trigger reveal

### Properties Panel Structure
- **Appearance** section (shape/text markup properties)
- **General** section (subject, author, comments, layer, status)
- **Measurement** section (length, area, volume, count, height/width, units)

### Bottom Panel Tabs
- **Markups List** - default, shows all markups in table
- **Quantities** - takeoff aggregation
- **Layers** - document layer management  
- **Thumbnails** - page navigation

## Implementation Approach
1. Panel consolidation (tabbed left panel)
2. Properties panel refactor
3. Bottom tab strip
4. Toolbar redesign
5. A11y polish