with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'r') as f:
    content = f.read()

# Fix 1: format!("Selected {} markup(s)", ui_state.selected.len()) - collapse to one line
old1 = '''                                ui_state.status = format!(
                                    "Selected {} markup(s)",
                                    ui_state.selected.len()
                                );'''
new1 = '''                                ui_state.status = format!("Selected {} markup(s)", ui_state.selected.len());'''
count = content.count(old1)
content = content.replace(old1, new1)
print(f'Fix 1 (format! Select markup(s)): {count} replacements')

# Fix 2: matches!(mk.kind, MarkupKind::Text | MarkupKind::Callout) - collapse
old2 = '''                                matches!(
                                    mk.kind,
                                    MarkupKind::Text | MarkupKind::Callout
                                )'''
new2 = '''                                matches!(mk.kind, MarkupKind::Text | MarkupKind::Callout)'''
count = content.count(old2)
content = content.replace(old2, new2)
print(f'Fix 2 (matches! collapse): {count} replacements')

# Fix 3: MarkupGeometry::Text { text, .. } => Some(text.clone()), - collapse
old3 = '''                                        MarkupGeometry::Text { text, .. } => {\n                                            Some(text.clone())\n                                        }\n                                        MarkupGeometry::Callout { text, .. } => {\n                                            Some(text.clone())\n                                        }\n                                        _ => None,'''
new3 = '''                                        MarkupGeometry::Text { text, .. } => Some(text.clone()),\n                                        MarkupGeometry::Callout { text, .. } => Some(text.clone()),\n                                        _ => None,'''
count = content.count(old3)
content = content.replace(old3, new3)
print(f'Fix 3 (Text/Callout collapse): {count} replacements')

# Fix 4: ui_state.status = "Locked layer — edits disabled".to_string(); - collapse
old4 = '''                                    ui_state.status =\n                                        "Locked layer — edits disabled".to_string();'''
new4 = '''                                    ui_state.status = "Locked layer — edits disabled".to_string();'''
count = content.count(old4)
content = content.replace(old4, new4)
print(f'Fix 4 (locked layer status): {count} replacements')

# Fix 5: ui_state.text_edit = Some(...) - collapse
old5 = '''                                    ui_state.text_edit =\n                                        Some(super::app::TextEditState {\n                                            id,\n                                            original: original.clone(),\n                                            text: original,\n                                            focus_requested: false,\n                                        });'''
new5 = '''                                    ui_state.text_edit = Some(super::app::TextEditState {\n                                        id,\n                                        original: original.clone(),\n                                        text: original,\n                                        focus_requested: false,\n                                    });'''
count = content.count(old5)
content = content.replace(old5, new5)
print(f'Fix 5 (text_edit = Some): {count} replacements')

# Fix 6: ui_state.marquee_start = Some(...) - collapse
old6 = '''                        ui_state.marquee_start = Some(\n                            response.hover_pos().unwrap_or(Pos2::ZERO),\n                        );'''
new6 = '''                        ui_state.marquee_start = Some(response.hover_pos().unwrap_or(Pos2::ZERO));'''
count = content.count(old6)
content = content.replace(old6, new6)
print(f'Fix 6 (marquee_start): {count} replacements')

# Fix 7: ui_state.status = format!("Selected {} markup{}", ...) - collapse
old7 = '''                        ui_state.status = format!(\n                            "Selected {} markup{}", n, if n == 1 { "" } else { "s" });'''
new7 = '''                        ui_state.status = format!("Selected {} markup{}", n, if n == 1 { "" } else { "s" });'''
count = content.count(old7)
content = content.replace(old7, new7)
print(f'Fix 7 (format! markup{{}}): {count} replacements')

with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'w') as f:
    f.write(content)

print('Done')
