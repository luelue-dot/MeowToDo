import sys, os
sys.stdout.reconfigure(encoding='utf-8')

vue_content = r'''<script setup lang="ts">
console.log("hello world")
</script>'''

path = os.path.join(os.path.dirname(__file__), 'task-bar', 'index.vue')
with open(path, 'w', encoding='utf-8') as f:
    f.write(vue_content)
print('OK')
