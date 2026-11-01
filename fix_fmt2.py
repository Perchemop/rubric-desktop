T = '\t'

with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'r') as f:
    content = f.read()

# Fix 2: matches!(mk.kind, MarkupKind::Text | MarkupKind::Callout) - collapse
old2 = f'''{T*4}matches!(\n{T*5}mk.kind,\n{T*5}MarkupKind::Text | MarkupKind::Callout\n{T*4})'''
new2 = '''                                matches!(mk.kind, MarkupKind::Text | MarkupKind::Callout)'''
count = content.count(old2)
content = content.replace(old2, new2)
print(f'Fix 2 (matches! collapse): {count} replacements')

# Fix 7: ui_state.status = format!("Selected {} markup{}", n, ...) - collapse
old7 = '''                        ui_state.status = format!(\n                            "Selected {} markup{}", n, if n == 1 { "" } else { "s" });'''
new7 = '''                        ui_state.status = format!("Selected {} markup{}", n, if n == 1 { "" } else { "s" });'''
count = content.count(old7)
content = content.replace(old7, new7)
print(f'Fix 7 (format! markup{{}}): {count} replacements')

with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'w') as f:
    f.write(content)

print('Done')
