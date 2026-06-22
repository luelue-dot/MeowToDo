import codecs

path = 'E:/26cyf/01代码/BongoCat-master/src/pages/main/index.vue'
with codecs.open(path, 'r', encoding='utf-8-sig') as f:
    content = f.read()

# Find the task-completed listener and add dropdown listeners after it
old_listener = 'useTauriListen(\"task-completed\", () => {\r\n  // Trigger expression 2 (the celebration expression) on task completion\r\n  live2d.setExpression(2)\r\n})'

if old_listener in content:
    new_listeners = old_listener + '\r\n\r\nuseTauriListen(\"dropdown-mouse-enter\", () => {\r\n  dropdownHovered.value = true\r\n  if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }\r\n})\r\n\r\nuseTauriListen(\"dropdown-mouse-leave\", () => {\r\n  dropdownHovered.value = false\r\n  hideTimer = setTimeout(hideTaskDropdown, 400)\r\n})'
    content = content.replace(old_listener, new_listeners)
    with codecs.open(path, 'w', encoding='utf-8-sig') as f:
        f.write(content)
    print('Added dropdown listeners')
else:
    print('Could not find old listener')
    # Debug
    idx = content.find('task-completed')
    if idx >= 0:
        print(repr(content[idx-10:idx+200]))
