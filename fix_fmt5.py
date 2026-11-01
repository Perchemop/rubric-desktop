with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'r') as f:
    content = f.read()

# Fix the broken matches! block
old = '''                                let kind_ok = m
                                    .map(|mk| {
                                matches!(mk.kind, MarkupKind::Text | MarkupKind::Callout)
                                    .and_then(|d| m.map(|mk| d.is_markup_editable(mk)))
                                    .unwrap_or(false);'''
new = '''                                let kind_ok = m
                                    .map(|mk| {
                                        matches!(mk.kind, MarkupKind::Text | MarkupKind::Callout)
                                    })
                                    .unwrap_or(false);'''

count = content.count(old)
content = content.replace(old, new)
print(f'Fix matches!: {count} replacements')

with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'w') as f:
    f.write(content)

print('Done')
