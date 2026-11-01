with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'r') as f:
    content = f.read()

# Fix 1: ui_state.status = format!("Selected {} markup(s)", ui_state.selected.len());
# Split into two lines
old1 = '''                                ui_state.status = format!("Selected {} markup(s)", ui_state.selected.len());'''
new1 = '''                                ui_state.status =
                                format!("Selected {} markup(s)", ui_state.selected.len());'''
content = content.replace(old1, new1)
print(f'Fix 1: {content.count(new1)} replacements')

# Fix 2: ui_state.status = format!("Selected {} markup{}", n, ...) - collapse to split
old2 = '''                        ui_state.status = format!("Selected {} markup{}", n, if n == 1 { "" } else { "s" });'''
new2 = '''                        ui_state.status =
                            format!("Selected {} markup{}", n, if n == 1 { "" } else { "s" });'''
count = content.count(old2)
content = content.replace(old2, new2)
print(f'Fix 2: {count} replacements')

with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'w') as f:
    f.write(content)

print('Done')
