with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
i = 0
while i < len(lines):
    if 'matches!(' in lines[i] and 'mk.kind' in lines[i]:
        # Replace multi-line matches! with single line
        new_lines.append('                                matches!(mk.kind, MarkupKind::Text | MarkupKind::Callout)\n')
        # Skip the next 3 lines (mk.kind, MarkupKind..., ))
        i += 4
        continue
    new_lines.append(lines[i])
    i += 1

with open('/home/leonardolscinto/rubric/apps/rubric/src/ui/canvas.rs', 'w') as f:
    f.writelines(new_lines)

print('Done fixing matches!')
