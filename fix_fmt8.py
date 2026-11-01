with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'r') as f:
    content = f.read()

old = '''                        ui_state.status =
                                format!("Selected {} markup{}", n, if n == 1 { "" } else { "s" });'''
new = '''                        ui_state.status =
                            format!("Selected {} markup{}", n, if n == 1 { "" } else { "s" });'''
count = content.count(old)
content = content.replace(old, new)
print(f'Fix: {count} replacements')

with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'w') as f:
    f.write(content)

print('Done')
